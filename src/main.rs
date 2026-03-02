mod system;
mod view;

use iced::time;
use iced::{window, Element, Task, Theme};
use std::time::Duration;
use system::SystemStats;

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

/// Result from async stats collection including current network totals.
#[derive(Debug, Clone)]
struct RefreshResult {
    stats: SystemStats,
    current_rx: u64,
    current_tx: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    StatsUpdated(RefreshResult),
}

struct BobDisplay {
    stats: SystemStats,
    last_rx: u64,
    last_tx: u64,
    last_time: std::time::Instant,
}

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        // Get initial network snapshot synchronously
        let (initial_rx, initial_tx) = Self::get_network_totals();
        let initial_time = std::time::Instant::now();
        
        // Do an initial stats collection
        let (initial_stats, current_rx, current_tx) = 
            Self::collect_stats_sync(initial_rx, initial_tx, initial_time);

        (
            Self {
                stats: initial_stats,
                last_rx: current_rx,
                last_tx: current_tx,
                last_time: std::time::Instant::now(),
            },
            window::get_latest()
                .and_then(|id| Task::batch([window::change_mode(id, window::Mode::Windowed)])),
        )
    }

    /// Get current network totals synchronously.
    fn get_network_totals() -> (u64, u64) {
        use sysinfo::Networks;
        let networks = Networks::new_with_refreshed_list();
        let rx: u64 = networks.iter().map(|(_, n)| n.total_received()).sum();
        let tx: u64 = networks.iter().map(|(_, n)| n.total_transmitted()).sum();
        (rx, tx)
    }

    /// Collect stats synchronously (for initial load).
    fn collect_stats_sync(
        last_rx: u64, 
        last_tx: u64, 
        last_time: std::time::Instant
    ) -> (SystemStats, u64, u64) {
        use sysinfo::{Components, Networks, System};
        
        let mut system = System::new_all();
        let mut components = Components::new_with_refreshed_list();
        let mut networks = Networks::new_with_refreshed_list();

        system.refresh_all();
        components.refresh(false);
        networks.refresh(false);

        let now = std::time::Instant::now();
        let elapsed_secs = now.duration_since(last_time).as_secs_f32();

        // CPU
        let cpu_usage = if !system.cpus().is_empty() {
            system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() 
                / system.cpus().len() as f32
        } else {
            0.0
        };

        // Memory
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let ram_used_gb = used_memory as f32 / 1024.0 / 1024.0 / 1024.0;
        let ram_total_gb = total_memory as f32 / 1024.0 / 1024.0 / 1024.0;
        let ram_usage_percent = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };

        // Network
        let current_rx: u64 = networks.iter().map(|(_, n)| n.total_received()).sum();
        let current_tx: u64 = networks.iter().map(|(_, n)| n.total_transmitted()).sum();
        
        let rx_delta = current_rx.saturating_sub(last_rx);
        let tx_delta = current_tx.saturating_sub(last_tx);
        
        let download_mbps = (rx_delta as f32 * 8.0 / 1_000_000.0) / elapsed_secs.max(0.001);
        let upload_mbps = (tx_delta as f32 * 8.0 / 1_000_000.0) / elapsed_secs.max(0.001);

        // Temperature
        let temperature_celsius = components
            .iter()
            .filter(|c| {
                c.label().to_lowercase().contains("cpu")
                    || c.label().to_lowercase().contains("thermal")
                    || c.label().to_lowercase().contains("k10temp")
                    || c.label().to_lowercase().contains("coretemp")
            })
            .map(|c| c.temperature())
            .next()
            .flatten()
            .unwrap_or(0.0);

        let stats = SystemStats {
            cpu_usage,
            ram_used_gb,
            ram_total_gb,
            ram_usage_percent,
            upload_mbps,
            download_mbps,
            temperature_celsius,
        };
        
        (stats, current_rx, current_tx)
    }

    /// Collect stats asynchronously on a background thread.
    async fn collect_stats_async(
        last_rx: u64,
        last_tx: u64,
        last_time: std::time::Instant,
    ) -> RefreshResult {
        tokio::task::spawn_blocking(move || {
            use sysinfo::{Components, Networks, System};
            
            let mut system = System::new_all();
            let mut components = Components::new_with_refreshed_list();
            let mut networks = Networks::new_with_refreshed_list();

            system.refresh_all();
            components.refresh(false);
            networks.refresh(false);

            let now = std::time::Instant::now();
            let elapsed_secs = now.duration_since(last_time).as_secs_f32();

            // CPU
            let cpu_usage = if !system.cpus().is_empty() {
                system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() 
                    / system.cpus().len() as f32
            } else {
                0.0
            };

            // Memory
            let total_memory = system.total_memory();
            let used_memory = system.used_memory();
            let ram_used_gb = used_memory as f32 / 1024.0 / 1024.0 / 1024.0;
            let ram_total_gb = total_memory as f32 / 1024.0 / 1024.0 / 1024.0;
            let ram_usage_percent = if total_memory > 0 {
                (used_memory as f32 / total_memory as f32) * 100.0
            } else {
                0.0
            };

            // Network
            let current_rx: u64 = networks.iter().map(|(_, n)| n.total_received()).sum();
            let current_tx: u64 = networks.iter().map(|(_, n)| n.total_transmitted()).sum();
            
            let rx_delta = current_rx.saturating_sub(last_rx);
            let tx_delta = current_tx.saturating_sub(last_tx);
            
            let download_mbps = (rx_delta as f32 * 8.0 / 1_000_000.0) / elapsed_secs.max(0.001);
            let upload_mbps = (tx_delta as f32 * 8.0 / 1_000_000.0) / elapsed_secs.max(0.001);

            // Temperature
            let temperature_celsius = components
                .iter()
                .filter(|c| {
                    c.label().to_lowercase().contains("cpu")
                        || c.label().to_lowercase().contains("thermal")
                        || c.label().to_lowercase().contains("k10temp")
                        || c.label().to_lowercase().contains("coretemp")
                })
                .map(|c| c.temperature())
                .next()
                .flatten()
                .unwrap_or(0.0);

            RefreshResult {
                stats: SystemStats {
                    cpu_usage,
                    ram_used_gb,
                    ram_total_gb,
                    ram_usage_percent,
                    upload_mbps,
                    download_mbps,
                    temperature_celsius,
                },
                current_rx,
                current_tx,
            }
        })
        .await
        .unwrap_or_else(|_| RefreshResult {
            stats: SystemStats::default(),
            current_rx: 0,
            current_tx: 0,
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                return Task::perform(
                    Self::collect_stats_async(self.last_rx, self.last_tx, self.last_time),
                    Message::StatsUpdated,
                );
            }
            Message::StatsUpdated(result) => {
                self.stats = result.stats;
                self.last_rx = result.current_rx;
                self.last_tx = result.current_tx;
                self.last_time = std::time::Instant::now();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        view::build_view(&self.stats)
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        time::every(Duration::from_millis(REFRESH_INTERVAL_MS)).map(|_| Message::Tick)
    }
}
