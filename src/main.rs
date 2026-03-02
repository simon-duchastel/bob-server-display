mod system;
mod view;

use iced::{window, Element, Subscription, Task, Theme};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use system::{SystemMonitor, SystemStats, StatsResponse};

/// Refresh rate - 10 times per second (100ms)
const REFRESH_INTERVAL_MS: u64 = 100;

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
    stats_receiver: Arc<Mutex<Receiver<StatsResponse>>>,
}

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        // Create system monitor with dedicated thread
        let (system_monitor, stats_receiver) = SystemMonitor::new();

        // Wrap receiver in Arc<Mutex> for thread-safe access
        let stats_receiver = Arc::new(Mutex::new(stats_receiver));

        let display = Self {
            stats: SystemStats::default(),
            system_monitor,
            stats_receiver,
        };

        // Request initial stats
        display.system_monitor.refresh();

        let init_task =
            window::get_latest().and_then(|id| Task::batch([window::change_mode(
                id,
                window::Mode::Windowed,
            )]));

        (display, init_task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                // Send refresh command to dedicated thread
                self.system_monitor.refresh();

                // Check for pending stats responses
                while let Ok(StatsResponse::Stats(stats)) =
                    self.stats_receiver.lock().unwrap().try_recv()
                {
                    self.stats = stats;
                }
            }
            Message::StatsUpdated(stats) => {
                self.stats = stats;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        view::build_view(&self.stats)
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;

        // Timer for sending refresh commands
        time::every(Duration::from_millis(REFRESH_INTERVAL_MS)).map(|_| Message::Tick)
    }
}
