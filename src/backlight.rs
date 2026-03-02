//! Display backlight control module
//!
//! This module provides actual hardware brightness control for the display
//! by writing to /sys/class/backlight/*/brightness

use std::fs;
use std::path::Path;

/// Controls the display backlight brightness
pub struct BacklightController {
    brightness_path: String,
    max_brightness: u32,
    /// Store the normal brightness level to restore to
    normal_brightness: u32,
}

impl BacklightController {
    /// Create a new backlight controller
    /// Attempts to auto-detect the backlight device
    pub fn new() -> Result<Self, String> {
        // Auto-detect backlight device
        let backlight_dir = Path::new("/sys/class/backlight");

        if !backlight_dir.exists() {
            return Err("No backlight control available".to_string());
        }

        // Find first available backlight device
        let entries = fs::read_dir(backlight_dir)
            .map_err(|e| format!("Failed to read backlight directory: {}", e))?;

        for entry in entries.flatten() {
            let name = entry.file_name();
            let device_name = name.to_string_lossy();

            // Skip symlinks like ".." and "."
            if device_name.starts_with(".") {
                continue;
            }

            let brightness_path = format!("/sys/class/backlight/{}/brightness", device_name);
            let max_brightness_path =
                format!("/sys/class/backlight/{}/max_brightness", device_name);

            // Check if brightness file exists and is writable
            if Path::new(&brightness_path).exists() {
                // Read max brightness
                let max_brightness = fs::read_to_string(&max_brightness_path)
                    .map_err(|e| format!("Failed to read max brightness: {}", e))?
                    .trim()
                    .parse::<u32>()
                    .map_err(|e| format!("Failed to parse max brightness: {}", e))?;

                // Read current brightness as "normal" level
                let current_brightness = fs::read_to_string(&brightness_path)
                    .map_err(|e| format!("Failed to read current brightness: {}", e))?
                    .trim()
                    .parse::<u32>()
                    .map_err(|e| format!("Failed to parse current brightness: {}", e))?;

                println!(
                    "Found backlight device: {} (max: {}, current: {})",
                    device_name, max_brightness, current_brightness
                );

                return Ok(Self {
                    brightness_path,
                    max_brightness,
                    normal_brightness: current_brightness,
                });
            }
        }

        Err("No usable backlight device found".to_string())
    }

    /// Create a new backlight controller for a specific device
    #[allow(dead_code)]
    pub fn with_device(device_name: &str) -> Result<Self, String> {
        let brightness_path = format!("/sys/class/backlight/{}/brightness", device_name);
        let max_brightness_path = format!("/sys/class/backlight/{}/max_brightness", device_name);

        if !Path::new(&brightness_path).exists() {
            return Err(format!("Backlight device '{}' not found", device_name));
        }

        let max_brightness = fs::read_to_string(&max_brightness_path)
            .map_err(|e| format!("Failed to read max brightness: {}", e))?
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse max brightness: {}", e))?;

        let current_brightness = fs::read_to_string(&brightness_path)
            .map_err(|e| format!("Failed to read current brightness: {}", e))?
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse current brightness: {}", e))?;

        Ok(Self {
            brightness_path,
            max_brightness,
            normal_brightness: current_brightness,
        })
    }

    /// Set brightness to a specific percentage (0-100)
    pub fn set_brightness_percent(&self, percent: u32) -> Result<(), String> {
        let percent = percent.clamp(0, 100);
        let brightness = (self.max_brightness as f32 * percent as f32 / 100.0) as u32;
        self.set_brightness(brightness)
    }

    /// Set brightness to a specific value
    fn set_brightness(&self, brightness: u32) -> Result<(), String> {
        let brightness = brightness.min(self.max_brightness);

        fs::write(&self.brightness_path, brightness.to_string()).map_err(|e| {
            format!(
                "Failed to set brightness: {} (path: {})",
                e, self.brightness_path
            )
        })?;

        println!(
            "Set brightness to {} ({}%)",
            brightness,
            (brightness as f32 / self.max_brightness as f32 * 100.0) as u32
        );

        Ok(())
    }

    /// Dim the display to a very low brightness (5%)
    pub fn dim(&self) -> Result<(), String> {
        self.set_brightness_percent(5)
    }

    /// Restore display to normal brightness
    pub fn restore(&self) -> Result<(), String> {
        self.set_brightness(self.normal_brightness)
    }

    /// Get current brightness
    #[allow(dead_code)]
    pub fn current_brightness(&self) -> Result<u32, String> {
        fs::read_to_string(&self.brightness_path)
            .map_err(|e| format!("Failed to read brightness: {}", e))?
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse brightness: {}", e))
    }

    /// Get max brightness
    #[allow(dead_code)]
    pub fn max_brightness(&self) -> u32 {
        self.max_brightness
    }
}

impl Default for BacklightController {
    fn default() -> Self {
        // This will panic if no backlight is available, but provides a default
        Self::new().expect("No backlight control available")
    }
}
