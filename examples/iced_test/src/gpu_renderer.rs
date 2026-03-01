//! GPU-accelerated renderer using wgpu - Step 3: Text Rendering
//!
//! This step adds text rendering with glyphon:
//! - Initialize glyphon text atlas and renderer
//! - Rasterize text glyphs to atlas
//! - Render text primitives using cached glyphs

use anyhow::{anyhow, Context, Result};
use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics, Resolution,
    Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer,
};
use iced::advanced::graphics::color;
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::{self, Layout, Widget};
use iced::{Color, Element, Length, Renderer as IcedRenderer, Size, Theme, Vector};
use iced_wgpu::graphics::Viewport;
use iced_wgpu::wgpu;
use iced_wgpu::{self, Backend, Engine, Settings};
use tracing::{debug, info, warn};

/// GPU renderer with text rendering support
pub struct GpuRenderer {
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    readback_buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    engine: Engine,
    backend: Backend,
    viewport: Viewport,
    scale_factor: f64,
    // STEP 3: Text rendering components
    /// Font system for loading and managing fonts
    font_system: FontSystem,
    /// Swash cache for glyph rasterization
    swash_cache: SwashCache,
    /// Text atlas for cached glyphs
    text_atlas: TextAtlas,
    /// Text renderer for drawing text
    text_renderer: TextRenderer,
}

impl GpuRenderer {
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        info!("Step 3: Initializing text rendering: {}x{}", width, height);

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
        let viewport = Viewport::with_physical_size(iced::Size::new(width, height), scale_factor);

        // STEP 3: Initialize glyphon text rendering
        info!("Initializing glyphon text rendering system");

        // Create font system with default fonts
        let font_system = FontSystem::new();

        // Create swash cache for glyph rasterization
        let swash_cache = SwashCache::new();

        // Create text atlas for caching rasterized glyphs
        let text_atlas = TextAtlas::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb);

        // Create text renderer
        let text_renderer = TextRenderer::new(
            &mut text_atlas,
            &device,
            wgpu::MultisampleState::default(),
            None, // No depth testing for UI
        );

        info!("Step 3 complete: Text rendering system initialized");

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
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
        })
    }

    pub fn render_frame(&mut self, view: &Element<Message>, width: u32, height: u32) -> Result<Vec<u8>>
    where
        Message: Clone + std::fmt::Debug + 'static,
    {
        let primitives = self.extract_primitives(view)?;
        debug!("Rendering {} primitives", primitives.len());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.render_primitives(&primitives, &mut encoder)?;

        // Copy texture to readback buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.readback_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Read back buffer
        let buffer_slice = self.readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().context("Failed to map buffer")??;

        // Convert to BGRX
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

    fn extract_primitives<Message>(
        &mut self,
        view: &Element<Message>,
    ) -> Result<Vec<Primitive>>
    where
        Message: Clone + std::fmt::Debug + 'static,
    {
        // TODO: Implement actual primitive extraction
        Ok(vec![Primitive::Clear(Color::from_rgb(0.15, 0.15, 0.2))])
    }

    fn render_primitives(
        &mut self,
        primitives: &[Primitive],
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()> {
        // Prepare text rendering
        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                Resolution {
                    width: self.width,
                    height: self.height,
                },
                vec![], // Text areas - would be populated from Text primitives
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow!("Text preparation failed: {:?}", e))?;

        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Text Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.15,
                        g: 0.15,
                        b: 0.2,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // STEP 3: Render text primitives
        for primitive in primitives {
            match primitive {
                Primitive::Text { content, position, color, size } => {
                    // Would create TextArea and render via glyphon
                    debug!("Rendering text: '{}' at {:?}", content, position);
                }
                _ => {}
            }
        }

        // Render text atlas
        self.text_renderer
            .render(&self.text_atlas, &mut render_pass)
            .map_err(|e| anyhow!("Text render failed: {:?}", e))?;

        drop(render_pass);

        warn!("Step 3: Text rendering initialized but not yet integrated with primitive extraction");
        warn!("Next step: Connect Text primitives to actual text area rendering");

        Ok(())
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Clear(Color),
    Quad {
        bounds: iced::Rectangle,
        color: Color,
        border_radius: [f32; 4],
    },
    Clip {
        bounds: iced::Rectangle,
        content: Box<Primitive>,
    },
    Text {
        content: String,
        position: iced::Point,
        color: Color,
        size: f32,
    },
    Image {
        handle: iced::advanced::image::Handle,
        bounds: iced::Rectangle,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ButtonPressed(String),
    ToggleAnimation,
}
