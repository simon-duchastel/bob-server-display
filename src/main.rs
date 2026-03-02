mod system;
mod view;

use iced::time;
use iced::{window, Element, Subscription, Task, Theme};
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

#[derive(Debug, Clone)]
pub enum Message {
    StatsUpdated(SystemStats),
}

struct BobDisplay {
    stats: SystemStats,
}

impl BobDisplay {
    fn new() -> (Self, Task<Message>) {
        // Spawn dedicated thread that continuously refreshes stats
        let (resp_tx, resp_rx) = std::sync::mpsc::channel::<SystemStats>();
        
        std::thread::spawn(move || {
            use sysinfo::{Components, Networks, System};
            
            // Initialize persistent sysinfo objects (created once)
            let mut system = System::new_all();
            let mut components = Components::new_with_refreshed_list();
            let mut networks = Networks::new_with_refreshed_list();
            
            let mut last_rx: u64 = 0;
            let mut last_tx: u64 = 0;
            let mut last_time = std::time::Instant::now();
            
            // Initial refresh to populate baseline
            system.refresh_all();
            components.refresh(false);
            networks.refresh(false);
            
            for (_, network) in &networks {
                last_rx += network.total_received();
                last_tx += network.total_transmitted();
            }
            
            // Continuous refresh loop
            loop {
                let start_time = std::time::Instant::now();
                
                // Refresh existing objects (fast, no allocation)
                system.refresh_all();
                components.refresh(false);
                networks.refresh(false);
                
                let elapsed_secs = start_time.duration_since(last_time).as_secs_f32();
                last_time = start_time;
                
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
                
                last_rx = current_rx;
                last_tx = current_tx;
                
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
                    ram_total_gb: total_memory as f32 / 1024.0 / 1024.0 / 1024.0,
                    ram_usage_percent,
                    upload_mbps,
                    download_mbps,
                    temperature_celsius,
                };
                
                // Send to main thread
                if resp_tx.send(stats).is_err() {
                    break;
                }
                
                // Sleep until next refresh
                let elapsed = start_time.elapsed();
                if elapsed < Duration::from_millis(REFRESH_INTERVAL_MS) {
                    std::thread::sleep(Duration::from_millis(REFRESH_INTERVAL_MS) - elapsed);
                }
            }
        });
        
        let display = Self {
            stats: SystemStats::default(),
        };
        
        // Store receiver for subscription
        unsafe {
            STATS_RECEIVER = Some(std::sync::Arc::new(std::sync::Mutex::new(resp_rx)));
        }
        
        let init_task = window::get_latest()
            .and_then(|id| Task::batch([window::change_mode(id, window::Mode::Windowed)]));

        (display, init_task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
        // Channel receiver subscription
        unsafe {
            if let Some(ref rx) = STATS_RECEIVER {
                let rx = rx.clone();
                // Use unfold to continuously poll the receiver
                iced::subscription::unfold(
                    std::any::TypeId::of::<SystemStats>(),
                    rx,
                    |rx| async move {
                        // Try to receive without blocking
                        loop {
                            if let Ok(stats) = rx.lock().unwrap().try_recv() {
                                return (Some(Message::StatsUpdated(stats)), rx);
                            }
                            // Yield to let other tasks run
                            tokio::task::yield_now().await;
                        }
                    },
                )
            } else {
                iced::Subscription::none()
            }
        }
    }
}

/// Global storage for the stats receiver (wrapped in Arc<Mutex> for sharing).
static mut STATS_RECEIVER: Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<SystemStats>>>> = None;
