use anyhow::Result;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info, warn};

mod config;
mod display;
mod render;

use config::Config;
use display::Display;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "bob_server_display=info".to_string()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Starting bob-server-display v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize display
    let mut display = Display::new(&config)?;
    info!("Display initialized: {}x{}", display.width(), display.height());

    // Set up signal handlers
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    // Main display loop
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
                if let Err(e) = display.render_frame() {
                    error!("Render error: {}", e);
                    // Continue running even on render errors
                }
            }
        }
    }

    info!("Shutting down bob-server-display");
    Ok(())
}