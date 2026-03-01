//! Iced GUI example with direct KMS rendering using bob_display_core Renderer
//!
//! This example uses the iced application model with bob_display_core's Renderer
//! for double-buffered, tear-free rendering to KMS/DRM.

use anyhow::Result;
use bob_display_core::{Config, Display, Renderer};
use iced::{Color, Task};
use std::time::Instant;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "iced_test=info".to_string()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Starting Iced KMS Direct Rendering Example");
    info!("Using bob_display_core Renderer for tear-free output");

    // Load configuration and initialize KMS display
    let config = Config::load()?;
    let display = Display::new(&config)?;
    let width = display.width();
    let height = display.height();

    info!("Display initialized: {}x{}", width, height);
    info!("Starting iced application with KMS backend");

    // Run the iced application
    run_iced_kms(display, width, height, &config)
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ButtonPressed(String),
    ToggleAnimation,
}

/// The iced application state
pub struct IcedApp {
    frame_count: u64,
    start_time: Instant,
    counter: u32,
    animation_enabled: bool,
    hue: f32,
    width: u32,
    height: u32,
}

impl IcedApp {
    fn new(width: u32, height: u32) -> Self {
        Self {
            frame_count: 0,
            start_time: Instant::now(),
            counter: 0,
            animation_enabled: true,
            hue: 0.0,
            width,
            height,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.frame_count += 1;
                if self.animation_enabled {
                    self.hue = (self.hue + 1.0) % 360.0;
                }
            }
            Message::ButtonPressed(label) => {
                info!("Button pressed: {}", label);
                if label == "Counter" {
                    self.counter += 1;
                }
            }
            Message::ToggleAnimation => {
                self.animation_enabled = !self.animation_enabled;
                info!("Animation {}", if self.animation_enabled { "enabled" } else { "disabled" });
            }
        }
        Task::none()
    }
}

/// Run the iced application with KMS backend using Renderer
fn run_iced_kms(mut display: Display, width: u32, height: u32, config: &Config) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    let mut app = IcedApp::new(width, height);
    
    // Create renderer for double-buffered drawing
    let mut renderer = Renderer::new(width, height, config)?;

    info!("Entering main render loop");
    let start_time = Instant::now();
    let mut frame_count = 0u64;

    rt.block_on(async {
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sigint = signal(SignalKind::interrupt())?;

        loop {
            // Check for shutdown signals with timeout
            tokio::select! {
                _ = sigterm.recv() => {
                    info!("SIGTERM received, shutting down");
                    break;
                }
                _ = sigint.recv() => {
                    info!("SIGINT received, shutting down");
                    break;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(16)) => {
                    // Continue with frame rendering
                }
            }

            // Update application state
            let _ = app.update(Message::Tick);

            // Get background color (bright, animated)
            let bg_color = hsl_to_rgb(app.hue, 0.8, 0.5);
            let bg_rgba = [
                (bg_color.r * 255.0) as u8,
                (bg_color.g * 255.0) as u8,
                (bg_color.b * 255.0) as u8,
                255,
            ];

            // Clear renderer buffer with animated background
            renderer.clear(bg_rgba);

            // Draw cool colored rectangles
            let rect_colors = [
                [255, 0, 0, 255],     // Red
                [0, 255, 0, 255],     // Green
                [0, 0, 255, 255],     // Blue
                [255, 255, 0, 255],   // Yellow
            ];

            for (i, color) in rect_colors.iter().enumerate() {
                let x = 50 + (i as i32) * 120;
                let y = 50;
                renderer.draw_rect(x, y, 100, 60, *color);
            }

            // Draw counter button (orange background)
            let btn_color = [255, 128, 0, 255]; // Orange
            renderer.draw_rect(50, 150, 200, 50, btn_color);

            // Draw counter text on button
            let counter_text = format!("Count: {}", app.counter);
            renderer.draw_text(&counter_text, 70, 165, 24, [255, 255, 255, 255]);

            // Copy renderer buffer to display
            if let Err(e) = display.render_frame(|kms_buffer, display_width, display_height| {
                let renderer_buffer = renderer.buffer();
                let stride = kms_buffer.len() / display_height as usize;
                let bytes_per_pixel = 4;

                // Copy row by row to handle stride padding
                for y in 0..display_height as usize {
                    let display_row_start = y * stride;
                    let renderer_row_start = y * (display_width as usize * bytes_per_pixel);

                    let display_row_end = display_row_start + (display_width as usize * bytes_per_pixel);
                    let renderer_row_end = renderer_row_start + (display_width as usize * bytes_per_pixel);

                    if display_row_end <= kms_buffer.len() && renderer_row_end <= renderer_buffer.len() {
                        kms_buffer[display_row_start..display_row_end]
                            .copy_from_slice(&renderer_buffer[renderer_row_start..renderer_row_end]);
                    }
                }
            }) {
                error!("Failed to present frame: {}", e);
            }

            frame_count += 1;

            // Log FPS every 60 frames
            if frame_count % 60 == 0 {
                let elapsed = start_time.elapsed().as_secs_f32();
                let fps = frame_count as f32 / elapsed;
                info!("FPS: {:.1}", fps);
            }
        }

        info!("Shutting down after {} frames", frame_count);
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

/// Convert HSL to RGB
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::from_rgb(r + m, g + m, b + m)
}
