mod system;
mod view;
mod wave_chart;

use iced::{window, Element, Subscription, Task, Theme};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use system::{StatsResponse, SystemMonitor, SystemStats};
use wave_chart::WaveData;

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
    // Historical data for wave animations
    cpu_history: WaveData,
    ram_history: WaveData,
    temp_history: WaveData,
    upload_history: WaveData,
    download_history: WaveData,
    // For network auto-scaling: track max values over time
    max_upload_mbps: f32,
    max_download_mbps: f32,
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
            // Store 60 data points (6 seconds of history at 10Hz)
            cpu_history: WaveData::new(60),
            ram_history: WaveData::new(60),
            temp_history: WaveData::new(60),
            upload_history: WaveData::new(60),
            download_history: WaveData::new(60),
            max_upload_mbps: 1.0,   // Start with 1 Mbps baseline
            max_download_mbps: 1.0, // Start with 1 Mbps baseline
        };

        // Request initial stats
        display.system_monitor.refresh();

        let init_task = window::get_latest()
            .and_then(|id| Task::batch([window::change_mode(id, window::Mode::Windowed)]));

        (display, init_task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                // Send refresh command to dedicated thread
                self.system_monitor.refresh();

                // Check for pending stats responses
                let mut new_stats = None;
                while let Ok(StatsResponse::Stats(stats)) =
                    self.stats_receiver.lock().unwrap().try_recv()
                {
                    new_stats = Some(stats);
                }

                // Update history and stats if we received new data
                if let Some(stats) = new_stats {
                    self.update_history(&stats);
                    self.stats = stats;
                }
            }
            Message::StatsUpdated(stats) => {
                self.update_history(&stats);
                self.stats = stats;
            }
        }
        Task::none()
    }

    /// Update historical data with new stats.
    fn update_history(&mut self, stats: &SystemStats) {
        // CPU: normalize to 0.0-1.0 range (already in %)
        self.cpu_history.push(stats.cpu_usage / 100.0);

        // RAM: normalize to 0.0-1.0 range (already in %)
        self.ram_history.push(stats.ram_usage_percent / 100.0);

        // Temperature: normalize based on reasonable range (40-80°C)
        // Below 40°C = 0%, above 80°C = 100%
        const TEMP_MIN: f32 = 40.0;
        const TEMP_MAX: f32 = 80.0;
        let temp_normalized = if stats.temperature_celsius <= TEMP_MIN {
            0.0
        } else if stats.temperature_celsius >= TEMP_MAX {
            1.0
        } else {
            (stats.temperature_celsius - TEMP_MIN) / (TEMP_MAX - TEMP_MIN)
        };
        self.temp_history.push(temp_normalized);

        // Network: auto-scaling based on recent max values
        // Update max values with exponential decay (recent values matter more)
        const DECAY: f32 = 0.98; // Decay factor for old max values
        self.max_upload_mbps = self.max_upload_mbps * DECAY + stats.upload_mbps * (1.0 - DECAY);
        self.max_download_mbps =
            self.max_download_mbps * DECAY + stats.download_mbps * (1.0 - DECAY);

        // Ensure minimum scale to avoid division by zero
        let upload_scale = self.max_upload_mbps.max(1.0);
        let download_scale = self.max_download_mbps.max(1.0);

        // Normalize network values (can exceed 1.0 if current > recent max, which is fine)
        self.upload_history.push(stats.upload_mbps / upload_scale);
        self.download_history
            .push(stats.download_mbps / download_scale);
    }

    fn view(&self) -> Element<'_, Message> {
        view::build_view(
            &self.stats,
            &self.cpu_history,
            &self.ram_history,
            &self.temp_history,
            &self.upload_history,
            &self.download_history,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;

        // Timer for sending refresh commands
        time::every(Duration::from_millis(REFRESH_INTERVAL_MS)).map(|_| Message::Tick)
    }
}
