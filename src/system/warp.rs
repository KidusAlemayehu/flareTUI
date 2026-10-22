use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

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

        // Run warp-cli --accept-tos status with 3s timeout
        let status_cmd = tokio::process::Command::new("warp-cli")
            .args(["--accept-tos", "status"])
            .stdin(Stdio::null())
            .output();

        let status_output = timeout(Duration::from_secs(3), status_cmd).await;

        let mut connected = false;
        let mut status_text = "Disconnected".to_string();

        if let Ok(Ok(out)) = status_output {
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
        } else if status_output.is_err() {
            status_text = "Timeout".to_string();
        }

        // Check settings/mode with 3s timeout
        let settings_cmd = tokio::process::Command::new("warp-cli")
            .args(["--accept-tos", "settings"])
            .stdin(Stdio::null())
            .output();

        let settings_output = timeout(Duration::from_secs(3), settings_cmd).await;

        let mut mode = "WARP".to_string();
        let mut organization = None;

        if let Ok(Ok(out)) = settings_output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                let lower = trimmed.to_lowercase();
                if lower.contains("mode:") {
                    if let Some(idx) = lower.find("mode:") {
                        let val = trimmed[idx + 5..].trim();
                        if !val.is_empty() {
                            mode = val.to_string();
                        }
                    }
                } else if lower.contains("organization:") || lower.contains("team:") {
                    let parts: Vec<&str> = trimmed.split(':').collect();
                    if parts.len() > 1 && !parts[1].trim().is_empty() {
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
        let connect_cmd = tokio::process::Command::new("warp-cli")
            .args(["--accept-tos", "connect"])
            .stdin(Stdio::null())
            .output();

        let out = match timeout(Duration::from_secs(6), connect_cmd).await {
            Ok(res) => res?,
            Err(_) => color_eyre::eyre::bail!("Connection attempt timed out. Ensure warp-svc daemon is running."),
        };

        let msg = String::from_utf8_lossy(&out.stdout).to_string();
        if out.status.success() {
            Ok(if msg.trim().is_empty() || msg.trim().eq_ignore_ascii_case("success") {
                "Connected to Cloudflare WARP".to_string()
            } else {
                msg.trim().to_string()
            })
        } else {
            let err = String::from_utf8_lossy(&out.stderr).to_string();
            color_eyre::eyre::bail!("{err}")
        }
    }

    pub async fn disconnect() -> Result<String> {
        if !Self::is_installed() {
            color_eyre::eyre::bail!("warp-cli is not installed on this system");
        }
        let disconnect_cmd = tokio::process::Command::new("warp-cli")
            .args(["--accept-tos", "disconnect"])
            .stdin(Stdio::null())
            .output();

        let out = match timeout(Duration::from_secs(6), disconnect_cmd).await {
            Ok(res) => res?,
            Err(_) => color_eyre::eyre::bail!("Disconnection attempt timed out."),
        };

        let msg = String::from_utf8_lossy(&out.stdout).to_string();
        if out.status.success() {
            Ok(if msg.trim().is_empty() || msg.trim().eq_ignore_ascii_case("success") {
                "Disconnected from Cloudflare WARP".to_string()
            } else {
                msg.trim().to_string()
            })
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

        // Try `warp-cli --accept-tos mode <mode>` first (modern), fallback to `set-mode` (legacy)
        let mode_cmd = tokio::process::Command::new("warp-cli")
            .args(["--accept-tos", "mode", new_mode])
            .stdin(Stdio::null())
            .output();

        let out = match timeout(Duration::from_secs(5), mode_cmd).await {
            Ok(Ok(out)) if out.status.success() => out,
            _ => {
                let fallback_cmd = tokio::process::Command::new("warp-cli")
                    .args(["--accept-tos", "set-mode", new_mode])
                    .stdin(Stdio::null())
                    .output();
                match timeout(Duration::from_secs(5), fallback_cmd).await {
                    Ok(res) => res?,
                    Err(_) => color_eyre::eyre::bail!("Mode switch command timed out."),
                }
            }
        };

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
