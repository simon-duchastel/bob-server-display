//! Simple Iced kiosk example using standard windowing
//! 
//! This uses Iced's built-in wgpu backend for GPU-accelerated rendering
//! Run with: cargo run --example iced_test --release

use iced::{Element, Task, Theme, window};
use iced::widget::{column, row, button, text, container};
use iced::Length;
use std::time::Instant;

fn main() -> iced::Result {
    iced::application("Bob Server Display", BobDisplay::update, BobDisplay::view)
        .theme(|_| Theme::Dark)
        .window(window::Settings {
            size: iced::Size::new(1920.0, 1080.0),
            ..window::Settings::default()
        })
        .run_with(BobDisplay::new)
}

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            window::get_latest().and_then(|id| {
                Task::batch([
                    window::change_mode(id, window::Mode::Fullscreen),
                    window::change_cursor(id, iced::mouse::Cursor::None),
                ])
            }),
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ButtonPressed(String),
}

struct BobDisplay {
    start_time: Instant,
    frame_count: u64,
    counter: u32,
}

impl Default for BobDisplay {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            frame_count: 0,
            counter: 0,
        }
    }
}

impl BobDisplay {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.frame_count += 1;
            }
            Message::ButtonPressed(label) => {
                if label == "Increment" {
                    self.counter += 1;
                }
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

        let content = column![
            row![
                text("Bob Server Display").size(40),
            ]
            .spacing(20),
            
            row![
                text(format!("FPS: {:.1}", fps)).size(24),
                text(format!("Frames: {}", self.frame_count)).size(24),
            ]
            .spacing(20),
            
            row![
                text(format!("Counter: {}", self.counter)).size(30),
                button("Increment").on_press(Message::ButtonPressed("Increment".to_string())),
            ]
            .spacing(20),
            
            text("Running in fullscreen GPU-accelerated mode").size(16),
        ]
        .spacing(30)
        .align_x(iced::Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
