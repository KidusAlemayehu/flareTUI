use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CommandCategory {
    Navigation,
    Cloudflare,
    Security,
    Storage,
    Workers,
    Domains,
    Profiles,
    System,
}

impl CommandCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandCategory::Navigation => "Navigation",
            CommandCategory::Cloudflare => "Cloudflare",
            CommandCategory::Security => "Security",
            CommandCategory::Storage => "Storage",
            CommandCategory::Workers => "Workers",
            CommandCategory::Domains => "Domains & Registrar",
            CommandCategory::Profiles => "Profiles",
            CommandCategory::System => "System",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaletteAction {
    SwitchView(usize),
    TriggerSync,
    FastSyncActiveView,
    ToggleFastSync,
    CycleSyncInterval,
    ToggleAutoSyncOnTabChange,
    PurgeCache,
    OpenD1Console,
    OpenKvModal,
    OpenSecurityInspector,
    OpenProfileSwitcher,
    OpenOAuthLogin,
    ToggleAuth,
    Logout,
    OpenDomainCreateModal,
    ToggleDomainAutoRenew,
    ToggleDomainLock,
    ToggleDomainPrivacy,
    ToggleWarp,
    CycleWarpMode,
    OpenHelp,
    SetTheme(String),
    Quit,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteCommand {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: CommandCategory,
    pub shortcut: Option<String>,
    pub keywords: Vec<String>,
    pub action: PaletteAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoredCommand {
    pub command: PaletteCommand,
    pub score: u32,
    pub matched_indices: Vec<u32>,
}

pub struct PaletteEngine {
    matcher: Matcher,
    commands: Vec<PaletteCommand>,
}

impl Default for PaletteEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PaletteEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            matcher: Matcher::new(Config::DEFAULT),
            commands: Vec::new(),
        };
        engine.register_default_commands();
        engine
    }

    pub fn commands(&self) -> &[PaletteCommand] {
        &self.commands
    }

    pub fn register(&mut self, cmd: PaletteCommand) {
        if let Some(pos) = self.commands.iter().position(|c| c.id == cmd.id) {
            self.commands[pos] = cmd;
        } else {
            self.commands.push(cmd);
        }
    }

    pub fn register_all(&mut self, cmds: Vec<PaletteCommand>) {
        for cmd in cmds {
            self.register(cmd);
        }
    }

    pub fn register_default_commands(&mut self) {
        let defaults = vec![
            // Navigation
            PaletteCommand {
                id: "nav_dns".to_string(),
                title: "DNS Records & Zones".to_string(),
                description: "Manage DNS zone records, proxy status, and TTLs".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("1".to_string()),
                keywords: vec![
                    "dns".into(),
                    "zone".into(),
                    "cname".into(),
                    "a".into(),
                    "mx".into(),
                    "proxied".into(),
                ],
                action: PaletteAction::SwitchView(0),
            },
            PaletteCommand {
                id: "nav_workers".to_string(),
                title: "Workers Scripts & Deployments".to_string(),
                description: "View and inspect serverless Cloudflare Workers".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("2".to_string()),
                keywords: vec![
                    "workers".into(),
                    "serverless".into(),
                    "scripts".into(),
                    "routes".into(),
                ],
                action: PaletteAction::SwitchView(1),
            },
            PaletteCommand {
                id: "nav_pages".to_string(),
                title: "Pages Projects & Builds".to_string(),
                description: "Inspect Cloudflare Pages deployments and build logs".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("3".to_string()),
                keywords: vec![
                    "pages".into(),
                    "deployments".into(),
                    "builds".into(),
                    "jamstack".into(),
                ],
                action: PaletteAction::SwitchView(2),
            },
            PaletteCommand {
                id: "nav_storage".to_string(),
                title: "Storage: D1 & KV".to_string(),
                description: "Explore D1 SQL databases and Workers KV namespaces".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("4".to_string()),
                keywords: vec![
                    "storage".into(),
                    "d1".into(),
                    "kv".into(),
                    "database".into(),
                    "sql".into(),
                ],
                action: PaletteAction::SwitchView(3),
            },
            PaletteCommand {
                id: "nav_tail".to_string(),
                title: "Live Tail Log Stream".to_string(),
                description: "Stream live Cloudflare Workers console logs and errors".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("5".to_string()),
                keywords: vec![
                    "tail".into(),
                    "logs".into(),
                    "stream".into(),
                    "realtime".into(),
                    "console".into(),
                ],
                action: PaletteAction::SwitchView(4),
            },
            PaletteCommand {
                id: "nav_security".to_string(),
                title: "Security & WAF Dashboard".to_string(),
                description: "Monitor real-time threat levels, WAF rules, and attacks".to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("6".to_string()),
                keywords: vec![
                    "security".into(),
                    "waf".into(),
                    "firewall".into(),
                    "threats".into(),
                    "ddos".into(),
                    "blocks".into(),
                ],
                action: PaletteAction::SwitchView(5),
            },
            PaletteCommand {
                id: "nav_domains".to_string(),
                title: "Domains & Registrar Dashboard".to_string(),
                description: "Manage registered domains, renewals, and registrar settings"
                    .to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("7".to_string()),
                keywords: vec![
                    "domains".into(),
                    "registrar".into(),
                    "whois".into(),
                    "dns".into(),
                    "nameservers".into(),
                ],
                action: PaletteAction::SwitchView(6),
            },
            PaletteCommand {
                id: "nav_tunnels".to_string(),
                title: "Zero Trust & Tunnels Dashboard".to_string(),
                description: "Monitor Cloudflare Tunnels, ingress rules, and WARP CLI status"
                    .to_string(),
                category: CommandCategory::Navigation,
                shortcut: Some("8".to_string()),
                keywords: vec![
                    "tunnels".into(),
                    "tunnel".into(),
                    "warp".into(),
                    "cloudflared".into(),
                    "zero trust".into(),
                    "ingress".into(),
                ],
                action: PaletteAction::SwitchView(7),
            },
            PaletteCommand {
                id: "warp_toggle".to_string(),
                title: "WARP: Connect / Disconnect".to_string(),
                description: "Toggle Cloudflare WARP client tunnel connectivity".to_string(),
                category: CommandCategory::System,
                shortcut: Some("w".to_string()),
                keywords: vec![
                    "warp".into(),
                    "connect".into(),
                    "disconnect".into(),
                    "toggle".into(),
                    "vpn".into(),
                    "tunnel".into(),
                ],
                action: PaletteAction::ToggleWarp,
            },
            PaletteCommand {
                id: "warp_mode".to_string(),
                title: "WARP: Cycle Client Mode".to_string(),
                description: "Switch WARP operating mode between WARP, DoH, and Proxy".to_string(),
                category: CommandCategory::System,
                shortcut: Some("m".to_string()),
                keywords: vec![
                    "warp".into(),
                    "mode".into(),
                    "doh".into(),
                    "dns".into(),
                    "switch".into(),
                ],
                action: PaletteAction::CycleWarpMode,
            },
            // Cloudflare Actions
            PaletteCommand {
                id: "cf_sync".to_string(),
                title: "Sync All Cloudflare Resources".to_string(),
                description: "Fetch and cache all zones, DNS, workers, and security events"
                    .to_string(),
                category: CommandCategory::Cloudflare,
                shortcut: Some("s".to_string()),
                keywords: vec![
                    "sync".into(),
                    "refresh".into(),
                    "reload".into(),
                    "fetch".into(),
                    "update".into(),
                ],
                action: PaletteAction::TriggerSync,
            },
            PaletteCommand {
                id: "cf_fast_sync".to_string(),
                title: "Fast Sync Active View".to_string(),
                description: "Immediately revalidate Cloudflare live data for active view tab"
                    .to_string(),
                category: CommandCategory::Cloudflare,
                shortcut: Some("r".to_string()),
                keywords: vec![
                    "fast".into(),
                    "sync".into(),
                    "revalidate".into(),
                    "poll".into(),
                    "view".into(),
                    "refresh".into(),
                ],
                action: PaletteAction::FastSyncActiveView,
            },
            PaletteCommand {
                id: "cf_purge".to_string(),
                title: "Purge Zone CDN Cache".to_string(),
                description: "Purge cached assets from Cloudflare Edge CDN".to_string(),
                category: CommandCategory::Cloudflare,
                shortcut: None,
                keywords: vec![
                    "purge".into(),
                    "cache".into(),
                    "flush".into(),
                    "clear".into(),
                    "cdn".into(),
                ],
                action: PaletteAction::PurgeCache,
            },
            PaletteCommand {
                id: "auth_account".to_string(),
                title: "Cloudflare Authentication & Account".to_string(),
                description: "View account profile, manage session, or authenticate".to_string(),
                category: CommandCategory::Profiles,
                shortcut: Some("a".to_string()),
                keywords: vec![
                    "auth".into(),
                    "account".into(),
                    "profile".into(),
                    "login".into(),
                    "user".into(),
                    "credentials".into(),
                ],
                action: PaletteAction::ToggleAuth,
            },
            PaletteCommand {
                id: "auth_oauth".to_string(),
                title: "Login via Browser (OAuth)".to_string(),
                description: "Authenticate flareTUI by redirecting to your web browser".to_string(),
                category: CommandCategory::Cloudflare,
                shortcut: None,
                keywords: vec![
                    "login".into(),
                    "oauth".into(),
                    "browser".into(),
                    "auth".into(),
                    "signin".into(),
                ],
                action: PaletteAction::OpenOAuthLogin,
            },
            PaletteCommand {
                id: "auth_logout".to_string(),
                title: "Log Out Active Session".to_string(),
                description: "Disconnect active Cloudflare account and switch to offline mode"
                    .to_string(),
                category: CommandCategory::Cloudflare,
                shortcut: Some("Ctrl+L".to_string()),
                keywords: vec![
                    "logout".into(),
                    "signout".into(),
                    "disconnect".into(),
                    "auth".into(),
                ],
                action: PaletteAction::Logout,
            },
            // Domains & Registrar Commands
            PaletteCommand {
                id: "domain_register".to_string(),
                title: "Register / Add New Domain".to_string(),
                description: "Onboard domain zone or register domain with Cloudflare Registrar"
                    .to_string(),
                category: CommandCategory::Domains,
                shortcut: Some("n".to_string()),
                keywords: vec![
                    "domain".into(),
                    "register".into(),
                    "add".into(),
                    "new".into(),
                    "zone".into(),
                    "onboard".into(),
                ],
                action: PaletteAction::OpenDomainCreateModal,
            },
            PaletteCommand {
                id: "domain_toggle_autorenew".to_string(),
                title: "Toggle Domain Auto-Renew".to_string(),
                description: "Enable or disable automatic renewal for selected domain".to_string(),
                category: CommandCategory::Domains,
                shortcut: Some("a".to_string()),
                keywords: vec![
                    "auto-renew".into(),
                    "renew".into(),
                    "subscription".into(),
                    "domain".into(),
                ],
                action: PaletteAction::ToggleDomainAutoRenew,
            },
            PaletteCommand {
                id: "domain_toggle_lock".to_string(),
                title: "Toggle Domain Transfer Lock".to_string(),
                description: "Enable or disable registrar transfer lock for selected domain"
                    .to_string(),
                category: CommandCategory::Domains,
                shortcut: Some("l".to_string()),
                keywords: vec![
                    "lock".into(),
                    "transfer".into(),
                    "domain".into(),
                    "registrar".into(),
                ],
                action: PaletteAction::ToggleDomainLock,
            },
            PaletteCommand {
                id: "domain_toggle_privacy".to_string(),
                title: "Toggle WHOIS Privacy Protection".to_string(),
                description: "Enable or disable WHOIS privacy redaction for selected domain"
                    .to_string(),
                category: CommandCategory::Domains,
                shortcut: Some("p".to_string()),
                keywords: vec![
                    "privacy".into(),
                    "whois".into(),
                    "redact".into(),
                    "domain".into(),
                ],
                action: PaletteAction::ToggleDomainPrivacy,
            },
            // Storage
            PaletteCommand {
                id: "storage_d1_console".to_string(),
                title: "Open D1 SQL Query Console".to_string(),
                description: "Interactive SQL query runner with table schema browser".to_string(),
                category: CommandCategory::Storage,
                shortcut: Some("d".to_string()),
                keywords: vec![
                    "d1".into(),
                    "sql".into(),
                    "query".into(),
                    "sqlite".into(),
                    "console".into(),
                ],
                action: PaletteAction::OpenD1Console,
            },
            PaletteCommand {
                id: "storage_kv_browser".to_string(),
                title: "Browse Workers KV Keys".to_string(),
                description: "Explore, search, and view JSON values in KV namespaces".to_string(),
                category: CommandCategory::Storage,
                shortcut: Some("k".to_string()),
                keywords: vec![
                    "kv".into(),
                    "keys".into(),
                    "browser".into(),
                    "values".into(),
                    "json".into(),
                ],
                action: PaletteAction::OpenKvModal,
            },
            // Security
            PaletteCommand {
                id: "sec_inspect".to_string(),
                title: "Inspect Security Threat Event".to_string(),
                description: "Open detailed inspector with IP lookup and WAF rule details"
                    .to_string(),
                category: CommandCategory::Security,
                shortcut: Some("Enter".to_string()),
                keywords: vec![
                    "inspect".into(),
                    "threat".into(),
                    "ray".into(),
                    "ip".into(),
                    "rule".into(),
                    "payload".into(),
                ],
                action: PaletteAction::OpenSecurityInspector,
            },
            // Profiles
            PaletteCommand {
                id: "profile_switch".to_string(),
                title: "Switch Cloudflare Account Profile".to_string(),
                description: "Hot-swap active credentials and profile environment".to_string(),
                category: CommandCategory::Profiles,
                shortcut: Some("Ctrl+U".to_string()),
                keywords: vec![
                    "profile".into(),
                    "account".into(),
                    "switch".into(),
                    "token".into(),
                    "credentials".into(),
                ],
                action: PaletteAction::OpenProfileSwitcher,
            },
            // System
            PaletteCommand {
                id: "sys_help".to_string(),
                title: "Keyboard Shortcuts & Help".to_string(),
                description: "Display global keybindings and navigation cheat sheet".to_string(),
                category: CommandCategory::System,
                shortcut: Some("?".to_string()),
                keywords: vec![
                    "help".into(),
                    "shortcuts".into(),
                    "keys".into(),
                    "guide".into(),
                ],
                action: PaletteAction::OpenHelp,
            },
            PaletteCommand {
                id: "sys_quit".to_string(),
                title: "Quit flareTUI".to_string(),
                description: "Exit flareTUI terminal interface cleanly".to_string(),
                category: CommandCategory::System,
                shortcut: Some("q".to_string()),
                keywords: vec!["quit".into(), "exit".into(), "close".into()],
                action: PaletteAction::Quit,
            },
            // Theme Switching
            PaletteCommand {
                id: "theme_cf_dark".to_string(),
                title: "Theme: Cloudflare Dark (Default)".to_string(),
                description: "Switch to official Cloudflare Dark orange and navy palette"
                    .to_string(),
                category: CommandCategory::System,
                shortcut: None,
                keywords: vec![
                    "theme".into(),
                    "cloudflare".into(),
                    "dark".into(),
                    "orange".into(),
                    "color".into(),
                ],
                action: PaletteAction::SetTheme("Cloudflare Dark".to_string()),
            },
            PaletteCommand {
                id: "theme_catppuccin".to_string(),
                title: "Theme: Catppuccin Mocha".to_string(),
                description: "Switch to soothing Catppuccin Mocha pastel palette".to_string(),
                category: CommandCategory::System,
                shortcut: None,
                keywords: vec![
                    "theme".into(),
                    "catppuccin".into(),
                    "mocha".into(),
                    "pastel".into(),
                    "mauve".into(),
                ],
                action: PaletteAction::SetTheme("Catppuccin Mocha".to_string()),
            },
            PaletteCommand {
                id: "theme_tokyo_night".to_string(),
                title: "Theme: Tokyo Night".to_string(),
                description: "Switch to vibrant Tokyo Night neon cyan and purple palette"
                    .to_string(),
                category: CommandCategory::System,
                shortcut: None,
                keywords: vec![
                    "theme".into(),
                    "tokyo".into(),
                    "night".into(),
                    "neon".into(),
                    "cyan".into(),
                ],
                action: PaletteAction::SetTheme("Tokyo Night".to_string()),
            },
            PaletteCommand {
                id: "theme_nord".to_string(),
                title: "Theme: Nord".to_string(),
                description: "Switch to arctic, north-bluish Nord palette".to_string(),
                category: CommandCategory::System,
                shortcut: None,
                keywords: vec![
                    "theme".into(),
                    "nord".into(),
                    "arctic".into(),
                    "blue".into(),
                    "frost".into(),
                ],
                action: PaletteAction::SetTheme("Nord".to_string()),
            },
            // System Fast Sync & Persistence Settings
            PaletteCommand {
                id: "sys_fast_sync_toggle".to_string(),
                title: "Toggle Fast Live Sync".to_string(),
                description: "Enable or disable background periodic polling of Cloudflare changes"
                    .to_string(),
                category: CommandCategory::System,
                shortcut: None,
                keywords: vec![
                    "fast".into(),
                    "sync".into(),
                    "live".into(),
                    "toggle".into(),
                    "poll".into(),
                    "cache".into(),
                ],
                action: PaletteAction::ToggleFastSync,
            },
            PaletteCommand {
                id: "sys_sync_interval_cycle".to_string(),
                title: "Cycle Fast Sync Interval (5s/10s/15s/30s/60s)".to_string(),
                description: "Change background live sync polling frequency".to_string(),
                category: CommandCategory::System,
                shortcut: None,
                keywords: vec![
                    "sync".into(),
                    "interval".into(),
                    "frequency".into(),
                    "rate".into(),
                    "period".into(),
                    "timer".into(),
                ],
                action: PaletteAction::CycleSyncInterval,
            },
            PaletteCommand {
                id: "sys_auto_sync_tab_toggle".to_string(),
                title: "Toggle Auto-Sync on Tab Switch".to_string(),
                description: "Revalidate Cloudflare data automatically when switching tabs"
                    .to_string(),
                category: CommandCategory::System,
                shortcut: None,
                keywords: vec![
                    "auto".into(),
                    "sync".into(),
                    "tab".into(),
                    "switch".into(),
                    "navigation".into(),
                ],
                action: PaletteAction::ToggleAutoSyncOnTabChange,
            },
        ];

        self.register_all(defaults);
    }

    pub fn search(&mut self, query: &str) -> Vec<ScoredCommand> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            // Return all commands sorted by category and title with base score
            let mut all: Vec<ScoredCommand> = self
                .commands
                .iter()
                .cloned()
                .map(|cmd| ScoredCommand {
                    command: cmd,
                    score: 100,
                    matched_indices: Vec::new(),
                })
                .collect();
            all.sort_by(|a, b| {
                a.command
                    .category
                    .cmp(&b.command.category)
                    .then_with(|| a.command.title.cmp(&b.command.title))
            });
            return all;
        }

        let pattern = Pattern::parse(trimmed, CaseMatching::Ignore, Normalization::Smart);
        let mut results = Vec::new();
        let mut title_buf = Vec::new();
        let mut haystack_buf = Vec::new();

        for cmd in &self.commands {
            // 1. Check title match for highlighting
            title_buf.clear();
            let title_utf32 = Utf32Str::new(&cmd.title, &mut title_buf);
            let mut matched_indices = Vec::new();
            let title_score = pattern.indices(title_utf32, &mut self.matcher, &mut matched_indices);
            matched_indices.sort_unstable();
            matched_indices.dedup();

            // 2. Build full searchable haystack: title + category + description + keywords
            let haystack = format!(
                "{} {} {} {}",
                cmd.title,
                cmd.category.as_str(),
                cmd.description,
                cmd.keywords.join(" ")
            );
            haystack_buf.clear();
            let haystack_utf32 = Utf32Str::new(&haystack, &mut haystack_buf);
            let haystack_score = pattern.score(haystack_utf32, &mut self.matcher);

            let final_score = match (title_score, haystack_score) {
                (Some(ts), Some(hs)) => Some(ts.max(hs) + 40), // Title boost
                (Some(ts), None) => Some(ts + 40),
                (None, Some(hs)) => Some(hs),
                (None, None) => None,
            };

            if let Some(score) = final_score {
                results.push(ScoredCommand {
                    command: cmd.clone(),
                    score,
                    matched_indices,
                });
            }
        }

        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.command.title.cmp(&b.command.title))
        });
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_commands_catalog() {
        let engine = PaletteEngine::new();
        assert!(engine.commands().len() >= 10);
        assert!(engine.commands().iter().any(|c| c.id == "nav_security"));
        assert!(engine
            .commands()
            .iter()
            .any(|c| c.id == "storage_d1_console"));
        assert!(engine.commands().iter().any(|c| c.id == "profile_switch"));
        assert!(engine.commands().iter().any(|c| c.id == "nav_tunnels"));
        assert!(engine.commands().iter().any(|c| c.id == "warp_toggle"));
    }

    #[test]
    fn test_palette_tunnels_and_warp_commands() {
        let mut engine = PaletteEngine::new();
        let results = engine.search("warp");
        assert!(results.iter().any(|r| r.command.id == "warp_toggle"));
    }

    #[test]
    fn test_fuzzy_search_empty_returns_all() {
        let mut engine = PaletteEngine::new();
        let results = engine.search("");
        assert_eq!(results.len(), engine.commands().len());
    }

    #[test]
    fn test_fuzzy_search_matching_security() {
        let mut engine = PaletteEngine::new();
        let results = engine.search("sec");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.command.id == "nav_security"));
        assert!(results.iter().any(|r| r.command.id == "sec_inspect"));
        assert!(!results[0].matched_indices.is_empty() || results[0].score > 0);
    }

    #[test]
    fn test_fuzzy_search_matching_d1_sql() {
        let mut engine = PaletteEngine::new();
        let results = engine.search("sql");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.command.id == "storage_d1_console"));
    }

    #[test]
    fn test_dynamic_command_registration() {
        let mut engine = PaletteEngine::new();
        let custom_cmd = PaletteCommand {
            id: "zone_example_com".to_string(),
            title: "Switch Zone: example.com".to_string(),
            description: "Direct jump to example.com zone overview".to_string(),
            category: CommandCategory::Navigation,
            shortcut: None,
            keywords: vec!["example".into(), "zone".into()],
            action: PaletteAction::Custom("switch_zone:example.com".to_string()),
        };

        engine.register(custom_cmd);
        let results = engine.search("example.com");
        assert!(!results.is_empty());
        assert_eq!(results[0].command.id, "zone_example_com");
    }

    #[test]
    fn test_palette_theme_commands() {
        let mut engine = PaletteEngine::new();
        let results = engine.search("theme");
        assert!(results.len() >= 4);
        assert!(results.iter().any(|r| r.command.id == "theme_catppuccin"));
        assert!(results.iter().any(|r| r.command.id == "theme_tokyo_night"));
        assert!(results.iter().any(|r| r.command.id == "theme_nord"));
    }

    #[test]
    fn test_palette_auth_commands() {
        let mut engine = PaletteEngine::new();
        let oauth_res = engine.search("oauth");
        assert!(!oauth_res.is_empty());
        assert!(oauth_res.iter().any(|r| r.command.id == "auth_oauth"));

        let logout_res = engine.search("logout");
        assert!(!logout_res.is_empty());
        assert!(logout_res.iter().any(|r| r.command.id == "auth_logout"));
    }

    #[test]
    fn test_palette_domain_commands() {
        let mut engine = PaletteEngine::new();
        let dom_res = engine.search("registrar");
        assert!(!dom_res.is_empty());
        assert!(dom_res.iter().any(|r| r.command.id == "nav_domains"));
        assert!(dom_res.iter().any(|r| r.command.id == "domain_register"));
        assert!(dom_res
            .iter()
            .any(|r| r.command.id == "domain_toggle_autorenew"));
        assert!(dom_res.iter().any(|r| r.command.id == "domain_toggle_lock"));
    }

    #[test]
    fn test_palette_fast_sync_and_settings_commands() {
        let mut engine = PaletteEngine::new();
        let fast_res = engine.search("fast sync");
        assert!(!fast_res.is_empty());
        assert!(fast_res.iter().any(|r| r.command.id == "cf_fast_sync"));
        assert!(fast_res
            .iter()
            .any(|r| r.command.id == "sys_fast_sync_toggle"));

        let interval_res = engine.search("interval");
        assert!(!interval_res.is_empty());
        assert!(interval_res
            .iter()
            .any(|r| r.command.id == "sys_sync_interval_cycle"));
    }
}
