//! Bob Display - Kiosk UI with auto-turn-off display functionality
//!
//! Features:
//! - Auto-turn-off display after configurable period of inactivity
//! - Tap to wake up the display
//! - Manual turn-off button with power icon
//!
//! Run with: cargo run --release

use iced::time;
use iced::widget::{button, center, column, container, row, text};
use iced::Length;
use iced::{mouse, window, Element, Event, Subscription, Task, Theme};
use std::time::Instant;

/// Configuration for display auto-turn-off behavior
mod config {
    use std::time::Duration;

    /// Duration of inactivity before display turns off (default: 10 minutes)
    pub const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(600);

    /// Interval to check for inactivity (checks every second)
    pub const CHECK_INTERVAL: Duration = Duration::from_secs(1);

    /// Size of the turn-off button
    pub const BUTTON_SIZE: f32 = 80.0;

    /// Size of the icon inside the button
    pub const ICON_SIZE: f32 = 40.0;
}

fn main() -> iced::Result {
    iced::application("Bob Server Display", BobDisplay::update, BobDisplay::view)
        .theme(|_| Theme::Dark)
        .window(window::Settings {
            size: iced::Size::new(1920.0, 1080.0),
            ..window::Settings::default()
        })
        .subscription(BobDisplay::subscription)
        .run_with(BobDisplay::new)
}

#[derive(Debug, Clone)]
enum Message {
    /// Timer tick for checking inactivity
    CheckInactivity,
    /// User interaction detected
    ActivityDetected,
    /// Turn off display button pressed
    TurnOffDisplay,
    /// Button pressed
    ButtonPressed(String),
}

struct BobDisplay {
    /// Whether the display is currently off
    display_off: bool,
    /// Last time user activity was detected
    last_activity: Instant,
    /// Start time for FPS calculation
    start_time: Instant,
    /// Frame counter
    frame_count: u64,
    /// Demo counter
    counter: u32,
}

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        let now = Instant::now();
        (
            Self {
                display_off: false,
                last_activity: now,
                start_time: now,
                frame_count: 0,
                counter: 0,
            },
            window::get_latest().and_then(|id| window::change_mode(id, window::Mode::Fullscreen)),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to timer events for inactivity checking
        let timer_subscription =
            time::every(config::CHECK_INTERVAL).map(|_| Message::CheckInactivity);

        // Subscribe to all events to detect activity
        let event_subscription = iced::event::listen().map(|event| {
            match event {
                // Detect mouse movement, clicks, and touch events
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Mouse(mouse::Event::ButtonPressed(_))
                | Event::Mouse(mouse::Event::ButtonReleased(_)) => Message::ActivityDetected,
                _ => Message::CheckInactivity,
            }
        });

        Subscription::batch([timer_subscription, event_subscription])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CheckInactivity => {
                if !self.display_off {
                    let elapsed = self.last_activity.elapsed();
                    if elapsed >= config::INACTIVITY_TIMEOUT {
                        self.display_off = true;
                    }
                }
                self.frame_count += 1;
            }
            Message::ActivityDetected => {
                if self.display_off {
                    // When display is off, any activity should wake it
                    self.display_off = false;
                }
                self.last_activity = Instant::now();
            }
            Message::TurnOffDisplay => {
                self.display_off = true;
            }
            Message::ButtonPressed(label) => {
                self.last_activity = Instant::now();
                if label == "Increment" {
                    self.counter += 1;
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.display_off {
            self.view_screen_off()
        } else {
            self.view_main_screen()
        }
    }

    fn view_screen_off(&self) -> Element<'_, Message> {
        // When screen is off, show a black container that captures all interactions
        // Tapping anywhere will wake the display
        container(center(
            text(""), // Empty text, just need something
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.0, 0.0, 0.0,
            ))),
            ..Default::default()
        })
        .into()
    }

    fn view_main_screen(&self) -> Element<'_, Message> {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let fps = if elapsed > 0.0 {
            self.frame_count as f32 / elapsed
        } else {
            0.0
        };

        let time_remaining =
            config::INACTIVITY_TIMEOUT.saturating_sub(self.last_activity.elapsed());
        let time_remaining_mins = time_remaining.as_secs() / 60;
        let time_remaining_secs = time_remaining.as_secs() % 60;

        let turn_off_button = button(
            center(
                // Square icon for turn-off button
                text("■").size(config::ICON_SIZE),
            )
            .width(Length::Fixed(config::BUTTON_SIZE))
            .height(Length::Fixed(config::BUTTON_SIZE)),
        )
        .on_press(Message::TurnOffDisplay)
        .width(Length::Fixed(config::BUTTON_SIZE))
        .height(Length::Fixed(config::BUTTON_SIZE));

        let content = column![
            row![text("Bob Server Display").size(40),].spacing(20),
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
            row![text(format!(
                "Auto-off in: {:02}:{:02}",
                time_remaining_mins, time_remaining_secs
            ))
            .size(20),]
            .spacing(20),
            // Turn off display button with icon
            row![text("Tap to turn off:").size(20), turn_off_button,]
                .spacing(10)
                .align_y(iced::Alignment::Center),
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
