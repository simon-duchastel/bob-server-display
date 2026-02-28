use anyhow::{anyhow, Context, Result};
use drm::buffer::DrmFourcc;
use drm::control::crtc;
use drm::control::framebuffer;
use drm::control::Device;
use gbm::BufferObjectFlags;
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::fs::OpenOptionsExt;
use tracing::info;

use crate::config::Config;
use crate::render::Renderer;

pub struct Display {
    drm_device: File,
    gbm_device: gbm::Device<File>,
    width: u32,
    height: u32,
    renderer: Renderer,
    framebuffer: Option<framebuffer::Handle>,
    crtc: crtc::Handle,
    connector: drm::control::connector::Handle,
}

impl Display {
    pub fn new(config: &Config) -> Result<Self> {
        // Open DRM device
        let drm_device = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_CLOEXEC)
            .open(&config.drm_device)
            .with_context(|| format!("Failed to open DRM device: {}", config.drm_device))?;

        info!("Opened DRM device: {}", config.drm_device);

        // Create GBM device
        let gbm_device = unsafe {
            gbm::Device::new(File::from_raw_fd(drm_device.as_raw_fd()))
                .map_err(|_| anyhow!("Failed to create GBM device"))?
        };

        // Get display resources using the ControlDevice trait
        let resources = drm_device
            .resource_handles()
            .context("Failed to get DRM resources")?;

        // Find a connected connector
        let connector = resources
            .connectors()
            .iter()
            .find_map(|&conn| {
                let info = drm_device.get_connector(conn, true).ok()?;
                if info.state() == drm::control::connector::State::Connected {
                    Some(conn)
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("No connected display found"))?;

        let connector_info = drm_device
            .get_connector(connector, true)
            .context("Failed to get connector info")?;

        info!(
            "Found connected display: {:?}",
            connector_info.interface()
        );

        // Get the preferred mode or use the first available mode
        // If config has a mode, we'll use that size, otherwise use first available
        let mode = connector_info
            .modes()
            .first()
            .copied()
            .ok_or_else(|| anyhow!("No display modes available"))?;

        let (width, height) = mode.size();
        let width = width as u32;
        let height = height as u32;
        info!("Selected mode: {}x{} @ {}Hz", width, height, mode.vrefresh());

        // Find a CRTC that can drive this connector
        let crtc = resources
            .crtcs()
            .iter()
            .find(|&&crtc| {
                drm_device
                    .get_crtc(crtc)
                    .map(|info: drm::control::crtc::Info| info.mode().is_none())
                    .unwrap_or(false)
            })
            .copied()
            .ok_or_else(|| anyhow!("No available CRTC found"))?;

        let mut display = Self {
            drm_device,
            gbm_device,
            width,
            height,
            renderer: Renderer::new(width, height, config)?,
            framebuffer: None,
            crtc,
            connector,
        };

        // Initialize the display
        display.init_framebuffer(mode)?;

        Ok(display)
    }

    fn init_framebuffer(&mut self, mode: drm::control::Mode) -> Result<()> {
        // Create a GBM buffer
        let buffer = self
            .gbm_device
            .create_buffer_object::<()>(
                self.width,
                self.height,
                gbm::Format::Xrgb8888,
                BufferObjectFlags::SCANOUT | BufferObjectFlags::RENDERING,
            )
            .map_err(|_| anyhow!("Failed to create GBM buffer"))?;

        // Get the buffer's file descriptor and stride
        let fd = buffer.fd();
        let stride = buffer.stride()?;
        let handle = buffer.handle()?;
        
        // Create DRM framebuffer from GBM buffer
        // The handle is a union, we need to access the u32 field
        let fb = self.drm_device
            .add_framebuffer(
                unsafe { handle.s32 } as u32,
                self.width,
                self.height,
                stride,
                DrmFourcc::Xrgb8888 as u32,
                32, // bits per pixel for XRGB8888
                0,
            )
            .context("Failed to create framebuffer")?;

        self.framebuffer = Some(fb);

        // Set the CRTC to display the framebuffer with the selected mode
        self.drm_device
            .set_crtc(self.crtc, Some(fb), (0, 0), &[self.connector], Some(mode))
            .context("Failed to set CRTC")?;

        info!("Framebuffer initialized successfully");

        Ok(())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn render_frame(&mut self) -> Result<()> {
        // Render to the framebuffer using our renderer
        self.renderer.render()?;

        // Flip the buffer (if using double buffering - for now we render directly)
        // In a more advanced implementation, we'd use page flipping here

        Ok(())
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        // Cleanup DRM resources
        if let Some(fb) = self.framebuffer {
            let _ = self.drm_device.destroy_framebuffer(fb);
        }
        info!("Display resources cleaned up");
    }
}