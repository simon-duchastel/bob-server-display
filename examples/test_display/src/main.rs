//! Simple test display program
//! 
//! This example displays "TEST TEST TEST" repeatedly across the full width of the screen
//! to verify the display is working correctly.

use anyhow::Result;
use bob_display_core::{Config, Display, Renderer};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info};

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
    let display = Display::new(&config)?;
    let width = display.width();
    let height = display.height();
    info!("Display initialized: {}x{}", width, height);

    // Set up signal handlers
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    // Create a custom renderer for the test pattern
    let mut renderer = Renderer::new(width, height, &config)?;
    
    info!("Entering main display loop - printing TEST TEST TEST");
    
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
                if let Err(e) = render_test_pattern(&mut renderer, width, height) {
                    error!("Render error: {}", e);
                }
            }
        }
    }

    info!("Shutting down test display");
    Ok(())
}

/// Renders "TEST TEST TEST" repeatedly across the screen
fn render_test_pattern(renderer: &mut Renderer, width: u32, height: u32) -> Result<()> {
    // Clear with black background
    renderer.clear([0, 0, 0, 255]);
    
    // Test text to display
    const TEST_TEXT: &str = "TEST TEST TEST ";
    const FONT_SIZE: u32 = 32;
    const LINE_HEIGHT: i32 = 40;
    const TEXT_COLOR: [u8; 4] = [0, 255, 0, 255]; // Green
    
    // Calculate how many times "TEST TEST TEST" fits across the screen
    let char_width = (FONT_SIZE as f32 * 0.6) as i32; // Approximate character width
    let text_width = TEST_TEXT.len() as i32 * char_width;
    
    // Fill the screen with repeated "TEST TEST TEST" lines
    let mut y = 10;
    while y < height as i32 - LINE_HEIGHT {
        let mut x = 10;
        
        // Keep printing the text until we reach the edge of the screen
        while x < width as i32 - text_width {
            renderer.draw_text(TEST_TEXT, x, y, FONT_SIZE, TEXT_COLOR);
            x += text_width;
        }
        
        y += LINE_HEIGHT;
    }
    
    // Add a border around the screen
    renderer.draw_rect(0, 0, width, 5, [255, 0, 0, 255]); // Top border (red)
    renderer.draw_rect(0, height as i32 - 5, width, 5, [255, 0, 0, 255]); // Bottom border (red)
    renderer.draw_rect(0, 0, 5, height, [255, 0, 0, 255]); // Left border (red)
    renderer.draw_rect(width as i32 - 5, 0, 5, height, [255, 0, 0, 255]); // Right border (red)
    
    Ok(())
}