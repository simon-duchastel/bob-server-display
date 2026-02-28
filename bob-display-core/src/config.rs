use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// DRM device path (e.g., /dev/dri/card0)
    pub drm_device: String,

    /// Display mode (optional - auto-detect if not specified)
    pub mode: Option<DisplayMode>,

    /// Background color (RGBA)
    #[serde(default = "default_background")]
    pub background_color: [u8; 4],

    /// Text color (RGBA)
    #[serde(default = "default_text_color")]
    pub text_color: [u8; 4],

    /// Font size in pixels
    #[serde(default = "default_font_size")]
    pub font_size: f32,

    /// Target frames per second
    #[serde(default = "default_fps")]
    pub target_fps: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32, // in Hz
}

impl Default for Config {
    fn default() -> Self {
        Self {
            drm_device: "/dev/dri/card0".to_string(),
            mode: None,
            background_color: default_background(),
            text_color: default_text_color(),
            font_size: default_font_size(),
            target_fps: default_fps(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Check for config file locations in order of priority
        let config_paths = [
            Path::new("/etc/bob-display/config.toml"),
            Path::new("./config.toml"),
        ];

        for path in &config_paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read config from {:?}", path))?;
                let config: Config = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse config from {:?}", path))?;
                tracing::info!("Loaded configuration from {:?}", path);
                return Ok(config);
            }
        }

        // Use default configuration
        tracing::warn!("No configuration file found, using defaults");
        Ok(Config::default())
    }
}

fn default_background() -> [u8; 4] {
    [0, 0, 0, 255] // Black
}

fn default_text_color() -> [u8; 4] {
    [255, 255, 255, 255] // White
}

fn default_font_size() -> f32 {
    24.0
}

fn default_fps() -> u32 {
    60
}