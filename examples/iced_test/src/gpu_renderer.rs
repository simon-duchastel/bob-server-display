//! GPU-accelerated renderer using wgpu - Step 1: Proper Setup
//!
//! This step properly initializes iced_wgpu with:
//! - iced_wgpu::Engine for managing GPU resources
//! - Proper Viewport with physical/logical size handling
//! - Correct Backend configuration for off-screen rendering

use anyhow::{anyhow, Context, Result};
use iced::advanced::graphics::color;
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::{self, Layout, Widget};
use iced::{Color, Element, Length, Renderer, Size, Theme};
use iced_wgpu::graphics::Viewport;
use iced_wgpu::wgpu;
use iced_wgpu::{self, Backend, Engine, Settings};
use tracing::{info, warn};

/// GPU renderer with proper iced_wgpu setup
pub struct GpuRenderer {
    /// wgpu instance
    #[allow(dead_code)]
    instance: wgpu::Instance,
    /// wgpu adapter
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
    /// Iced wgpu engine - manages GPU resources and render pipeline
    engine: Engine,
    /// Iced wgpu backend - handles primitive rendering
    backend: Backend,
    /// Viewport - manages coordinate spaces and scaling
    viewport: Viewport,
    /// Scale factor for DPI handling
    scale_factor: f64,
}

impl GpuRenderer {
    /// Create a new GPU renderer with proper iced_wgpu initialization
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        info!("Step 1: Initializing iced_wgpu with proper setup: {}x{}", width, height);

        // Create wgpu instance with Vulkan and GLES backends
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });

        // Request high-performance GPU adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("Failed to find GPU adapter"))?;

        let adapter_info = adapter.get_info();
        info!("GPU adapter: {} ({:?})", adapter_info.name, adapter_info.backend);

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

        // Create render target texture for off-screen rendering
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
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create CPU-readable buffer for readback
        let buffer_size = (width * height * 4) as u64;
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // STEP 1: Proper iced_wgpu Engine setup
        // Engine manages the GPU resources, pipelines, and memory
        let engine = Engine::new(
            &adapter,
            &device,
            &queue,
            Settings {
                present_mode: wgpu::PresentMode::AutoVsync,
                ..Default::default()
            },
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        // STEP 1: Create Backend for rendering primitives
        // Backend handles the actual drawing of iced primitives (quads, text, etc.)
        let backend = Backend::new(&device, &engine, iced::Font::DEFAULT, (width, height));

        // STEP 1: Proper Viewport setup
        // Viewport manages the transformation between logical and physical pixels
        // This is crucial for proper DPI/scaling handling
        let scale_factor = 1.0; // No scaling for embedded display
        let viewport = Viewport::with_physical_size(
            iced::Size::new(width, height),
            scale_factor,
        );

        info!("Step 1 complete: Engine, Backend, and Viewport initialized");
        info!("  - Physical size: {}x{}", width, height);
        info!("  - Logical size: {}x{}", 
            width as f64 / scale_factor, 
            height as f64 / scale_factor
        );
        info!("  - Scale factor: {}", scale_factor);

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
            engine,
            backend,
            viewport,
            scale_factor,
        })
    }

    /// Render a frame and return the pixel buffer (BGRX format for KMS)
    pub fn render_frame(&mut self, view: &Element<Message>, width: u32, height: u32) -> Result<Vec<u8>>
    where
        Message: Clone + std::fmt::Debug + 'static,
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
                kms_buffer[idx] = pixel[2]; // B
                kms_buffer[idx + 1] = pixel[1]; // G
                kms_buffer[idx + 2] = pixel[0]; // R
                kms_buffer[idx + 3] = 0xFF; // X
            }
        }

        drop(data);
        self.readback_buffer.unmap();

        Ok(kms_buffer)
    }

    /// Render iced UI to the texture using the properly configured backend
    fn render_iced_to_texture<Message>(
        &mut self,
        view: &Element<Message>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()>
    where
        Message: Clone + std::fmt::Debug + 'static,
    {
        // Clear the texture to background color
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

        warn!("Step 1: Setup complete. Next steps needed:");
        warn!("  - Step 2: Extract and render primitives");
        warn!("  - Step 3: Add text rendering with glyphon");
        warn!("  - Step 4: Encode draw commands");

        Ok(())
    }

    /// Get the viewport for layout calculations
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Get the scale factor
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}

/// Message type for iced rendering
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ButtonPressed(String),
    ToggleAnimation,
}
