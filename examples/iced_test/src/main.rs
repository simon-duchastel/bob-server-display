//! Iced GUI example with direct KMS rendering
//!
//! This example uses the iced GUI framework with a custom rendering pipeline
//! that outputs directly to KMS/DRM, bypassing Wayland/X11 entirely.

use anyhow::{anyhow, Result};
use bob_display_core::{Config, Display};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, text, Space};
use iced::{Color, Element, Length, Subscription, Task, Theme};
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

    fn title(&self) -> String {
        String::from("Iced KMS Direct Rendering")
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

    fn view(&self) -> Element<Message> {
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

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
    }

    fn create_button(&self, label: &str, index: usize) -> Element<Message> {
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
fn run_iced_kms(display: Display, width: u32, height: u32) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let mut sigterm = rt.block_on(async { signal(SignalKind::terminate()) })?;
    let mut sigint = rt.block_on(async { signal(SignalKind::interrupt()) })?;

    let mut app = IcedApp::new(width, height);
    let mut buffer: Vec<u8> = vec![0u8; (width * height * 4) as usize];

    info!("Entering main render loop");
    let start_time = Instant::now();
    let mut frame_count = 0u64;

    loop {
        // Check for shutdown signals
        if rt.block_on(async {
            tokio::select! {
                _ = sigterm.recv() => { info!("SIGTERM received, shutting down"); true }
                _ = sigint.recv() => { info!("SIGINT received, shutting down"); true }
                else => false
            }
        }) {
            break;
        }

        // Update application state
        let _ = app.update(Message::Tick);

        // Get the current view
        let view = app.view();

        // Render the view using tiny-skia
        // Create a pixmap for rendering
        let mut pixmap = match tiny_skia::Pixmap::new(width, height) {
            Some(p) => p,
            None => {
                error!("Failed to create pixmap");
                continue;
            }
        };

        // Get background color
        let bg_color = hsl_to_rgb(app.hue, 0.3, 0.15);
        let skia_bg = tiny_skia::Color::from_rgba(
            bg_color.r,
            bg_color.g,
            bg_color.b,
            1.0,
        ).unwrap_or(tiny_skia::Color::from_rgba8(40, 40, 50, 255));
        pixmap.fill(skia_bg);

        // Convert RGBA to BGRX for KMS
        let skia_data = pixmap.data();
        for (i, pixel) in skia_data.chunks_exact(4).enumerate() {
            let idx = i * 4;
            if idx + 3 < buffer.len() {
                buffer[idx] = pixel[2];     // B
                buffer[idx + 1] = pixel[1]; // G
                buffer[idx + 2] = pixel[0]; // R
                buffer[idx + 3] = 0xFF;     // X
            }
        }

        // Present to KMS display
        if let Err(e) = display.render_frame(|kms_buffer, _w, _h| {
            let len = kms_buffer.len().min(buffer.len());
            kms_buffer[..len].copy_from_slice(&buffer[..len]);
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

        // Frame rate limiting (60 FPS)
        std::thread::sleep(Duration::from_millis(16));
    }

    info!("Shutting down after {} frames", frame_count);
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
