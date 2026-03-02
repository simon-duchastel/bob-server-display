mod system;
mod view;

use iced::time;
use iced::widget::{container, mouse_area};
use iced::{mouse, window, Background, Color, Element, Event, Length, Subscription, Task, Theme};
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
    /// Always dimmed at 5% brightness (95% opacity overlay) for testing
    dimmed: bool,
}

/// Opacity of the dim overlay (0.0 = fully transparent, 1.0 = fully opaque)
/// Set to 0.95 for 5% brightness (95% opacity)
const DIM_OPACITY: f32 = 0.95;

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        let mut system_monitor = SystemMonitor::new();
        let initial_stats = system_monitor.refresh();

        (
            Self {
                stats: initial_stats,
                system_monitor,
                dimmed: true, // Always start dimmed
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
                // Wake up from dimmed state on any interaction
                self.dimmed = false;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Build the main content view
        let main_content = view::build_view(&self.stats);

        // Wrap in dimmed overlay
        mouse_area(
            container(
                container(main_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(Background::Color(Color::from_rgba(
                            0.0,
                            0.0,
                            0.0,
                            DIM_OPACITY,
                        ))),
                        ..Default::default()
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .on_press(Message::WakeUp)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to timer events for stats update
        let stats_subscription = time::every(Duration::from_secs(5)).map(|_| Message::Tick);

        // Subscribe to mouse events to wake up
        let event_subscription = iced::event::listen().map(|event| {
            match event {
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Mouse(mouse::Event::ButtonPressed(_))
                | Event::Mouse(mouse::Event::ButtonReleased(_)) => Message::WakeUp,
                _ => Message::Tick, // Just tick for other events
            }
        });

        Subscription::batch([stats_subscription, event_subscription])
    }
}
