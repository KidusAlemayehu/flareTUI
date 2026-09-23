use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WarpStatus {
    pub installed: bool,
    pub connected: bool,
    pub status_text: String,
    pub mode: String,
    pub organization: Option<String>,
}

impl Default for WarpStatus {
    fn default() -> Self {
        Self {
            installed: false,
            connected: false,
            status_text: "Checking...".to_string(),
            mode: "Unknown".to_string(),
            organization: None,
        }
    }
}

impl WarpStatus {
    pub fn not_installed() -> Self {
        Self {
            installed: false,
            connected: false,
            status_text: "Not Installed".to_string(),
            mode: "N/A".to_string(),
            organization: None,
        }
    }
}

pub struct WarpManager;

impl WarpManager {
    pub fn is_installed() -> bool {
        Command::new("which")
            .arg("warp-cli")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub async fn check_status() -> WarpStatus {
        if !Self::is_installed() {
            return WarpStatus::not_installed();
        }

        let status_output = tokio::process::Command::new("warp-cli")
            .arg("status")
            .output()
            .await;

        let mut connected = false;
        let mut status_text = "Disconnected".to_string();

        if let Ok(out) = status_output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lower = stdout.to_lowercase();
            if lower.contains("status update: connected") || (lower.contains("connected") && !lower.contains("disconnected")) {
                connected = true;
                status_text = "Connected".to_string();
            } else if lower.contains("connecting") {
                status_text = "Connecting...".to_string();
            } else {
                status_text = "Disconnected".to_string();
            }
        }

        // Check settings/mode
        let settings_output = tokio::process::Command::new("warp-cli")
            .arg("settings")
            .output()
            .await;

        let mut mode = "WARP".to_string();
        let mut organization = None;

        if let Ok(out) = settings_output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.to_lowercase().starts_with("mode:") {
                    mode = trimmed[5..].trim().to_string();
                } else if trimmed.to_lowercase().starts_with("organization:") || trimmed.to_lowercase().starts_with("team:") {
                    let parts: Vec<&str> = trimmed.split(':').collect();
                    if parts.len() > 1 {
                        organization = Some(parts[1].trim().to_string());
                    }
                }
            }
        }

        WarpStatus {
            installed: true,
            connected,
            status_text,
            mode,
            organization,
        }
    }

    pub async fn connect() -> Result<String> {
        if !Self::is_installed() {
            color_eyre::eyre::bail!("warp-cli is not installed on this system");
        }
        let out = tokio::process::Command::new("warp-cli")
            .arg("connect")
            .output()
            .await?;
        let msg = String::from_utf8_lossy(&out.stdout).to_string();
        if out.status.success() {
            Ok(if msg.trim().is_empty() { "Connecting to Cloudflare WARP...".to_string() } else { msg })
        } else {
            let err = String::from_utf8_lossy(&out.stderr).to_string();
            color_eyre::eyre::bail!("{err}")
        }
    }

    pub async fn disconnect() -> Result<String> {
        if !Self::is_installed() {
            color_eyre::eyre::bail!("warp-cli is not installed on this system");
        }
        let out = tokio::process::Command::new("warp-cli")
            .arg("disconnect")
            .output()
            .await?;
        let msg = String::from_utf8_lossy(&out.stdout).to_string();
        if out.status.success() {
            Ok(if msg.trim().is_empty() { "Disconnected from Cloudflare WARP".to_string() } else { msg })
        } else {
            let err = String::from_utf8_lossy(&out.stderr).to_string();
            color_eyre::eyre::bail!("{err}")
        }
    }

    pub async fn toggle() -> Result<String> {
        let status = Self::check_status().await;
        if status.connected {
            Self::disconnect().await
        } else {
            Self::connect().await
        }
    }

    pub async fn cycle_mode() -> Result<String> {
        if !Self::is_installed() {
            color_eyre::eyre::bail!("warp-cli is not installed on this system");
        }
        let status = Self::check_status().await;
        let new_mode = match status.mode.to_lowercase().as_str() {
            m if m.contains("doh") => "warp",
            _ => "doh",
        };
        let out = tokio::process::Command::new("warp-cli")
            .args(["set-mode", new_mode])
            .output()
            .await?;
        if out.status.success() {
            Ok(format!("WARP mode switched to {new_mode}"))
        } else {
            let err = String::from_utf8_lossy(&out.stderr).to_string();
            color_eyre::eyre::bail!("{err}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warp_status_defaults() {
        let status = WarpStatus::not_installed();
        assert!(!status.installed);
        assert!(!status.connected);
        assert_eq!(status.status_text, "Not Installed");
        assert_eq!(status.mode, "N/A");
    }
}
