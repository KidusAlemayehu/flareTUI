use std::fs;
use std::path::PathBuf;
use color_eyre::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub api_token: Option<String>,
    pub account_id: Option<String>,
    pub account_name: Option<String>,
}

pub struct AuthManager;

impl AuthManager {
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "flare", "flareTUI").map(|dirs| {
            let config_dir = dirs.config_dir();
            let _ = fs::create_dir_all(config_dir);
            config_dir.join("config.toml")
        })
    }

    pub fn load_token() -> Option<String> {
        // 1. Environment variable takes highest priority
        if let Ok(token) = std::env::var("CLOUDFLARE_API_TOKEN") {
            if !token.trim().is_empty() {
                return Some(token.trim().to_string());
            }
        }

        // 2. Load from config.toml
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str::<Config>(&contents) {
                        if let Some(token) = config.api_token {
                            if !token.trim().is_empty() {
                                return Some(token.trim().to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn load_config() -> Config {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str::<Config>(&contents) {
                        return config;
                    }
                }
            }
        }
        Config::default()
    }

    pub fn save_token(token: &str, account_id: Option<String>, account_name: Option<String>) -> Result<()> {
        if let Some(path) = Self::config_path() {
            let config = Config {
                api_token: Some(token.to_string()),
                account_id,
                account_name,
            };
            let toml_str = toml::to_string_pretty(&config)?;
            fs::write(path, toml_str)?;
        }
        Ok(())
    }
}
