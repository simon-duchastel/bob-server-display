use anyhow::{anyhow, Context, Result};
use drm::control::crtc;
use drm::control::framebuffer;
use drm::control::Device as ControlDevice;
use drm::Device;
use gbm::BufferObjectFlags;
use std::fs::File;
use std::os::unix::io::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::fs::OpenOptionsExt;
use tracing::info;

use crate::config::Config;
use crate::render::Renderer;

/// Wrapper struct around File that implements the DRM Device traits
#[derive(Debug)]
pub struct DrmDevice(File);

impl AsFd for DrmDevice {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for DrmDevice {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}

impl Device for DrmDevice {}
impl ControlDevice for DrmDevice {}

pub struct Display {
    gbm_device: gbm::Device<DrmDevice>,
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
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        options.write(true);
        options.custom_flags(libc::O_CLOEXEC);
        let file = options.open(&config.drm_device)
            .with_context(|| format!("Failed to open DRM device: {}", config.drm_device))?;

        let drm_device = DrmDevice(file);
        info!("Opened DRM device: {}", config.drm_device);

        // Create GBM device - this will implement DRM Device traits too
        let gbm_device = gbm::Device::new(drm_device)
            .map_err(|_| anyhow!("Failed to create GBM device"))?;

        // Get display resources using the ControlDevice trait (now through GBM device)
        let resources = gbm_device
            .resource_handles()
            .context("Failed to get DRM resources")?;

        // Find a connected connector
        let connector = resources
            .connectors()
            .iter()
            .find_map(|&conn| {
                let info = gbm_device.get_connector(conn, true).ok()?;
                if info.state() == drm::control::connector::State::Connected {
                    Some(conn)
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("No connected display found"))?;

        let connector_info = gbm_device
            .get_connector(connector, true)
            .context("Failed to get connector info")?;

        info!(
            "Found connected display: {:?}",
            connector_info.interface()
        );

        // Get the preferred mode or use the first available mode
        let mode = connector_info
            .modes()
            .first()
            .copied()
            .ok_or_else(|| anyhow!("No display modes available"))?;

        let (width, height) = mode.size();
        let width = width as u32;
        let height = height as u32;
        info!("Selected mode: {}x{} @ {}Hz", width, height, mode.vrefresh());

        // Find a suitable CRTC for this connector
        // First, try to find the CRTC that's currently driving this connector
        let mut selected_crtc = None;
        
        // Check which encoders are possible for this connector
        for &encoder_id in connector_info.encoders() {
            if let Ok(encoder_info) = gbm_device.get_encoder(encoder_id) {
                // If encoder has a CRTC, check if it's in our resources
                if let Some(crtc_id) = encoder_info.crtc() {
                    if resources.crtcs().contains(&crtc_id) {
                        selected_crtc = Some(crtc_id);
                        info!("Found CRTC {:?} via encoder {:?}", crtc_id, encoder_id);
                        break;
                    }
                }
            }
        }
        
        // If no CRTC found via encoder, try to find an unused one
        if selected_crtc.is_none() {
            selected_crtc = resources
                .crtcs()
                .iter()
                .find(|&&crtc| {
                    gbm_device
                        .get_crtc(crtc)
                        .map(|info: drm::control::crtc::Info| info.mode().is_none())
                        .unwrap_or(false)
                })
                .copied();
        }
        
        // If still no CRTC, just use the first one available
        let crtc = selected_crtc
            .or_else(|| resources.crtcs().first().copied())
            .ok_or_else(|| anyhow!("No CRTC found"))?;

        info!("Using CRTC: {:?}", crtc);

        let mut display = Self {
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

        // Create DRM framebuffer from GBM buffer using GBM's helper method
        let fb = self.gbm_device
            .add_framebuffer(&buffer, 24, 32)
            .map_err(|_| anyhow!("Failed to create framebuffer"))?;

        self.framebuffer = Some(fb);

        // Set the CRTC to display the framebuffer with the selected mode
        self.gbm_device
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
            let _ = self.gbm_device.destroy_framebuffer(fb);
        }
        info!("Display resources cleaned up");
    }
}