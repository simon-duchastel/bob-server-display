//! Display power control module
//!
//! This module provides functionality to actually turn off the display backlight/output
//! using system commands like wlr-randr (for Wayland/Sway) or xrandr (for X11).

use std::process::Command;

/// Display controller that manages the actual display power state
pub struct DisplayController {
    display_output: String,
}

impl DisplayController {
    /// Create a new display controller
    /// Uses "HEADLESS-1" as default output for headless/Wayland setups
    pub fn new() -> Self {
        Self {
            display_output: "HEADLESS-1".to_string(),
        }
    }

    /// Create a new display controller with a specific output name
    pub fn with_output(output: &str) -> Self {
        Self {
            display_output: output.to_string(),
        }
    }

    /// Turn off the display using wlr-randr (Wayland/Sway)
    pub fn turn_off(&self) -> Result<(), String> {
        // Try wlr-randr first (Wayland/Sway)
        let result = Command::new("wlr-randr")
            .args([&self.display_output, "--off"])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                println!("Display turned off using wlr-randr");
                Ok(())
            }
            _ => {
                // Fallback: try swaymsg if wlr-randr fails
                let sway_result = Command::new("swaymsg")
                    .args(["output", &self.display_output, "disable"])
                    .output();

                match sway_result {
                    Ok(output) if output.status.success() => {
                        println!("Display turned off using swaymsg");
                        Ok(())
                    }
                    Ok(output) => Err(format!(
                        "Failed to turn off display: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )),
                    Err(e) => Err(format!("Failed to execute display off command: {}", e)),
                }
            }
        }
    }

    /// Turn on the display using wlr-randr (Wayland/Sway)
    pub fn turn_on(&self) -> Result<(), String> {
        // Try wlr-randr first (Wayland/Sway)
        let result = Command::new("wlr-randr")
            .args([&self.display_output, "--on"])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                println!("Display turned on using wlr-randr");
                Ok(())
            }
            _ => {
                // Fallback: try swaymsg if wlr-randr fails
                let sway_result = Command::new("swaymsg")
                    .args(["output", &self.display_output, "enable"])
                    .output();

                match sway_result {
                    Ok(output) if output.status.success() => {
                        println!("Display turned on using swaymsg");
                        Ok(())
                    }
                    Ok(output) => Err(format!(
                        "Failed to turn on display: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )),
                    Err(e) => Err(format!("Failed to execute display on command: {}", e)),
                }
            }
        }
    }

    /// Set the display output name
    #[allow(dead_code)]
    pub fn set_output(&mut self, output: &str) {
        self.display_output = output.to_string();
    }
}

impl Default for DisplayController {
    fn default() -> Self {
        Self::new()
    }
}
