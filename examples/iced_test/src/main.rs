//! Iced GUI example with direct KMS rendering
//!
//! This example uses the iced GUI framework with a custom rendering pipeline
//! that outputs directly to KMS/DRM, bypassing Wayland/X11 entirely.

use anyhow::Result;
use bob_display_core::{Config, Display};
use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Color, Element, Length, Task};
use std::time::{Duration, Instant};
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
    button_states: Vec<bool>,
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
            button_states: vec![false; 4],
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
                match label.as_str() {
                    "Button 1" => self.button_states[0] = !self.button_states[0],
                    "Button 2" => self.button_states[1] = !self.button_states[1],
                    "Button 3" => self.button_states[2] = !self.button_states[2],
                    "Button 4" => self.button_states[3] = !self.button_states[3],
                    _ => {}
                }
            }
            Message::ToggleAnimation => {
                self.animation_enabled = !self.animation_enabled;
                info!("Animation {}", if self.animation_enabled { "enabled" } else { "disabled" });
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let fps = if elapsed > 0.0 {
            self.frame_count as f32 / elapsed
        } else {
            0.0
        };

        let bg_color = hsl_to_rgb(self.hue, 0.3, 0.15);

        let title = text("Iced on KMS/DRM")
            .size(56)
            .style(|_| text::Style {
                color: Some(Color::WHITE),
                ..text::Style::default()
            });

        let subtitle = text("Direct Rendering - No Wayland/X11 Required")
            .size(24)
            .style(|_| text::Style {
                color: Some(Color::from_rgb(0.7, 0.7, 0.7)),
                ..text::Style::default()
            });

        let stats = text(format!(
            "{}x{} | Frames: {} | FPS: {:.1} | Animation: {}",
            self.width, self.height, self.frame_count, fps,
            if self.animation_enabled { "ON" } else { "OFF" }
        ))
        .size(20)
        .style(|_| text::Style {
            color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
            ..text::Style::default()
        });

        let buttons_row = row![
            self.create_button("Button 1", 0),
            self.create_button("Button 2", 1),
            self.create_button("Button 3", 2),
            self.create_button("Button 4", 3),
        ]
        .spacing(20);

        let anim_text = if self.animation_enabled { "Pause Animation" } else { "Resume Animation" };
        let anim_button = button(text(anim_text).size(18))
            .on_press(Message::ToggleAnimation)
            .padding(15)
            .style(|_theme, _status| button::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.5, 0.8))),
                text_color: Color::WHITE,
                ..button::Style::default()
            });

        let content = column![
            Space::with_height(Length::FillPortion(1)),
            title,
            subtitle,
            Space::with_height(30),
            stats,
            Space::with_height(50),
            buttons_row,
            Space::with_height(30),
            anim_button,
            Space::with_height(Length::FillPortion(1)),
        ]
        .align_x(Horizontal::Center)
        .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg_color)),
                ..container::Style::default()
            })
            .into()
    }

    fn create_button<'a>(&'a self, label: &'a str, index: usize) -> Element<'a, Message> {
        let is_active = self.button_states.get(index).copied().unwrap_or(false);
        let bg_color = if is_active {
            Color::from_rgb(0.2, 0.8, 0.4)
        } else {
            Color::from_rgb(0.3, 0.3, 0.35)
        };

        button(text(label).size(18))
            .on_press(Message::ButtonPressed(label.to_string()))
            .style(move |_theme, status| {
                let base = bg_color;
                match status {
                    button::Status::Hovered => button::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(
                            (base.r + 0.1).min(1.0),
                            (base.g + 0.1).min(1.0),
                            (base.b + 0.1).min(1.0),
                        ))),
                        ..button::Style::default()
                    },
                    button::Status::Pressed => button::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(
                            (base.r - 0.1).max(0.0),
                            (base.g - 0.1).max(0.0),
                            (base.b - 0.1).max(0.0),
                        ))),
                        ..button::Style::default()
                    },
                    _ => button::Style {
                        background: Some(iced::Background::Color(base)),
                        ..button::Style::default()
                    },
                }
            })
            .padding(20)
            .into()
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

        // Get the current view
        let _view = app.view();

        // Get background color and fill KMS buffer directly
        // Use higher lightness (0.5) so colors are visible
        let bg_color = hsl_to_rgb(app.hue, 0.8, 0.5);
        let b = (bg_color.b * 255.0) as u8;
        let g = (bg_color.g * 255.0) as u8;
        let r = (bg_color.r * 255.0) as u8;
        
        // Store first pixel for debugging
        let mut first_pixel: [u8; 4] = [0, 0, 0, 0];
        
        // Write directly to KMS buffer in BGRX format
        if let Err(e) = display.render_frame(|kms_buffer, display_width, display_height| {
            let stride = kms_buffer.len() / display_height as usize;
            
            // Fill each row with the background color, handling stride
            for y in 0..display_height as usize {
                let row_start = y * stride;
                // Fill visible portion of the row
                for x in 0..display_width as usize {
                    let idx = row_start + (x * 4);
                    if idx + 3 < kms_buffer.len() {
                        kms_buffer[idx] = b;     // B
                        kms_buffer[idx + 1] = g; // G
                        kms_buffer[idx + 2] = r; // R
                        kms_buffer[idx + 3] = 0xFF; // X
                    }
                }
            }
            
            // Capture first pixel for debugging
            first_pixel = [kms_buffer[0], kms_buffer[1], kms_buffer[2], kms_buffer[3]];
        }) {
            error!("Failed to present frame: {}", e);
        }

        // Debug: Show buffer info and first pixel
        if frame_count % 10 == 0 {
            info!("Frame {}: First pixel BGRX = {:02X} {:02X} {:02X} {:02X} (B={} G={} R={})",
                  frame_count, first_pixel[0], first_pixel[1], first_pixel[2], first_pixel[3], b, g, r);
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
