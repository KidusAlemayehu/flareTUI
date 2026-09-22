use color_eyre::Result;
#[cfg(not(test))]
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountProfile {
    pub name: String,
    pub api_token: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub account_name: Option<String>,
    #[serde(default)]
    pub default_zone_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub current_profile: Option<String>,
    #[serde(default)]
    pub profiles: Vec<AccountProfile>,

    // System settings persistence
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default = "default_fast_sync")]
    pub fast_sync: bool,
    #[serde(default = "default_sync_interval")]
    pub sync_interval_secs: u64,
    #[serde(default = "default_auto_sync_on_tab_change")]
    pub auto_sync_on_tab_change: bool,
    #[serde(default)]
    pub default_tab: Option<usize>,

    // Legacy fallback fields:
    #[serde(default)]
    pub api_token: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub account_name: Option<String>,
}

fn default_fast_sync() -> bool {
    true
}

fn default_sync_interval() -> u64 {
    15
}

fn default_auto_sync_on_tab_change() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            current_profile: None,
            profiles: Vec::new(),
            theme: None,
            fast_sync: default_fast_sync(),
            sync_interval_secs: default_sync_interval(),
            auto_sync_on_tab_change: default_auto_sync_on_tab_change(),
            default_tab: None,
            api_token: None,
            account_id: None,
            account_name: None,
        }
    }
}

impl Config {
    pub fn get_effective_profiles(&self) -> Vec<AccountProfile> {
        let mut profiles = self.profiles.clone();
        if profiles.is_empty() {
            if let Some(tok) = &self.api_token {
                if !tok.trim().is_empty() {
                    profiles.push(AccountProfile {
                        name: "default".to_string(),
                        api_token: tok.clone(),
                        account_id: self.account_id.clone(),
                        account_name: self.account_name.clone(),
                        default_zone_id: None,
                    });
                }
            }
        }
        profiles
    }

    pub fn active_profile(&self) -> Option<AccountProfile> {
        let effective = self.get_effective_profiles();
        if let Some(name) = &self.current_profile {
            if let Some(p) = effective.iter().find(|p| p.name == *name) {
                return Some(p.clone());
            }
        }
        effective.first().cloned()
    }

    pub fn active_token(&self) -> Option<String> {
        // 1. Environment variable takes highest priority
        if let Ok(token) = std::env::var("CLOUDFLARE_API_TOKEN") {
            if !token.trim().is_empty() {
                return Some(token.trim().to_string());
            }
        }

        // 2. Profile token
        if let Some(profile) = self.active_profile() {
            if !profile.api_token.trim().is_empty() {
                return Some(profile.api_token.trim().to_string());
            }
        }

        // 3. Legacy token
        if let Some(token) = &self.api_token {
            if !token.trim().is_empty() {
                return Some(token.trim().to_string());
            }
        }

        None
    }

    pub fn switch_profile(&mut self, profile_name: &str) -> bool {
        let effective = self.get_effective_profiles();
        if effective.iter().any(|p| p.name == profile_name) {
            self.current_profile = Some(profile_name.to_string());
            if self.profiles.is_empty() {
                self.profiles = effective;
            }
            true
        } else {
            false
        }
    }

    pub fn add_or_update_profile(&mut self, profile: AccountProfile) {
        if self.profiles.is_empty() {
            self.profiles = self.get_effective_profiles();
        }
        if let Some(pos) = self.profiles.iter().position(|p| p.name == profile.name) {
            self.profiles[pos] = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    pub fn delete_profile(&mut self, profile_name: &str) -> bool {
        if self.profiles.is_empty() {
            self.profiles = self.get_effective_profiles();
        }
        if let Some(pos) = self.profiles.iter().position(|p| p.name == profile_name) {
            self.profiles.remove(pos);
            if self.current_profile.as_deref() == Some(profile_name) {
                self.current_profile = self.profiles.first().map(|p| p.name.clone());
            }
            true
        } else {
            false
        }
    }

    pub fn logout_active_profile(&mut self) -> bool {
        let active_name = self
            .current_profile
            .clone()
            .unwrap_or_else(|| "default".to_string());
        if let Some(pos) = self.profiles.iter().position(|p| p.name == active_name) {
            self.profiles[pos].api_token.clear();
        }
        self.api_token = None;
        true
    }
}

pub struct AuthManager;

impl AuthManager {
    pub fn config_path() -> Option<PathBuf> {
        if let Ok(custom) = std::env::var("FLARE_CONFIG_PATH") {
            return Some(PathBuf::from(custom));
        }
        #[cfg(test)]
        {
            let tmp = std::env::temp_dir().join("flaretui_test_config.toml");
            Some(tmp)
        }
        #[cfg(not(test))]
        ProjectDirs::from("com", "flare", "flareTUI").map(|dirs| {
            let config_dir = dirs.config_dir();
            let _ = fs::create_dir_all(config_dir);
            config_dir.join("config.toml")
        })
    }

    pub fn load_token() -> Option<String> {
        let config = Self::load_config();
        config.active_token()
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

    pub fn save_config(config: &Config) -> Result<()> {
        if let Some(path) = Self::config_path() {
            let toml_str = toml::to_string_pretty(config)?;
            fs::write(path, toml_str)?;
        }
        Ok(())
    }

    pub fn save_token(
        token: &str,
        account_id: Option<String>,
        account_name: Option<String>,
    ) -> Result<()> {
        let mut config = Self::load_config();
        let active_name = config
            .current_profile
            .clone()
            .unwrap_or_else(|| "default".to_string());
        config.add_or_update_profile(AccountProfile {
            name: active_name.clone(),
            api_token: token.to_string(),
            account_id: account_id.clone(),
            account_name: account_name.clone(),
            default_zone_id: None,
        });
        config.current_profile = Some(active_name);
        config.api_token = Some(token.to_string());
        config.account_id = account_id;
        config.account_name = account_name;
        Self::save_config(&config)
    }

    pub fn switch_active_profile(profile_name: &str) -> Result<Option<AccountProfile>> {
        let mut config = Self::load_config();
        if config.switch_profile(profile_name) {
            let active = config.active_profile();
            Self::save_config(&config)?;
            Ok(active)
        } else {
            Ok(None)
        }
    }

    pub fn logout() -> Result<()> {
        let mut config = Self::load_config();
        config.logout_active_profile();
        Self::save_config(&config)?;
        std::env::remove_var("CLOUDFLARE_API_TOKEN");
        Ok(())
    }

    pub fn update_theme(theme_name: &str) -> Result<()> {
        let mut config = Self::load_config();
        config.theme = Some(theme_name.to_string());
        Self::save_config(&config)
    }

    pub fn update_fast_sync(enabled: bool) -> Result<()> {
        let mut config = Self::load_config();
        config.fast_sync = enabled;
        Self::save_config(&config)
    }

    pub fn update_sync_interval(interval_secs: u64) -> Result<()> {
        let mut config = Self::load_config();
        config.sync_interval_secs = interval_secs;
        Self::save_config(&config)
    }

    pub fn update_auto_sync_on_tab_change(enabled: bool) -> Result<()> {
        let mut config = Self::load_config();
        config.auto_sync_on_tab_change = enabled;
        Self::save_config(&config)
    }

    pub fn update_default_tab(tab: usize) -> Result<()> {
        let mut config = Self::load_config();
        config.default_tab = Some(tab);
        Self::save_config(&config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logout_active_profile() {
        let mut config = Config {
            current_profile: Some("prod".to_string()),
            profiles: vec![AccountProfile {
                name: "prod".to_string(),
                api_token: "secret_tok_123".to_string(),
                account_id: None,
                account_name: None,
                default_zone_id: None,
            }],
            api_token: Some("secret_tok_123".to_string()),
            account_id: None,
            account_name: None,
            ..Default::default()
        };

        assert!(config.active_token().is_some());
        config.logout_active_profile();
        assert!(config.active_token().is_none());
    }

    #[test]
    fn test_config_legacy_fallback() {
        let config = Config {
            current_profile: None,
            profiles: Vec::new(),
            api_token: Some("legacy_tok_123".to_string()),
            account_id: Some("acc_123".to_string()),
            account_name: Some("Legacy Account".to_string()),
            ..Default::default()
        };

        let effective = config.get_effective_profiles();
        assert_eq!(effective.len(), 1);
        assert_eq!(effective[0].name, "default");
        assert_eq!(effective[0].api_token, "legacy_tok_123");

        let active = config.active_profile().expect("active profile");
        assert_eq!(active.name, "default");
    }

    #[test]
    fn test_config_active_profile_and_switching() {
        let mut config = Config {
            current_profile: Some("production".to_string()),
            profiles: vec![
                AccountProfile {
                    name: "staging".to_string(),
                    api_token: "tok_stage_456".to_string(),
                    account_id: Some("acc_stage".to_string()),
                    account_name: Some("Staging Account".to_string()),
                    default_zone_id: None,
                },
                AccountProfile {
                    name: "production".to_string(),
                    api_token: "tok_prod_789".to_string(),
                    account_id: Some("acc_prod".to_string()),
                    account_name: Some("Prod Account".to_string()),
                    default_zone_id: None,
                },
            ],
            api_token: None,
            account_id: None,
            account_name: None,
            ..Default::default()
        };

        let active = config.active_profile().expect("active profile");
        assert_eq!(active.name, "production");
        assert_eq!(active.api_token, "tok_prod_789");

        assert!(config.switch_profile("staging"));
        assert_eq!(config.current_profile.as_deref(), Some("staging"));
        assert_eq!(config.active_profile().unwrap().name, "staging");

        assert!(!config.switch_profile("non_existent"));
    }

    #[test]
    fn test_config_add_and_delete_profile() {
        let mut config = Config::default();
        config.add_or_update_profile(AccountProfile {
            name: "personal".to_string(),
            api_token: "tok_pers_111".to_string(),
            account_id: None,
            account_name: None,
            default_zone_id: None,
        });
        assert_eq!(config.profiles.len(), 1);

        config.add_or_update_profile(AccountProfile {
            name: "work".to_string(),
            api_token: "tok_work_222".to_string(),
            account_id: None,
            account_name: None,
            default_zone_id: None,
        });
        assert_eq!(config.profiles.len(), 2);

        assert!(config.delete_profile("personal"));
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, "work");
    }

    #[test]
    fn test_config_toml_serialization() {
        let config = Config {
            current_profile: Some("prod".to_string()),
            profiles: vec![AccountProfile {
                name: "prod".to_string(),
                api_token: "secret_tok".to_string(),
                account_id: Some("acc_abc".to_string()),
                account_name: Some("Prod".to_string()),
                default_zone_id: Some("zone_123".to_string()),
            }],
            api_token: None,
            account_id: None,
            account_name: None,
            ..Default::default()
        };

        let toml_str = toml::to_string_pretty(&config).expect("serialize config");
        assert!(toml_str.contains("current_profile = \"prod\""));
        assert!(toml_str.contains("api_token = \"secret_tok\""));
        assert!(toml_str.contains("fast_sync = true"));
        assert!(toml_str.contains("sync_interval_secs = 15"));

        let deserialized: Config = toml::from_str(&toml_str).expect("deserialize config");
        assert_eq!(deserialized, config);
    }

    #[test]
    fn test_config_settings_persistence_defaults() {
        let toml_str = r#"
            theme = "Tokyo Night"
            fast_sync = false
            sync_interval_secs = 30
            auto_sync_on_tab_change = false
            default_tab = 2
        "#;
        let config: Config = toml::from_str(toml_str).expect("deserialize settings");
        assert_eq!(config.theme.as_deref(), Some("Tokyo Night"));
        assert!(!config.fast_sync);
        assert_eq!(config.sync_interval_secs, 30);
        assert!(!config.auto_sync_on_tab_change);
        assert_eq!(config.default_tab, Some(2));
    }
}
