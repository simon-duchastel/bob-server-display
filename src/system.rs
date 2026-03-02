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
    Shutdown,
}

/// Responses from the stats thread.
#[derive(Debug, Clone)]
pub enum StatsResponse {
    Stats(SystemStats),
}
