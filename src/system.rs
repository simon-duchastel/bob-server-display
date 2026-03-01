use sysinfo::{Components, Networks, System};

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

pub struct SystemMonitor {
    system: System,
    components: Components,
    networks: Networks,
    last_network_rx: u64,
    last_network_tx: u64,
    last_update: std::time::Instant,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let system = System::new_all();
        let components = Components::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        let mut last_network_rx = 0u64;
        let mut last_network_tx = 0u64;

        for (_interface_name, network) in &networks {
            last_network_rx += network.total_received();
            last_network_tx += network.total_transmitted();
        }

        Self {
            system,
            components,
            networks,
            last_network_rx,
            last_network_tx,
            last_update: std::time::Instant::now(),
        }
    }

    pub fn refresh(&mut self) -> SystemStats {
        self.system.refresh_all();
        self.components.refresh(false);
        self.networks.refresh(false);

        let now = std::time::Instant::now();
        let elapsed_secs = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // CPU usage
        let cpu_usage = if !self.system.cpus().is_empty() {
            self.system
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .sum::<f32>()
                / self.system.cpus().len() as f32
        } else {
            0.0
        };

        // Memory
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let ram_used_gb = used_memory as f32 / 1024.0 / 1024.0 / 1024.0;
        let ram_total_gb = total_memory as f32 / 1024.0 / 1024.0 / 1024.0;
        let ram_usage_percent = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };

        // Network
        let mut current_rx = 0u64;
        let mut current_tx = 0u64;
        for (_interface_name, network) in &self.networks {
            current_rx += network.total_received();
            current_tx += network.total_transmitted();
        }

        let rx_delta = if current_rx >= self.last_network_rx {
            current_rx - self.last_network_rx
        } else {
            current_rx
        };
        let tx_delta = if current_tx >= self.last_network_tx {
            current_tx - self.last_network_tx
        } else {
            current_tx
        };

        self.last_network_rx = current_rx;
        self.last_network_tx = current_tx;

        let download_mbps = (rx_delta as f32 * 8.0 / 1_000_000.0) / elapsed_secs.max(0.1);
        let upload_mbps = (tx_delta as f32 * 8.0 / 1_000_000.0) / elapsed_secs.max(0.1);

        // Temperature - get the first CPU temperature sensor
        let temperature_celsius = self
            .components
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
            ram_total_gb,
            ram_usage_percent,
            upload_mbps,
            download_mbps,
            temperature_celsius,
        }
    }
}
