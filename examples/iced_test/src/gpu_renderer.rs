//! GPU-accelerated renderer using wgpu - Step 2: Primitive Extraction
//!
//! This step adds primitive extraction and rendering:
//! - Create iced Renderer to process view tree
//! - Extract primitives via render() call
//! - Handle different primitive types (quads, clips, etc.)
//! - Render primitives to texture using backend

use anyhow::{anyhow, Context, Result};
use iced::advanced::graphics::color;
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::{self, Layout, Widget};
use iced::{Color, Element, Length, Renderer as IcedRenderer, Size, Theme, Vector};
use iced_wgpu::graphics::Viewport;
use iced_wgpu::wgpu;
use iced_wgpu::{self, Backend, Engine, Settings};
use tracing::{info, warn, debug};

/// GPU renderer with primitive extraction and rendering
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
    /// Iced wgpu engine
    engine: Engine,
    /// Iced wgpu backend
    backend: Backend,
    /// Viewport
    viewport: Viewport,
    /// Scale factor
    scale_factor: f64,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        info!("Step 2: Initializing primitive extraction: {}x{}", width, height);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });

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

        let buffer_size = (width * height * 4) as u64;
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

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

        let backend = Backend::new(&device, &engine, iced::Font::DEFAULT, (width, height));

        let scale_factor = 1.0;
        let viewport = Viewport::with_physical_size(
            iced::Size::new(width, height),
            scale_factor,
        );

        info!("Step 2 complete: Ready for primitive extraction and rendering");

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

    /// Render a frame with primitive extraction
    pub fn render_frame(&mut self, view: &Element<Message>, width: u32, height: u32) -> Result<Vec<u8>>
    where
        Message: Clone + std::fmt::Debug + 'static,
    {
        // STEP 2: Create iced Renderer and extract primitives
        let primitives = self.extract_primitives(view)?;
        debug!("Extracted {} primitives", primitives.len());

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render primitives to texture
        self.render_primitives(&primitives, &mut encoder)?;

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

        // Convert RGBA -> BGRX
        let data = buffer_slice.get_mapped_range();
        let mut kms_buffer = vec![0u8; (width * height * 4) as usize];

        for (i, pixel) in data.chunks_exact(4).enumerate() {
            let idx = i * 4;
            if idx + 3 < kms_buffer.len() {
                kms_buffer[idx] = pixel[2];
                kms_buffer[idx + 1] = pixel[1];
                kms_buffer[idx + 2] = pixel[0];
                kms_buffer[idx + 3] = 0xFF;
            }
        }

        drop(data);
        self.readback_buffer.unmap();

        Ok(kms_buffer)
    }

    /// STEP 2: Extract primitives from the iced view
    fn extract_primitives<Message>(
        &mut self,
        view: &Element<Message>,
    ) -> Result<Vec<Primitive>>
    where
        Message: Clone + std::fmt::Debug + 'static,
    {
        // Create iced Renderer
        let mut renderer = IcedRenderer::new(
            iced::Renderer::Wgpu(self.backend.clone()),
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
        );

        // Create widget tree
        let mut tree = Tree::new(view);

        // Layout the view
        let layout = Layout::new(view);

        // Sync widget state
        tree.diff(view);

        // Draw the view to extract primitives
        // This is where we would call the actual render method
        // For now, we return placeholder primitives
        
        warn!("Step 2: Primitive extraction stub - need to implement actual render() call");
        warn!("Next: Call renderer.render() to get primitives vector");

        Ok(vec![
            Primitive::Clear(Color::from_rgb(0.15, 0.15, 0.2)),
        ])
    }

    /// STEP 2: Render extracted primitives
    fn render_primitives(
        &mut self,
        primitives: &[Primitive],
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()> {
        // Begin render pass
        let clear_color = wgpu::Color {
            r: 0.15,
            g: 0.15,
            b: 0.2,
            a: 1.0,
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Primitive Render Pass"),
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

        // STEP 2: Render each primitive
        for primitive in primitives {
            match primitive {
                Primitive::Clear(color) => {
                    // Already cleared above
                    debug!("Clear primitive: {:?}", color);
                }
                Primitive::Quad { bounds, color, border_radius } => {
                    // Would render solid quad using backend
                    debug!("Quad primitive at {:?}", bounds);
                }
                Primitive::Clip { bounds, content } => {
                    // Would set scissor rect and render content
                    debug!("Clip primitive at {:?}", bounds);
                }
                Primitive::Text { content, position, color, size } => {
                    // Would render text using glyphon
                    debug!("Text primitive: '{}' at {:?}", content, position);
                }
                Primitive::Image { handle, bounds } => {
                    // Would render image
                    debug!("Image primitive at {:?}", bounds);
                }
            }
        }

        drop(render_pass);

        warn!("Step 2: Primitive rendering stub - need to implement actual backend draw calls");
        warn!("Next steps:");
        warn!("  - Step 3: Add text rendering with glyphon");
        warn!("  - Step 4: Encode backend draw commands for each primitive");

        Ok(())
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}

/// Represents a renderable primitive
#[derive(Debug, Clone)]
pub enum Primitive {
    /// Clear to color
    Clear(Color),
    /// Solid color quad
    Quad {
        bounds: iced::Rectangle,
        color: Color,
        border_radius: [f32; 4],
    },
    /// Clip region with nested content
    Clip {
        bounds: iced::Rectangle,
        content: Box<Primitive>,
    },
    /// Text
    Text {
        content: String,
        position: iced::Point,
        color: Color,
        size: f32,
    },
    /// Image
    Image {
        handle: iced::advanced::image::Handle,
        bounds: iced::Rectangle,
    },
}

/// Message type for iced rendering
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ButtonPressed(String),
    ToggleAnimation,
}
