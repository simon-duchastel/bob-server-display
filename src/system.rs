use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use sysinfo::{Components, Networks, System};

/// System statistics data structure.
#[derive(Debug, Clone, Default)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub ram_usage_percent: f32,
    pub upload_mbps: f32,
    pub download_mbps: f32,
    pub temperature_celsius: f32,
}

/// Commands sent to the stats thread.
#[derive(Debug, Clone)]
pub enum StatsCommand {
    Refresh,
}

/// Responses from the stats thread.
#[derive(Debug, Clone)]
pub enum StatsResponse {
    Stats(SystemStats),
}

/// System monitor that runs in a dedicated thread.
pub struct SystemMonitor {
    command_sender: Sender<StatsCommand>,
}

impl SystemMonitor {
    /// Create a new system monitor with a dedicated background thread.
    /// Returns the monitor and a receiver for getting stats updates.
    pub fn new() -> (Self, Receiver<StatsResponse>) {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<StatsCommand>();
        let (resp_tx, resp_rx) = std::sync::mpsc::channel::<StatsResponse>();

        // Spawn dedicated thread for system monitoring
        thread::spawn(move || {
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

            for network in networks.values() {
                last_rx += network.total_received();
                last_tx += network.total_transmitted();
            }

            // Process commands from main thread
            while let Ok(StatsCommand::Refresh) = cmd_rx.recv() {
                let start_time = std::time::Instant::now();

                // Refresh existing objects (fast, no allocation)
                system.refresh_all();
                components.refresh(false);
                networks.refresh(false);

                let elapsed_secs = start_time.duration_since(last_time).as_secs_f32();
                last_time = start_time;

                // Calculate stats
                let stats = calculate_stats(
                    &system,
                    &components,
                    &networks,
                    &mut last_rx,
                    &mut last_tx,
                    elapsed_secs,
                );

                // Send back to main thread
                if resp_tx.send(StatsResponse::Stats(stats)).is_err() {
                    break;
                }
            }
        });

        (
            Self {
                command_sender: cmd_tx,
            },
            resp_rx,
        )
    }

    /// Send a refresh command to the stats thread.
    pub fn refresh(&self) {
        let _ = self.command_sender.send(StatsCommand::Refresh);
    }
}

/// Calculate system statistics from sysinfo objects.
fn calculate_stats(
    system: &System,
    components: &Components,
    networks: &Networks,
    last_rx: &mut u64,
    last_tx: &mut u64,
    elapsed_secs: f32,
) -> SystemStats {
    // CPU
    let cpu_usage = if !system.cpus().is_empty() {
        system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / system.cpus().len() as f32
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
    let current_rx: u64 = networks.values().map(|n| n.total_received()).sum();
    let current_tx: u64 = networks.values().map(|n| n.total_transmitted()).sum();

    let rx_delta = current_rx.saturating_sub(*last_rx);
    let tx_delta = current_tx.saturating_sub(*last_tx);

    let download_mbps = (rx_delta as f32 * 8.0 / 1_000_000.0) / elapsed_secs.max(0.001);
    let upload_mbps = (tx_delta as f32 * 8.0 / 1_000_000.0) / elapsed_secs.max(0.001);

    *last_rx = current_rx;
    *last_tx = current_tx;

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

    SystemStats {
        cpu_usage,
        ram_used_gb,
        ram_total_gb: total_memory as f32 / 1024.0 / 1024.0 / 1024.0,
        ram_usage_percent,
        upload_mbps,
        download_mbps,
        temperature_celsius,
    }
}
