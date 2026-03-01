//! GPU-accelerated renderer using wgpu
//!
//! This module provides GPU rendering via wgpu/iced_wgpu,
//! rendering to an off-screen texture and reading back to CPU for KMS display.

use anyhow::{anyhow, Context, Result};
use iced::advanced::renderer::Renderer as _;
use iced::advanced::widget::Tree;
use iced::advanced::{self, Layout, Widget};
use iced::{Color, Element, Renderer, Size, Theme};
use iced_wgpu::engine::Engine;
use iced_wgpu::graphics::Viewport;
use iced_wgpu::wgpu;
use tracing::{info, warn};

/// GPU renderer that uses wgpu for accelerated rendering
pub struct GpuRenderer {
    /// wgpu instance
    #[allow(dead_code)]
    instance: wgpu::Instance,
    /// wgpu adapter (GPU device handle)
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    /// wgpu device
    device: wgpu::Device,
    /// wgpu queue
    queue: wgpu::Queue,
    /// Render target texture
    render_texture: wgpu::Texture,
    /// Render texture view
    texture_view: wgpu::TextureView,
    /// CPU-readable buffer for readback
    readback_buffer: wgpu::Buffer,
    /// Texture dimensions
    width: u32,
    height: u32,
    /// Iced wgpu renderer
    iced_renderer: Renderer,
    /// Viewport for rendering
    viewport: Viewport,
    /// Staging buffer for async readback
    staging_buffer: Option<wgpu::Buffer>,
}

impl GpuRenderer {
    /// Create a new GPU renderer with the specified dimensions
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        info!("Initializing wgpu GPU renderer: {}x{}", width, height);

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });

        // Request adapter (GPU)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None, // Headless rendering
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("Failed to find GPU adapter"))?;

        let info = adapter.get_info();
        info!(
            "GPU adapter: {} ({:?})",
            info.name, info.backend
        );

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Iced GPU Renderer"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .context("Failed to create wgpu device")?;

        // Create render target texture
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Target"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create CPU-readable buffer for readback
        // KMS expects BGRX format (4 bytes per pixel), but wgpu renders RGBA
        let buffer_size = (width * height * 4) as u64;
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create staging buffer for async operations
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Initialize iced wgpu renderer
        let backend = iced_wgpu::Backend::new(
            &device,
            &queue,
            iced_wgpu::Settings::default(),
            iced_wgpu::backend::format(),
        );

        let iced_renderer = Renderer::new(backend, iced::Font::DEFAULT, width, height);

        // Create viewport
        let viewport = Viewport::with_physical_size(
            iced::Size::new(width, height),
            1.0, // scale factor
        );

        info!("GPU renderer initialized successfully");

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            render_texture,
            texture_view,
            readback_buffer,
            width,
            height,
            iced_renderer,
            viewport,
            staging_buffer: Some(staging_buffer),
        })
    }

    /// Render a frame and return the pixel buffer (BGRX format for KMS)
    pub fn render_frame(&mut self, view: &Element<Message>, width: u32, height: u32) -> Result<Vec<u8>>
    where
        Message: Clone + std::fmt::Debug,
    {
        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render the iced UI to texture
        self.render_iced_to_texture(view, &mut encoder)?;

        // Copy texture to readback buffer
        let texture_copy_view = wgpu::ImageCopyTexture {
            texture: &self.render_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };

        let buffer_copy_view = wgpu::ImageCopyBuffer {
            buffer: &self.readback_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
        };

        encoder.copy_texture_to_buffer(
            texture_copy_view,
            buffer_copy_view,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));

        // Read back the buffer
        let buffer_slice = self.readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);

        rx.recv().context("Failed to map buffer")??;

        // Copy data and convert RGBA -> BGRX
        let data = buffer_slice.get_mapped_range();
        let mut kms_buffer = vec![0u8; (width * height * 4) as usize];

        for (i, pixel) in data.chunks_exact(4).enumerate() {
            let idx = i * 4;
            if idx + 3 < kms_buffer.len() {
                // Convert RGBA (from wgpu) to BGRX (for KMS)
                kms_buffer[idx] = pixel[2];     // B
                kms_buffer[idx + 1] = pixel[1]; // G
                kms_buffer[idx + 2] = pixel[0]; // R
                kms_buffer[idx + 3] = 0xFF;     // X
            }
        }

        drop(data);
        self.readback_buffer.unmap();

        Ok(kms_buffer)
    }

    /// Render iced UI to the texture
    fn render_iced_to_texture<Message>(
        &mut self,
        view: &Element<Message>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()>
    where
        Message: Clone + std::fmt::Debug,
    {
        // For now, we'll clear the texture to the background color
        // Full iced_wgpu integration requires more setup
        
        let clear_color = wgpu::Color {
            r: 0.15,
            g: 0.15,
            b: 0.2,
            a: 1.0,
        };

        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Iced Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // TODO: Full iced_wgpu integration
        // This would involve:
        // 1. Creating a compatible iced Backend
        // 2. Using iced's render pipeline
        // 3. Handling text rendering via glyphon
        // 4. Managing draw commands
        
        // For now, we just clear to show the GPU is working
        warn!("Full iced_wgpu rendering not yet implemented - clearing to background color");

        Ok(())
    }
}

/// Placeholder message type for rendering
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ButtonPressed(String),
    ToggleAnimation,
}
