mod system;
mod view;

use iced::time;
use iced::{window, Element, Task, Theme};
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
}

struct BobDisplay {
    stats: SystemStats,
    system_monitor: SystemMonitor,
}

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        let mut system_monitor = SystemMonitor::new();
        let initial_stats = system_monitor.refresh();

        (
            Self {
                stats: initial_stats,
                system_monitor,
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
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        view::build_view(&self.stats)
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        time::every(Duration::from_secs(2)).map(|_| Message::Tick)
    }
}
