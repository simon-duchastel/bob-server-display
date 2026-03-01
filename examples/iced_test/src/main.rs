//! Iced GUI example with direct KMS rendering
//!
//! This example uses the iced GUI framework with a custom rendering pipeline
//! that outputs directly to KMS/DRM, bypassing Wayland/X11 entirely.

use anyhow::Result;
use bob_display_core::{Config, Display};
use iced::{Color, Task};
use std::collections::HashMap;
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
    info!("Using iced GUI framework with direct KMS/DRM output");

    // Load configuration and initialize KMS display
    let config = Config::load()?;
    let display = Display::new(&config)?;
    let width = display.width();
    let height = display.height();

    info!("Display initialized: {}x{}", width, height);
    info!("Starting iced application with KMS backend");

    // Run the iced application
    run_iced_kms(display, width, height)
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

/// Run the iced application with KMS backend
fn run_iced_kms(mut display: Display, width: u32, height: u32) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    let mut app = IcedApp::new(width, height);

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

        // Get background color
        let bg_color = hsl_to_rgb(app.hue, 0.8, 0.5);
        let bg_b = (bg_color.b * 255.0) as u8;
        let bg_g = (bg_color.g * 255.0) as u8;
        let bg_r = (bg_color.r * 255.0) as u8;
        
        // Counter text to display
        let counter_text = format!("Count: {}", app.counter);
        
        // Write directly to KMS buffer in BGRX format
        if let Err(e) = display.render_frame(|kms_buffer, display_width, display_height| {
            let stride = kms_buffer.len() / display_height as usize;
            
            // Fill background
            for y in 0..display_height as usize {
                let row_start = y * stride;
                for x in 0..display_width as usize {
                    let idx = row_start + (x * 4);
                    if idx + 3 < kms_buffer.len() {
                        kms_buffer[idx] = bg_b;
                        kms_buffer[idx + 1] = bg_g;
                        kms_buffer[idx + 2] = bg_r;
                        kms_buffer[idx + 3] = 0xFF;
                    }
                }
            }
            
            // Draw cool rectangles
            let rect_colors = [
                (0xFF, 0x00, 0x00), // Red
                (0x00, 0xFF, 0x00), // Green
                (0x00, 0x00, 0xFF), // Blue
                (0xFF, 0xFF, 0x00), // Yellow
            ];
            
            for (i, (r, g, b)) in rect_colors.iter().enumerate() {
                let rect_x = 50 + i * 120;
                let rect_y = 50;
                let rect_w = 100;
                let rect_h = 60;
                
                for ry in rect_y..(rect_y + rect_h) {
                    if ry >= display_height as usize { break; }
                    let row_start = ry * stride;
                    for rx in rect_x..(rect_x + rect_w) {
                        if rx >= display_width as usize { break; }
                        let idx = row_start + (rx * 4);
                        if idx + 3 < kms_buffer.len() {
                            kms_buffer[idx] = *b;
                            kms_buffer[idx + 1] = *g;
                            kms_buffer[idx + 2] = *r;
                            kms_buffer[idx + 3] = 0xFF;
                        }
                    }
                }
            }
            
            // Draw counter button
            let btn_x = 50;
            let btn_y = 150;
            let btn_w = 200;
            let btn_h = 50;
            
            // Button background (orange)
            for by in btn_y..(btn_y + btn_h) {
                if by >= display_height as usize { break; }
                let row_start = by * stride;
                for bx in btn_x..(btn_x + btn_w) {
                    if bx >= display_width as usize { break; }
                    let idx = row_start + (bx * 4);
                    if idx + 3 < kms_buffer.len() {
                        kms_buffer[idx] = 0x00;     // B
                        kms_buffer[idx + 1] = 0x80; // G
                        kms_buffer[idx + 2] = 0xFF; // R (orange-ish)
                        kms_buffer[idx + 3] = 0xFF;
                    }
                }
            }
            
            // Draw simple text (just white pixels for now)
            draw_text(kms_buffer, stride, display_width, display_height, &counter_text, 70, 165, 0xFF, 0xFF, 0xFF);
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

/// Simple text drawing function (8x8 pixel characters)
fn draw_text(buffer: &mut [u8], stride: usize, width: u32, height: u32, text: &str, x: usize, y: usize, r: u8, g: u8, b: u8) {
    // Very simple 3x5 font for digits and basic letters
    let chars: std::collections::HashMap<char, [u8; 15]> = [
        ('0', [1,1,1, 1,0,1, 1,0,1, 1,0,1, 1,1,1]),
        ('1', [0,1,0, 0,1,0, 0,1,0, 0,1,0, 0,1,0]),
        ('2', [1,1,1, 0,0,1, 1,1,1, 1,0,0, 1,1,1]),
        ('3', [1,1,1, 0,0,1, 1,1,1, 0,0,1, 1,1,1]),
        ('4', [1,0,1, 1,0,1, 1,1,1, 0,0,1, 0,0,1]),
        ('5', [1,1,1, 1,0,0, 1,1,1, 0,0,1, 1,1,1]),
        ('6', [1,1,1, 1,0,0, 1,1,1, 1,0,1, 1,1,1]),
        ('7', [1,1,1, 0,0,1, 0,0,1, 0,0,1, 0,0,1]),
        ('8', [1,1,1, 1,0,1, 1,1,1, 1,0,1, 1,1,1]),
        ('9', [1,1,1, 1,0,1, 1,1,1, 0,0,1, 1,1,1]),
        ('C', [1,1,1, 1,0,0, 1,0,0, 1,0,0, 1,1,1]),
        ('o', [0,0,0, 1,1,1, 1,0,1, 1,0,1, 1,1,1]),
        ('u', [0,0,0, 1,0,1, 1,0,1, 1,0,1, 1,1,1]),
        ('n', [0,0,0, 1,1,1, 1,0,1, 1,0,1, 1,0,1]),
        ('t', [0,1,0, 1,1,1, 0,1,0, 0,1,0, 0,1,0]),
        (' ', [0,0,0, 0,0,0, 0,0,0, 0,0,0, 0,0,0]),
        (':', [0,0,0, 0,1,0, 0,0,0, 0,1,0, 0,0,0]),
    ].iter().cloned().collect();
    
    let scale = 3; // 3x scale
    let mut cursor_x = x;
    
    for ch in text.chars() {
        if let Some(bitmap) = chars.get(&ch) {
            for row in 0..5 {
                for col in 0..3 {
                    if bitmap[row * 3 + col] == 1 {
                        // Draw scaled pixel
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let px = cursor_x + col * scale + sx;
                                let py = y + row * scale + sy;
                                if px < width as usize && py < height as usize {
                                    let idx = py * stride + px * 4;
                                    if idx + 3 < buffer.len() {
                                        buffer[idx] = b;
                                        buffer[idx + 1] = g;
                                        buffer[idx + 2] = r;
                                        buffer[idx + 3] = 0xFF;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            cursor_x += 4 * scale; // character width + spacing
        } else {
            cursor_x += 3 * scale; // space for unknown chars
        }
    }
}
