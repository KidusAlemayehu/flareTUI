use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CloudflaredStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub binary_path: Option<String>,
}

pub struct CloudflaredManager;

impl CloudflaredManager {
    pub fn is_installed() -> bool {
        Command::new("which")
            .arg("cloudflared")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn check_status() -> CloudflaredStatus {
        let which_out = Command::new("which")
            .arg("cloudflared")
            .output();

        let binary_path = which_out
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        if let Some(path) = binary_path {
            let ver_out = Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| {
                    let full = String::from_utf8_lossy(&o.stdout);
                    full.lines().next().unwrap_or("").trim().to_string()
                });

            CloudflaredStatus {
                installed: true,
                version: ver_out,
                binary_path: Some(path),
            }
        } else {
            CloudflaredStatus {
                installed: false,
                version: None,
                binary_path: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflared_status_defaults() {
        let status = CloudflaredStatus::default();
        assert!(!status.installed);
        assert!(status.version.is_none());
        assert!(status.binary_path.is_none());
    }
}
