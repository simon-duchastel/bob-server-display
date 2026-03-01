//! Simple test display program
//! 
//! This example displays "TEST TEST TEST" repeatedly across the full width of the screen
//! to verify the display is working correctly.

use anyhow::Result;
use bob_display_core::{Config, Display, Renderer};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "test_display=info".to_string()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Starting test display");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize display
    let mut display = Display::new(&config)?;
    let width = display.width();
    let height = display.height();
    info!("Display initialized: {}x{}", width, height);

    // Set up signal handlers
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    // Create renderer
    let mut renderer = Renderer::new(width, height, &config)?;
    
    info!("Entering main display loop");
    
    loop {
        tokio::select! {
            // Handle SIGTERM (systemd stop)
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down gracefully");
                break;
            }

            // Handle SIGINT (Ctrl+C)
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down gracefully");
                break;
            }

            // Render frame
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(16)) => {
                // Clear and render to renderer buffer
                if let Err(e) = renderer.render() {
                    error!("Render error: {}", e);
                }
                
                // Copy renderer buffer to display accounting for stride
                if let Err(e) = display.render_frame(|display_buffer, display_width, display_height| {
                    let renderer_buffer = renderer.buffer();
                    let stride = display_buffer.len() / display_height as usize;
                    let bytes_per_pixel = 4;
                    
                    info!("Copying renderer to display: {}x{}, stride={}, renderer_bytes={}", 
                          display_width, display_height, stride, renderer_buffer.len());
                    
                    // Copy row by row to handle stride padding
                    for y in 0..display_height as usize {
                        let display_row_start = y * stride;
                        let renderer_row_start = y * (display_width as usize * bytes_per_pixel);
                        
                        let display_row_end = display_row_start + (display_width as usize * bytes_per_pixel);
                        let renderer_row_end = renderer_row_start + (display_width as usize * bytes_per_pixel);
                        
                        if display_row_end <= display_buffer.len() && renderer_row_end <= renderer_buffer.len() {
                            display_buffer[display_row_start..display_row_end]
                                .copy_from_slice(&renderer_buffer[renderer_row_start..renderer_row_end]);
                        } else {
                            error!("Row {} out of bounds! display_end={}, display_len={}, renderer_end={}, renderer_len={}", 
                                  y, display_row_end, display_buffer.len(), renderer_row_end, renderer_buffer.len());
                        }
                    }
                }) {
                    error!("Display render error: {}", e);
                }
            }
        }
    }

    info!("Shutting down test display");
    Ok(())
}
