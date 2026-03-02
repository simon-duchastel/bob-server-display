mod system;
mod view;

use iced::time;
use iced::widget::{button, center, container, mouse_area, text};
use iced::{mouse, window, Element, Event, Length, Subscription, Task, Theme};
use std::time::{Duration, Instant};
use system::{SystemMonitor, SystemStats};

/// Configuration for display auto-turn-off behavior
pub mod config {
    use std::time::Duration;

    /// Duration of inactivity before display turns off (default: 20 minutes)
    pub const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(1200);

    /// Interval to check for inactivity (checks every second)
    pub const CHECK_INTERVAL: Duration = Duration::from_secs(1);

    /// Size of the turn-off button
    pub const BUTTON_SIZE: f32 = 60.0;

    /// Size of the icon inside the button
    pub const ICON_SIZE: f32 = 30.0;
}

fn main() -> iced::Result {
    iced::application("Bob Server Display", BobDisplay::update, BobDisplay::view)
        .theme(|_| Theme::Dark)
        .window(window::Settings {
            size: iced::Size::new(1424.0, 280.0),
            resizable: false,
            ..window::Settings::default()
        })
        .subscription(BobDisplay::subscription)
        .run_with(BobDisplay::new)
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    StatsUpdated(SystemStats),
    /// Timer tick for checking inactivity
    CheckInactivity,
    /// User interaction detected
    ActivityDetected,
    /// Turn off display button pressed
    TurnOffDisplay,
}

struct BobDisplay {
    stats: SystemStats,
    system_monitor: SystemMonitor,
    /// Whether the display is currently off
    display_off: bool,
    /// Last time user activity was detected
    last_activity: Instant,
}

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        let mut system_monitor = SystemMonitor::new();
        let initial_stats = system_monitor.refresh();
        let now = Instant::now();

        (
            Self {
                stats: initial_stats,
                system_monitor,
                display_off: false,
                last_activity: now,
            },
            window::get_latest()
                .and_then(|id| Task::batch([window::change_mode(id, window::Mode::Windowed)])),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let stats = self.system_monitor.refresh();
                self.stats = stats;
            }
            Message::StatsUpdated(_stats) => {
                // This variant exists for potential async updates in the future
            }
            Message::CheckInactivity => {
                if !self.display_off {
                    let elapsed = self.last_activity.elapsed();
                    if elapsed >= config::INACTIVITY_TIMEOUT {
                        self.display_off = true;
                    }
                }
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
        mouse_area(
            container(center(text("")))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.0, 0.0, 0.0,
                    ))),
                    ..Default::default()
                }),
        )
        .on_press(Message::ActivityDetected)
        .into()
    }

    fn view_main_screen(&self) -> Element<'_, Message> {
        let time_remaining =
            config::INACTIVITY_TIMEOUT.saturating_sub(self.last_activity.elapsed());
        let time_remaining_mins = time_remaining.as_secs() / 60;
        let time_remaining_secs = time_remaining.as_secs() % 60;

        // Turn-off button with square icon
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

        view::build_view_with_controls(
            &self.stats,
            time_remaining_mins,
            time_remaining_secs,
            turn_off_button.into(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to timer events for stats update and inactivity checking
        let stats_subscription = time::every(Duration::from_secs(5)).map(|_| Message::Tick);

        let inactivity_check_subscription =
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

        Subscription::batch([
            stats_subscription,
            inactivity_check_subscription,
            event_subscription,
        ])
    }
}
