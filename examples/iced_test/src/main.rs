use anyhow::Result;
use bob_display_core::{Config, Display};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info};
use tracing_subscriber;

fn draw_color_gradient(buffer: &mut [u8], width: u32, height: u32, frame: u64) {
    let bytes_per_pixel = 4;
    
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * bytes_per_pixel) as usize;
            
            let r = ((x as u32 * 255) / width) as u8;
            let g = ((y as u32 * 255) / height) as u8;
            let b = (frame % 255) as u8;
            
            if idx + 3 < buffer.len() {
                buffer[idx] = b;
                buffer[idx + 1] = g;
                buffer[idx + 2] = r;
                buffer[idx + 3] = 255;
            }
        }
    }
}

fn draw_test_pattern(buffer: &mut [u8], width: u32, height: u32) {
    let bytes_per_pixel = 4;
    let padding = 50;
    
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * bytes_per_pixel) as usize;
            
            let (r, g, b) = if x < padding || y < padding || x > width - padding || y > height - padding {
                (255, 255, 255)
            } else if x < width / 3 {
                (255, 0, 0)
            } else if x < 2 * width / 3 {
                (0, 255, 0)
            } else {
                (0, 0, 255)
            };
            
            if idx + 3 < buffer.len() {
                buffer[idx] = b;
                buffer[idx + 1] = g;
                buffer[idx + 2] = r;
                buffer[idx + 3] = 255;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "iced_test=info".to_string()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Starting iced test with direct KMS");

    let config = Config::load()?;
    info!("Configuration loaded");

    let mut display = Display::new(&config)?;
    let width = display.width();
    let height = display.height();
    info!("Display initialized: {}x{}", width, height);

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    
    let mut frame_count = 0u64;

    info!("Entering main loop");
    
    loop {
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down");
                break;
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(16)) => {
                if let Err(e) = display.render_frame(|buffer, w, h| {
                    draw_color_gradient(buffer, w, h, frame_count);
                    if frame_count % 120 == 0 {
                        draw_test_pattern(buffer, w, h);
                    }
                }) {
                    error!("Render error: {}", e);
                }
                frame_count = frame_count.wrapping_add(1);
            }
        }
    }

    info!("Shutting down");
    Ok(())
}
