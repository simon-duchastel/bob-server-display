mod backlight;
mod system;
mod view;

use iced::time;
use iced::widget::{container, mouse_area};
use iced::{mouse, window, Element, Event, Length, Subscription, Task, Theme};
use std::time::Duration;
use system::{SystemMonitor, SystemStats};

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
    /// User interaction detected - wake up from dimmed state
    WakeUp,
}

struct BobDisplay {
    stats: SystemStats,
    system_monitor: SystemMonitor,
    /// Track if display should be dimmed
    dimmed: bool,
}

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        let mut system_monitor = SystemMonitor::new();
        let initial_stats = system_monitor.refresh();

        // Dim the display immediately on startup
        if let Ok(controller) = backlight::BacklightController::new() {
            if let Err(e) = controller.dim() {
                eprintln!("Warning: Failed to dim display: {}", e);
            }
        } else {
            eprintln!("Warning: No backlight control available");
        }

        (
            Self {
                stats: initial_stats,
                system_monitor,
                dimmed: true, // Start dimmed
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
            Message::WakeUp => {
                if self.dimmed {
                    // Restore normal brightness
                    if let Ok(controller) = backlight::BacklightController::new() {
                        if let Err(e) = controller.restore() {
                            eprintln!("Warning: Failed to restore brightness: {}", e);
                        }
                    }
                    self.dimmed = false;
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Build the main content view
        let main_content = view::build_view(&self.stats);

        // Wrap in clickable area to wake up when dimmed
        mouse_area(
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::WakeUp)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to timer events for stats update
        let stats_subscription = time::every(Duration::from_secs(5)).map(|_| Message::Tick);

        // Subscribe to mouse events to wake up when dimmed
        let event_subscription = iced::event::listen().map(|event| match event {
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Mouse(mouse::Event::ButtonReleased(_)) => Message::WakeUp,
            _ => Message::Tick,
        });

        Subscription::batch([stats_subscription, event_subscription])
    }
}
