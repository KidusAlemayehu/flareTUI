use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::cloudflare::{
    auth::AuthManager,
    client::{CloudflareClient, SettingValue},
    export::ZoneExporter,
    oauth::{
        build_auth_url, generate_code_challenge, generate_code_verifier, generate_state,
        listen_for_callback, open_browser,
    },
    sync::CloudflareSync,
    tail::TailStream,
};
use crate::components::{
    auth_modal::{AuthMode, AuthModal, OAuthStatus},
    confirm_modal::ConfirmDeleteModal,
    dns_modal::DnsModal,
    header::Header,
    help::HelpModal,
    pages_deployments::PagesDeploymentsModal,
    pages_logs_modal::PagesLogsModal,
    pages_rollback_modal::ConfirmRollbackModal,
    status_bar::StatusBar,
    views::{
        dns_view::{DnsFocus, DnsView},
        pages_view::PagesView,
        security_view::SecurityView,
        storage_view::StorageView,
        tail_view::TailView,
        workers_view::WorkersView,
    },
    worker_inspector::WorkerInspectorModal,
    tail_log_modal::TailLogModal,
    zone_inspector::ZoneInspectorModal,
    d1_console_modal::D1ConsoleModal,
    kv_modal::KvModal,
    security_inspector::SecurityInspectorModal,
    command_palette_modal::CommandPaletteModal,
    profile_modal::ProfileModal,
};
use crate::db::{CacheDb, DnsRecord};
use crate::palette::{PaletteAction, PaletteEngine};
use crate::theme::{Theme, ThemePreset};

pub struct App {
    pub theme: Theme,
    pub header: Header,
    pub status_bar: StatusBar,
    pub auth_modal: AuthModal,
    pub zone_inspector: Option<ZoneInspectorModal>,
    pub worker_inspector: Option<WorkerInspectorModal>,
    pub tail_log_modal: Option<TailLogModal>,
    pub dns_modal: Option<DnsModal>,
    pub delete_modal: Option<ConfirmDeleteModal>,
    pub pages_deployments: Option<PagesDeploymentsModal>,
    pub pages_rollback_modal: Option<ConfirmRollbackModal>,
    pub pages_logs_modal: Option<PagesLogsModal>,
    pub d1_console_modal: Option<D1ConsoleModal>,
    pub kv_modal: Option<KvModal>,
    pub security_inspector: Option<SecurityInspectorModal>,
    pub command_palette_modal: Option<CommandPaletteModal>,
    pub profile_modal: Option<ProfileModal>,
    pub dns_view: DnsView,
    pub workers_view: WorkersView,
    pub pages_view: PagesView,
    pub storage_view: StorageView,
    pub tail_view: TailView,
    pub security_view: SecurityView,
    pub show_help: bool,
    pub show_auth_modal: bool,
    pub should_quit: bool,
    pub tick_counter: usize,
    pub db: CacheDb,
    pub cf_client: Option<CloudflareClient>,
    pub action_tx: UnboundedSender<Action>,
    pub active_tail_cancel: Option<tokio::sync::watch::Sender<bool>>,
    pub active_tail_worker: Option<String>,
    pub active_tail_id: Option<String>,
    pub oauth_cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl App {
    pub fn new(db: CacheDb, action_tx: UnboundedSender<Action>) -> Self {
        let mut header = Header::new();
        let mut cf_client = None;

        // Auto-load token if configured
        if let Some(token) = AuthManager::load_token() {
            let client = CloudflareClient::new(token);
            let config = AuthManager::load_config();
            if let Some(acc_name) = config.account_name {
                header.account_name = acc_name;
            }
            header.is_live = true;
            cf_client = Some(client);

            // Auto-trigger sync on startup
            let _ = action_tx.send(Action::Refresh);
        }

        Self {
            theme: Theme::default(),
            header,
            status_bar: StatusBar::new(),
            auth_modal: AuthModal::new(),
            zone_inspector: None,
            worker_inspector: None,
            tail_log_modal: None,
            dns_modal: None,
            delete_modal: None,
            pages_deployments: None,
            pages_rollback_modal: None,
            pages_logs_modal: None,
            d1_console_modal: None,
            kv_modal: None,
            security_inspector: None,
            command_palette_modal: None,
            profile_modal: None,
            dns_view: DnsView::new(db.clone()),
            workers_view: WorkersView::new(db.clone()),
            pages_view: PagesView::new(db.clone()),
            storage_view: StorageView::new(db.clone()),
            tail_view: TailView::new(),
            security_view: SecurityView::new(db.clone()),
            show_help: false,
            show_auth_modal: false,
            should_quit: false,
            tick_counter: 0,
            db,
            cf_client,
            action_tx,
            active_tail_cancel: None,
            active_tail_worker: None,
            active_tail_id: None,
            oauth_cancel_tx: None,
        }
    }

    pub fn is_search_active(&self) -> bool {
        (self.header.active_tab == 0 && self.dns_view.search_active)
            || (self.header.active_tab == 4 && self.tail_view.search_active)
            || (self.header.active_tab == 5 && self.security_view.is_searching)
    }

    pub fn is_auth_active(&self) -> bool {
        self.show_auth_modal
    }

    pub fn is_inspector_active(&self) -> bool {
        self.zone_inspector.is_some()
    }

    pub fn is_worker_inspector_active(&self) -> bool {
        self.worker_inspector.is_some()
    }

    pub fn is_tail_log_details_active(&self) -> bool {
        self.tail_log_modal.is_some()
    }

    pub fn is_dns_modal_active(&self) -> bool {
        self.dns_modal.is_some()
    }

    pub fn is_delete_modal_active(&self) -> bool {
        self.delete_modal.is_some()
    }

    pub fn is_pages_deployments_active(&self) -> bool {
        self.pages_deployments.is_some()
    }

    pub fn is_rollback_modal_active(&self) -> bool {
        self.pages_rollback_modal.is_some()
    }

    pub fn is_pages_logs_active(&self) -> bool {
        self.pages_logs_modal.is_some()
    }

    pub fn is_d1_console_active(&self) -> bool {
        self.d1_console_modal.is_some()
    }

    pub fn is_kv_modal_active(&self) -> bool {
        self.kv_modal.is_some()
    }

    pub fn is_command_palette_active(&self) -> bool {
        self.command_palette_modal.is_some()
    }

    pub fn is_security_inspector_active(&self) -> bool {
        self.security_inspector.is_some()
    }

    pub fn is_profile_switcher_active(&self) -> bool {
        self.profile_modal.is_some()
    }

    pub fn is_storage_kv_focused(&self) -> bool {
        self.storage_view.is_kv_focused
    }

    pub fn is_zone_pane_focused(&self) -> bool {
        self.header.active_tab == 0 && self.dns_view.focus == DnsFocus::Zones
    }

    pub fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Tick => {
                self.tick_counter += 1;
                // If not connected to live worker tail, simulate periodic log
                if self.cf_client.is_none() {
                    self.tail_view.add_tick_log(self.tick_counter);
                }
            }
            Action::NextTab => {
                self.header.active_tab = (self.header.active_tab + 1) % self.header.tabs.len();
                if self.header.active_tab == 5 {
                    let active_zone_id = self.dns_view.selected_zone().map(|z| z.id.clone());
                    self.security_view.reload(active_zone_id.as_deref());
                }
                self.status_bar.set_status(format!("Switched to {}", self.header.tabs[self.header.active_tab]));
            }
            Action::PrevTab => {
                if self.header.active_tab == 0 {
                    self.header.active_tab = self.header.tabs.len() - 1;
                } else {
                    self.header.active_tab -= 1;
                }
                if self.header.active_tab == 5 {
                    let active_zone_id = self.dns_view.selected_zone().map(|z| z.id.clone());
                    self.security_view.reload(active_zone_id.as_deref());
                }
                self.status_bar.set_status(format!("Switched to {}", self.header.tabs[self.header.active_tab]));
            }
            Action::SelectTab(idx) => {
                if idx < self.header.tabs.len() {
                    self.header.active_tab = idx;
                    if self.header.active_tab == 5 {
                        let active_zone_id = self.dns_view.selected_zone().map(|z| z.id.clone());
                        self.security_view.reload(active_zone_id.as_deref());
                    }
                    self.status_bar.set_status(format!("Switched to {}", self.header.tabs[idx]));
                }
            }
            Action::NavUp => match self.header.active_tab {
                0 => self.dns_view.nav_up(),
                1 => self.workers_view.nav_up(),
                2 => self.pages_view.nav_up(),
                3 => self.storage_view.nav_up(),
                4 => self.tail_view.nav_up(),
                5 => self.security_view.nav_up(),
                _ => {}
            },
            Action::NavDown => match self.header.active_tab {
                0 => self.dns_view.nav_down(),
                1 => self.workers_view.nav_down(),
                2 => self.pages_view.nav_down(),
                3 => self.storage_view.nav_down(),
                4 => self.tail_view.nav_down(20),
                5 => self.security_view.nav_down(),
                _ => {}
            },
            Action::NavLeft | Action::NavRight => {}
            Action::SwitchPane => match self.header.active_tab {
                0 => self.dns_view.switch_pane(),
                3 => self.storage_view.switch_pane(),
                _ => {}
            },
            Action::ToggleProxy => {
                if self.header.active_tab == 0 {
                    // Update locally
                    if let Some(msg) = self.dns_view.toggle_proxy() {
                        self.status_bar.set_status(msg);
                    }

                    // If live Cloudflare client is available, also dispatch live PATCH request
                    if let Some(client) = &self.cf_client {
                        let filtered = self.dns_view.records.clone();
                        if let Some(rec) = filtered.get(self.dns_view.selected_record_idx) {
                            let client = client.clone();
                            let zone_id = rec.zone_id.clone();
                            let record_id = rec.id.clone();
                            let proxied = rec.proxied;
                            let tx = self.action_tx.clone();

                            tokio::spawn(async move {
                                match client.toggle_dns_proxy(&zone_id, &record_id, proxied).await {
                                    Ok(new_status) => {
                                        let s = if new_status { "🟠 Proxied" } else { "⚪ DNS Only" };
                                        let _ = tx.send(Action::StatusMessage(format!("Cloudflare API: Record set to {s}")));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(format!("Cloudflare API Error: {e}")));
                                    }
                                }
                            });
                        }
                    }
                }
            }
            Action::Refresh => {
                if let Some(client) = &self.cf_client {
                    self.status_bar.set_status("🔄 Syncing live data from Cloudflare v4...");
                    let client = client.clone();
                    let db = self.db.clone();
                    let tx = self.action_tx.clone();

                    tokio::spawn(async move {
                        match CloudflareSync::sync_all(&client, &db).await {
                            Ok(rep) => {
                                let msg = if let Some(w) = &rep.warning {
                                    format!("Synced {} zones, {} records. ⚠️ {}", rep.zones_count, rep.records_count, w)
                                } else {
                                    let mut details = vec![
                                        format!("{} zones", rep.zones_count),
                                        format!("{} records", rep.records_count),
                                        format!("{} workers", rep.workers_count),
                                        format!("{} pages", rep.pages_count),
                                    ];
                                    if rep.d1_count > 0 {
                                        details.push(format!("{} D1 DBs", rep.d1_count));
                                    }
                                    if rep.kv_count > 0 {
                                        details.push(format!("{} KV namespaces", rep.kv_count));
                                    }
                                    if rep.security_events_count > 0 {
                                        details.push(format!("{} security events", rep.security_events_count));
                                    }
                                    if rep.waf_rules_count > 0 {
                                        details.push(format!("{} WAF rules", rep.waf_rules_count));
                                    }
                                    format!("Synced {}", details.join(", "))
                                };
                                let _ = tx.send(Action::SyncCompleted(msg));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Error(format!("Sync error: {e}")));
                            }
                        }
                    });
                } else {
                    self.dns_view.reload_records();
                    self.workers_view.reload();
                    self.pages_view.reload();
                    self.storage_view.reload();
                    let active_zone_id = self.dns_view.selected_zone().map(|z| z.id.clone());
                    self.security_view.reload(active_zone_id.as_deref());
                    self.status_bar.set_status("Cache data reloaded from SQLite");
                }
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
            }
            Action::ToggleSearch => {
                if self.header.active_tab == 0 {
                    self.dns_view.search_active = !self.dns_view.search_active;
                    if !self.dns_view.search_active {
                        self.dns_view.search_query.clear();
                    }
                } else if self.header.active_tab == 4 {
                    self.tail_view.toggle_search();
                } else if self.header.active_tab == 5 {
                    self.security_view.is_searching = !self.security_view.is_searching;
                    if !self.security_view.is_searching {
                        self.security_view.search_query.clear();
                    }
                }
            }
            Action::Logout => {
                let _ = AuthManager::logout();
                self.cf_client = None;
                self.header.is_live = false;
                self.header.account_name = "Logged Out".to_string();
                if let Some(cancel) = self.active_tail_cancel.take() {
                    let _ = cancel.send(true);
                }
                self.active_tail_worker = None;
                self.active_tail_id = None;
                self.status_bar.set_status("Logged out successfully. In offline cached mode (press 'a' to log in).".to_string());
            }
            Action::ToggleAuth => {
                self.show_auth_modal = !self.show_auth_modal;
                if self.show_auth_modal {
                    self.auth_modal.clear();
                } else {
                    if let Some(cancel) = self.oauth_cancel_tx.take() {
                        let _ = cancel.send(());
                    }
                    self.auth_modal.oauth_status = OAuthStatus::Idle;
                }
            }
            Action::AuthInputChar(c) => {
                if c == '\t' {
                    self.auth_modal.cycle_mode();
                } else if self.auth_modal.mode == AuthMode::OAuth {
                    match &self.auth_modal.oauth_status {
                        OAuthStatus::Idle | OAuthStatus::Error(_) => {
                            if c == 'o' || c == 'O' {
                                let _ = self.action_tx.send(Action::StartOAuthLogin);
                            } else if c == '2' {
                                self.auth_modal.switch_mode(AuthMode::Token);
                            }
                        }
                        OAuthStatus::Waiting { .. } => {
                            if c == 'c' || c == 'C' {
                                let _ = self.action_tx.send(Action::AuthCopyOAuthUrl);
                            } else if !c.is_control() {
                                self.auth_modal.handle_char(c);
                            }
                        }
                        _ => {}
                    }
                } else {
                    if c == '1' && self.auth_modal.input_token.is_empty() {
                        self.auth_modal.switch_mode(AuthMode::OAuth);
                    } else {
                        self.auth_modal.handle_char(c);
                    }
                }
            }
            Action::AuthInputBackspace => {
                self.auth_modal.handle_backspace();
            }
            Action::AuthSwitchMode(mode) => {
                self.auth_modal.switch_mode(mode);
            }
            Action::AuthCopyOAuthUrl => {
                self.auth_modal.copy_oauth_url();
                if let Some(fb) = &self.auth_modal.action_feedback {
                    self.status_bar.set_status(fb.clone());
                }
            }
            Action::StartOAuthLogin => {
                self.show_auth_modal = true;
                self.auth_modal.switch_mode(AuthMode::OAuth);
                self.auth_modal.error_msg = None;

                if let Some(cancel) = self.oauth_cancel_tx.take() {
                    let _ = cancel.send(());
                }

                let state = generate_state();
                let verifier = generate_code_verifier();
                let challenge = generate_code_challenge(&verifier);

                let std_listener = std::net::TcpListener::bind("127.0.0.1:8976")
                    .or_else(|_| std::net::TcpListener::bind("127.0.0.1:0"));

                let listener = std_listener.and_then(|l| {
                    l.set_nonblocking(true)?;
                    tokio::net::TcpListener::from_std(l)
                });

                match listener {
                    Ok(l) => {
                        let port = l.local_addr().map(|a| a.port()).unwrap_or(8976);
                        let auth_url = build_auth_url(port, &state, &challenge);

                        self.auth_modal.oauth_status = OAuthStatus::Waiting {
                            port,
                            auth_url: auth_url.clone(),
                            state: state.clone(),
                        };

                        let opened = open_browser(&auth_url);
                        if opened {
                            self.status_bar.set_status("Launched browser for OAuth login. Waiting for authorization...".to_string());
                        } else {
                            self.status_bar.set_status(format!("Listening on port {port}. Open authorization URL to log in."));
                        }

                        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();
                        self.oauth_cancel_tx = Some(cancel_tx);

                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            tokio::select! {
                                res = listen_for_callback(l, state, verifier) => {
                                    match res {
                                        Ok(payload) => {
                                            let _ = tx.send(Action::OAuthSuccess {
                                                token: payload.token,
                                                account_name: payload.account_name,
                                                account_id: payload.account_id,
                                            });
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::OAuthError(e));
                                        }
                                    }
                                }
                                _ = &mut cancel_rx => {
                                    // User cancelled
                                }
                                _ = tokio::time::sleep(tokio::time::Duration::from_secs(300)) => {
                                    let _ = tx.send(Action::OAuthError("OAuth authorization timed out (5 minutes).".to_string()));
                                }
                            }
                        });
                    }
                    Err(e) => {
                        let err = format!("Failed to start loopback server: {e}");
                        self.auth_modal.oauth_status = OAuthStatus::Error(err.clone());
                        self.status_bar.set_error(err);
                    }
                }
            }
            Action::CancelOAuthLogin => {
                if let Some(cancel) = self.oauth_cancel_tx.take() {
                    let _ = cancel.send(());
                }
                self.auth_modal.oauth_status = OAuthStatus::Idle;
                self.status_bar.set_status("OAuth login cancelled.".to_string());
            }
            Action::OAuthSuccess { token, account_name, account_id } => {
                self.oauth_cancel_tx = None;
                let _ = AuthManager::save_token(&token, account_id.clone(), account_name.clone());
                self.cf_client = Some(CloudflareClient::new(token));
                self.header.is_live = true;
                if let Some(name) = &account_name {
                    self.header.account_name = name.clone();
                }
                self.show_auth_modal = false;
                self.auth_modal.clear();
                self.auth_modal.oauth_status = OAuthStatus::Idle;
                self.status_bar.set_status("Authenticated via Browser OAuth! Syncing Cloudflare data...".to_string());
                let _ = self.action_tx.send(Action::Refresh);
            }
            Action::OAuthError(err) => {
                self.oauth_cancel_tx = None;
                self.auth_modal.oauth_status = OAuthStatus::Error(err.clone());
                self.auth_modal.error_msg = Some(err.clone());
                self.status_bar.set_error(format!("OAuth error: {err}"));
            }
            Action::AuthSubmit => {
                if self.auth_modal.mode == AuthMode::OAuth {
                    match &self.auth_modal.oauth_status {
                        OAuthStatus::Idle | OAuthStatus::Error(_) => {
                            let _ = self.action_tx.send(Action::StartOAuthLogin);
                        }
                        OAuthStatus::Waiting { .. } => {
                            let fallback = self.auth_modal.input_token.trim().to_string();
                            if !fallback.is_empty() {
                                let _ = self.action_tx.send(Action::OAuthSuccess {
                                    token: fallback,
                                    account_name: Some("Manual (OAuth Fallback)".to_string()),
                                    account_id: None,
                                });
                            }
                        }
                        _ => {}
                    }
                    return Ok(());
                }

                let token = self.auth_modal.input_token.trim().to_string();
                if token.is_empty() {
                    self.auth_modal.error_msg = Some("Token cannot be empty".to_string());
                    return Ok(());
                }

                self.auth_modal.is_validating = true;
                let client = CloudflareClient::new(token.clone());
                let db = self.db.clone();
                let tx = self.action_tx.clone();

                tokio::spawn(async move {
                    match client.verify_token().await {
                        Ok(_) => {
                            let accounts = client.get_accounts().await.unwrap_or_default();
                            let account_name = accounts.first().map(|a| a.name.clone());
                            let account_id = accounts.first().map(|a| a.id.clone());
                            let _ = AuthManager::save_token(&token, account_id.clone(), account_name.clone());

                            let _ = tx.send(Action::StatusMessage(format!(
                                "Authenticated as: {}",
                                account_name.as_deref().unwrap_or("Personal Account")
                            )));
                            let _ = tx.send(Action::SyncStarted);

                            // Initial sync
                            match CloudflareSync::sync_all(&client, &db).await {
                                Ok(rep) => {
                                    let msg = if let Some(w) = &rep.warning {
                                        format!("Connected! ⚠️ {}", w)
                                    } else {
                                        format!(
                                            "Connected! Synced {} zones, {} DNS records, {} workers",
                                            rep.zones_count, rep.records_count, rep.workers_count
                                        )
                                    };
                                    let _ = tx.send(Action::SyncCompleted(msg));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!("Sync notice: {e}")));
                                }
                            }

                            // Automatically attach live tail stream to primary worker
                            if let Some(worker) = db.get_workers().ok().and_then(|w| w.first().cloned()) {
                                let _ = tx.send(Action::AttachWorkerTail(worker.name));
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(format!("Authentication failed: {e}")));
                        }
                    }
                });
            }
            Action::SyncStarted => {
                self.status_bar.set_status("🔄 Syncing live data from Cloudflare v4...");
            }
            Action::SyncCompleted(msg) => {
                self.show_auth_modal = false;
                self.header.is_live = true;
                if let Some(token) = AuthManager::load_token() {
                    self.cf_client = Some(CloudflareClient::new(token));
                }
                let config = AuthManager::load_config();
                if let Some(name) = config.account_name {
                    self.header.account_name = name;
                }

                // Refresh view models
                self.dns_view.zones = self.db.get_zones().unwrap_or_default();
                if msg.contains("⚠️") {
                    self.dns_view.set_sync_warning(Some(msg.clone()));
                } else {
                    self.dns_view.set_sync_warning(None);
                }
                self.dns_view.reload_records();
                self.workers_view.reload();
                self.pages_view.reload();
                self.storage_view.reload();
                let active_zone_id = self.dns_view.selected_zone().map(|z| z.id.clone());
                self.security_view.reload(active_zone_id.as_deref());
                self.status_bar.set_status(msg);
            }
            Action::TailLog(entry) => {
                if !self.tail_view.is_paused {
                    self.tail_view.logs.push(entry);
                    if self.tail_view.logs.len() > 500 {
                        self.tail_view.logs.remove(0);
                    }
                }
            }
            Action::AttachWorkerTail(mut worker_name) => {
                if worker_name.is_empty() {
                    if let Some(w) = self.workers_view.selected_worker() {
                        worker_name = w.name.clone();
                    } else {
                        self.status_bar.set_status("No worker selected to attach Live Tail");
                        return Ok(());
                    }
                }
                self.worker_inspector = None;

                // 1. Terminate old tail session if active
                if let Some(cancel) = self.active_tail_cancel.take() {
                    let _ = cancel.send(true);
                }
                if let (Some(client), Some(tail_id), Some(old_worker)) = (
                    &self.cf_client,
                    self.active_tail_id.take(),
                    self.active_tail_worker.take(),
                ) {
                    let client = client.clone();
                    let config = AuthManager::load_config();
                    tokio::spawn(async move {
                        let account_id = match config.account_id {
                            Some(id) => id,
                            None => {
                                let accounts = client.get_accounts().await.unwrap_or_default();
                                accounts.first().map(|a| a.id.clone()).unwrap_or_default()
                            }
                        };
                        if !account_id.is_empty() {
                            let _ = client.delete_worker_tail(&account_id, &old_worker, &tail_id).await;
                        }
                    });
                }

                self.tail_view.target_worker = worker_name.clone();
                self.header.active_tab = 4;
                self.status_bar.set_status(format!("Connecting Live Tail to {}...", worker_name));

                self.tail_view.logs.push(crate::components::views::tail_view::LogEntry {
                    timestamp: crate::cloudflare::tail::chrono_or_simple_time(),
                    method: "ATTACH".to_string(),
                    url: format!("/{}", worker_name),
                    status: 101,
                    duration_ms: 0,
                    message: format!("Initiating live WebSocket tail session for '{worker_name}'..."),
                    raw_json: None,
                    exceptions: Vec::new(),
                    log_level: Some("info".to_string()),
                    outcome: "connecting".to_string(),
                });

                if let Some(client) = &self.cf_client {
                    let client = client.clone();
                    let config = AuthManager::load_config();
                    let tx = self.action_tx.clone();
                    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
                    self.active_tail_cancel = Some(cancel_tx);
                    self.active_tail_worker = Some(worker_name.clone());
                    let w_name = worker_name.clone();

                    tokio::spawn(async move {
                        let account_id = match config.account_id {
                            Some(id) => id,
                            None => {
                                let accounts = client.get_accounts().await.unwrap_or_default();
                                accounts.first().map(|a| a.id.clone()).unwrap_or_default()
                            }
                        };

                        if !account_id.is_empty() {
                            match client.create_worker_tail(&account_id, &w_name).await {
                                Ok(tail_res) => {
                                    let _ = tx.send(Action::TailSessionStarted {
                                        worker: w_name.clone(),
                                        tail_id: tail_res.id.clone(),
                                    });
                                    let (tail_tx, mut tail_rx) = tokio::sync::mpsc::unbounded_channel();
                                    let action_tx = tx.clone();
                                    tokio::spawn(async move {
                                        while let Some(log) = tail_rx.recv().await {
                                            let _ = action_tx.send(Action::TailLog(log));
                                        }
                                    });
                                    if let Err(e) = TailStream::start(tail_res.url, tail_tx, cancel_rx).await {
                                        let _ = tx.send(Action::Error(format!("Tail stream error: {e}")));
                                    } else {
                                        let _ = tx.send(Action::StatusMessage(format!("Connected to live tail for {w_name}!")));
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!("Failed to create tail session: {e}")));
                                }
                            }
                        }
                    });
                }
            }
            Action::TailSessionStarted { worker, tail_id } => {
                self.active_tail_worker = Some(worker.clone());
                self.active_tail_id = Some(tail_id);
                self.status_bar.set_status(format!("● Live Tail streaming for {worker}"));
            }
            Action::StopWorkerTail => {
                if let Some(cancel) = self.active_tail_cancel.take() {
                    let _ = cancel.send(true);
                }
                if let (Some(client), Some(tail_id), Some(worker)) = (
                    &self.cf_client,
                    self.active_tail_id.take(),
                    self.active_tail_worker.take(),
                ) {
                    let client = client.clone();
                    let config = AuthManager::load_config();
                    tokio::spawn(async move {
                        let account_id = match config.account_id {
                            Some(id) => id,
                            None => {
                                let accounts = client.get_accounts().await.unwrap_or_default();
                                accounts.first().map(|a| a.id.clone()).unwrap_or_default()
                            }
                        };
                        if !account_id.is_empty() {
                            let _ = client.delete_worker_tail(&account_id, &worker, &tail_id).await;
                        }
                    });
                }
                self.status_bar.set_status("Stopped live tail session");
            }
            Action::ToggleTailPause => {
                self.tail_view.toggle_pause();
                let state = if self.tail_view.is_paused { "PAUSED" } else { "RESUMED" };
                self.status_bar.set_status(format!("Live tail stream {state}"));
            }
            Action::ClearTailLogs => {
                self.tail_view.clear();
                self.status_bar.set_status("Live tail log buffer cleared");
            }
            Action::ToggleTailFollow => {
                self.tail_view.toggle_follow();
                let mode = if self.tail_view.follow_mode { "FOLLOW (auto-scroll enabled)" } else { "MANUAL (scroll history)" };
                self.status_bar.set_status(format!("Live tail mode: {mode}"));
            }
            Action::TailNavUp => {
                self.tail_view.nav_up();
            }
            Action::TailNavDown => {
                self.tail_view.nav_down(20);
            }
            Action::OpenTailLogDetails => {
                if self.header.active_tab == 4 {
                    if let Some(entry) = self.tail_view.selected_log().cloned() {
                        self.tail_log_modal = Some(TailLogModal::new(entry));
                    }
                }
            }
            Action::CloseTailLogDetails => {
                self.tail_log_modal = None;
            }
            Action::TailLogDetailsScrollUp => {
                if let Some(modal) = &mut self.tail_log_modal {
                    modal.scroll_up();
                }
            }
            Action::TailLogDetailsScrollDown => {
                if let Some(modal) = &mut self.tail_log_modal {
                    modal.scroll_down();
                }
            }

            // Worker Inspector Actions
            Action::OpenWorkerInspector => {
                if self.header.active_tab == 1 {
                    if let Some(w) = self.workers_view.selected_worker().cloned() {
                        let mut inspector = WorkerInspectorModal::new(
                            w.name.clone(),
                            w.status.clone(),
                            w.compatibility_date.clone(),
                        );

                        // 1. Load cached bindings from SQLite
                        if let Ok(cached) = self.db.get_worker_bindings(&w.name) {
                            inspector.bindings = cached;
                        }

                        // 2. Fetch live metadata, bindings, and routes from Cloudflare if connected
                        if let Some(client) = &self.cf_client {
                            inspector.is_loading = true;
                            let client = client.clone();
                            let config = AuthManager::load_config();
                            let tx = self.action_tx.clone();
                            let script_name = w.name.clone();

                            tokio::spawn(async move {
                                let account_id = match config.account_id {
                                    Some(id) => id,
                                    None => {
                                        let accounts = client.get_accounts().await.unwrap_or_default();
                                        accounts.first().map(|a| a.id.clone()).unwrap_or_default()
                                    }
                                };

                                if !account_id.is_empty() {
                                    let bindings_res = client.get_worker_bindings(&account_id, &script_name).await.unwrap_or_default();
                                    let routes_res = client.get_worker_routes(&account_id, &script_name).await.unwrap_or_default();
                                    let workers_list = client.get_workers(&account_id).await.unwrap_or_default();
                                    let worker_meta = workers_list.into_iter().find(|item| item.id == script_name);

                                    let (flags, usage_model, handlers) = if let Some(meta) = worker_meta {
                                        (
                                            meta.compatibility_flags.unwrap_or_default(),
                                            meta.usage_model.unwrap_or_else(|| "standard".to_string()),
                                            meta.handlers.unwrap_or_else(|| vec!["fetch".to_string()]),
                                        )
                                    } else {
                                        (vec!["nodejs_compat".to_string()], "standard".to_string(), vec!["fetch".to_string()])
                                    };

                                    let cached_bindings: Vec<crate::db::CachedWorkerBinding> = bindings_res
                                        .into_iter()
                                        .map(|b| {
                                            let details = if let Some(db_id) = b.database_id {
                                                format!("D1 DB ({db_id})")
                                            } else if let Some(ns_id) = b.namespace_id {
                                                format!("KV Namespace ({ns_id})")
                                            } else if let Some(bucket) = b.bucket_name {
                                                format!("R2 Bucket ({bucket})")
                                            } else if let Some(q) = b.queue_name {
                                                format!("Queue ({q})")
                                            } else if let Some(t) = b.text {
                                                format!("Text: {t}")
                                            } else {
                                                "Configured".to_string()
                                            };
                                            crate::db::CachedWorkerBinding {
                                                script_name: script_name.clone(),
                                                name: b.name,
                                                binding_type: b.binding_type,
                                                details,
                                            }
                                        })
                                        .collect();

                                    let _ = tx.send(Action::WorkerDetailsLoaded {
                                        flags,
                                        usage_model,
                                        handlers,
                                        routes: routes_res,
                                        bindings: cached_bindings,
                                    });
                                }
                            });
                        }

                        self.worker_inspector = Some(inspector);
                    }
                }
            }
            Action::CloseWorkerInspector => {
                self.worker_inspector = None;
            }
            Action::WorkerInspectorNavUp => {
                if let Some(inspector) = &mut self.worker_inspector {
                    inspector.nav_up();
                }
            }
            Action::WorkerInspectorNavDown => {
                if let Some(inspector) = &mut self.worker_inspector {
                    inspector.nav_down();
                }
            }
            Action::WorkerDetailsLoaded {
                flags,
                usage_model,
                handlers,
                routes,
                bindings,
            } => {
                if let Some(inspector) = &mut self.worker_inspector {
                    let _ = self.db.replace_worker_bindings(&inspector.worker_name, &bindings);
                    inspector.compatibility_flags = flags;
                    inspector.usage_model = usage_model;
                    inspector.handlers = handlers;
                    inspector.routes = routes;
                    inspector.bindings = bindings;
                    inspector.is_loading = false;
                    inspector.action_feedback = Some("Live metadata & bindings loaded from Cloudflare".to_string());
                }
            }

            // D1 SQL Console Actions
            Action::OpenD1Console => {
                if let Some(db) = self.storage_view.selected_d1().cloned() {
                    let tables = self.db.get_d1_tables(&db.uuid).unwrap_or_default();
                    let console = D1ConsoleModal::new(db.uuid.clone(), db.name.clone(), tables);

                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let db_id = db.uuid.clone();
                        let cache_db = self.db.clone();

                        tokio::spawn(async move {
                            if let Ok(accounts) = client.get_accounts().await {
                                if let Some(acc) = accounts.first() {
                                    let schema_query = "SELECT name, type, sql FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_cf_%' ORDER BY name;";
                                    if let Ok(results) = client.execute_d1_query(&acc.id, &db_id, schema_query).await {
                                        if let Some(first_res) = results.into_iter().next() {
                                            if let Some(rows) = &first_res.results {
                                                let mut fetched_tables = Vec::new();
                                                for r in rows {
                                                    if let serde_json::Value::Object(map) = r {
                                                        if let Some(serde_json::Value::String(name)) = map.get("name") {
                                                            let sql = map.get("sql").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                            fetched_tables.push(crate::db::CachedD1Table {
                                                                database_id: db_id.clone(),
                                                                table_name: name.clone(),
                                                                schema_sql: sql,
                                                                row_count: 0,
                                                            });
                                                        }
                                                    }
                                                }
                                                if !fetched_tables.is_empty() {
                                                    let _ = cache_db.replace_d1_tables(&db_id, &fetched_tables);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }

                    self.d1_console_modal = Some(console);
                } else {
                    self.status_bar.set_status("No D1 database selected to open console");
                }
            }
            Action::CloseD1Console => {
                self.d1_console_modal = None;
            }
            Action::D1ConsoleSwitchPane => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.switch_pane();
                }
            }
            Action::D1ConsoleNavUp => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.nav_up();
                }
            }
            Action::D1ConsoleNavDown => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.nav_down();
                }
            }
            Action::D1ConsoleNavLeft => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.nav_left();
                }
            }
            Action::D1ConsoleNavRight => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.nav_right();
                }
            }
            Action::D1ConsolePageUp => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.page_up();
                }
            }
            Action::D1ConsolePageDown => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.page_down();
                }
            }
            Action::D1ConsoleHome => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.nav_home();
                }
            }
            Action::D1ConsoleEnd => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.nav_end();
                }
            }
            Action::D1ConsoleInputChar(c) => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.input_char(c);
                }
            }
            Action::D1ConsoleInputBackspace => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.input_backspace();
                }
            }
            Action::D1ConsoleInputDelete => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.input_delete();
                }
            }
            Action::D1ConsoleClearQuery => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.clear_query();
                }
            }
            Action::D1ConsoleApplyPreset(idx) => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.apply_preset(idx);
                }
            }
            Action::D1ConsoleExecuteQuery => {
                if let Some(m) = &mut self.d1_console_modal {
                    let query = m.query_input.clone();
                    m.record_query_history(&query);
                    m.is_executing = true;
                    m.error_message = None;

                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let tx = self.action_tx.clone();
                        let db_id = m.database_id.clone();
                        let query_str = query.clone();

                        tokio::spawn(async move {
                            match client.get_accounts().await {
                                Ok(accounts) => {
                                    if let Some(acc) = accounts.first() {
                                        match client.execute_d1_query(&acc.id, &db_id, &query_str).await {
                                            Ok(results) => {
                                                if let Some(first_res) = results.into_iter().next() {
                                                    let _ = tx.send(Action::D1QueryResultLoaded(first_res));
                                                } else {
                                                    let _ = tx.send(Action::D1QueryResultLoaded(crate::cloudflare::client::D1QueryResult::default()));
                                                }
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Action::D1QueryExecutionFailed(e.to_string()));
                                            }
                                        }
                                    } else {
                                        let _ = tx.send(Action::D1QueryExecutionFailed("No account associated with token".to_string()));
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::D1QueryExecutionFailed(e.to_string()));
                                }
                            }
                        });
                    } else {
                        // Offline simulation mode
                        let sim_result = crate::cloudflare::client::D1QueryResult {
                            results: Some(vec![
                                serde_json::json!({
                                    "status": "offline_mode",
                                    "database": m.database_name.clone(),
                                    "query_executed": query,
                                    "note": "Connect Cloudflare API token with 'a' for live edge query execution"
                                }),
                            ]),
                            success: Some(true),
                            meta: Some(crate::cloudflare::client::D1QueryMeta {
                                duration: Some(0.35),
                                rows_read: Some(1),
                                rows_written: Some(0),
                                changes: Some(0),
                                last_row_id: None,
                            }),
                        };
                        let _ = self.action_tx.send(Action::D1QueryResultLoaded(sim_result));
                    }
                }
            }
            Action::D1QueryResultLoaded(results) => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.set_result(results);
                }
            }
            Action::D1QueryExecutionFailed(err) => {
                if let Some(m) = &mut self.d1_console_modal {
                    m.set_error(err);
                }
            }

            // KV Inspector Actions
            Action::OpenKvModal => {
                if let Some(kv) = self.storage_view.selected_kv().cloned() {
                    let mut modal = KvModal::new(kv.id.clone(), kv.title.clone(), Vec::new());
                    modal.is_loading = true;

                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let tx = self.action_tx.clone();
                        let ns_id = kv.id.clone();

                        tokio::spawn(async move {
                            if let Ok(accounts) = client.get_accounts().await {
                                if let Some(acc) = accounts.first() {
                                    match client.get_kv_keys(&acc.id, &ns_id, None, None).await {
                                        Ok(keys) => {
                                            let _ = tx.send(Action::KvModalValueLoaded(format!("__KEYS_LOADED__:{}", serde_json::to_string(&keys).unwrap_or_default())));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::KvModalValueFailed(e.to_string()));
                                        }
                                    }
                                }
                            }
                        });
                    } else {
                        // Offline simulation keys
                        let sample_keys = vec![
                            crate::cloudflare::client::KvKeyResult {
                                name: "app:config".to_string(),
                                expiration: None,
                                metadata: None,
                            },
                            crate::cloudflare::client::KvKeyResult {
                                name: "session:usr_99182".to_string(),
                                expiration: Some(1735689600),
                                metadata: None,
                            },
                            crate::cloudflare::client::KvKeyResult {
                                name: "ratelimit:ip_192_168_1_1".to_string(),
                                expiration: Some(1735693200),
                                metadata: None,
                            },
                        ];
                        modal.set_keys(sample_keys);
                        modal.is_loading = false;
                    }

                    self.kv_modal = Some(modal);
                } else {
                    self.status_bar.set_status("No KV namespace selected to inspect");
                }
            }
            Action::CloseKvModal => {
                self.kv_modal = None;
            }
            Action::KvModalSwitchPane => {
                if let Some(m) = &mut self.kv_modal {
                    m.switch_pane();
                }
            }
            Action::KvModalNavUp => {
                if let Some(m) = &mut self.kv_modal {
                    m.nav_up();
                }
            }
            Action::KvModalNavDown => {
                if let Some(m) = &mut self.kv_modal {
                    m.nav_down();
                }
            }
            Action::KvModalInputChar(c) => {
                if let Some(m) = &mut self.kv_modal {
                    m.filter_input_char(c);
                }
            }
            Action::KvModalInputBackspace => {
                if let Some(m) = &mut self.kv_modal {
                    m.filter_input_backspace();
                }
            }
            Action::KvModalClearFilter => {
                if let Some(m) = &mut self.kv_modal {
                    m.clear_filter();
                }
            }
            Action::KvModalFetchValue => {
                if let Some(m) = &mut self.kv_modal {
                    if let Some(key) = m.selected_key() {
                        let key_name = key.name.clone();
                        let ns_id = m.namespace_id.clone();
                        m.is_loading = true;
                        m.error_message = None;

                        if let Some(client) = &self.cf_client {
                            let client = client.clone();
                            let tx = self.action_tx.clone();

                            tokio::spawn(async move {
                                if let Ok(accounts) = client.get_accounts().await {
                                    if let Some(acc) = accounts.first() {
                                        match client.get_kv_value(&acc.id, &ns_id, &key_name).await {
                                            Ok(val) => {
                                                let _ = tx.send(Action::KvModalValueLoaded(val));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Action::KvModalValueFailed(e.to_string()));
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            // Offline sample formatted JSON
                            let sample_json = serde_json::json!({
                                "key": key_name,
                                "environment": "production",
                                "features": {
                                    "d1_edge_cache": true,
                                    "streaming_ssr": true,
                                    "max_concurrency": 250
                                },
                                "metadata": {
                                    "source": "offline_simulated",
                                    "version": "1.0.4"
                                }
                            });
                            let formatted = serde_json::to_string_pretty(&sample_json).unwrap_or_default();
                            m.set_value(formatted);
                        }
                    }
                }
            }
            Action::KvModalValueLoaded(val) => {
                if let Some(m) = &mut self.kv_modal {
                    if let Some(stripped) = val.strip_prefix("__KEYS_LOADED__:") {
                        if let Ok(keys) = serde_json::from_str::<Vec<crate::cloudflare::client::KvKeyResult>>(stripped) {
                            m.set_keys(keys);
                            m.is_loading = false;
                        }
                    } else {
                        m.set_value(val);
                    }
                }
            }
            Action::KvModalValueFailed(err) => {
                if let Some(m) = &mut self.kv_modal {
                    m.set_error(err);
                }
            }
            Action::InputChar(c) => {
                if self.header.active_tab == 0 && self.dns_view.search_active {
                    self.dns_view.search_query.push(c);
                } else if self.header.active_tab == 4 && self.tail_view.search_active {
                    self.tail_view.input_char(c);
                } else if self.header.active_tab == 5 && self.security_view.is_searching {
                    self.security_view.search_query.push(c);
                    self.security_view.selected_event_idx = 0;
                }
            }
            Action::InputBackspace => {
                if self.header.active_tab == 0 && self.dns_view.search_active {
                    self.dns_view.search_query.pop();
                } else if self.header.active_tab == 4 && self.tail_view.search_active {
                    self.tail_view.input_backspace();
                } else if self.header.active_tab == 5 && self.security_view.is_searching {
                    self.security_view.search_query.pop();
                    self.security_view.selected_event_idx = 0;
                }
            }
            Action::InputSubmit => {
                if self.header.active_tab == 0 {
                    self.dns_view.search_active = false;
                } else if self.header.active_tab == 4 {
                    self.tail_view.search_active = false;
                } else if self.header.active_tab == 5 {
                    self.security_view.is_searching = false;
                }
            }
            Action::ClearSearch => {
                if self.header.active_tab == 0 {
                    self.dns_view.search_query.clear();
                    self.dns_view.search_active = false;
                } else if self.header.active_tab == 4 {
                    self.tail_view.clear_search();
                } else if self.header.active_tab == 5 {
                    self.security_view.search_query.clear();
                    self.security_view.is_searching = false;
                }
            }
            Action::StatusMessage(msg) => {
                self.status_bar.set_status(msg);
            }
            Action::Error(err) => {
                self.auth_modal.is_validating = false;
                self.auth_modal.error_msg = Some(err.clone());
                self.status_bar.set_error(err.clone());
                if err.contains("DNS") || err.contains("Authentication") || err.contains("10000") {
                    self.dns_view.set_sync_warning(Some(err));
                }
            }

            // Zone Inspector
            Action::OpenZoneInspector => {
                if self.header.active_tab == 0 {
                    if let Some(zone) = self.dns_view.selected_zone().cloned() {
                        let mut inspector = ZoneInspectorModal::new(zone.clone());
                        if let Some(client) = &self.cf_client {
                            inspector.is_loading = true;
                            let client = client.clone();
                            let zone_id = zone.id.clone();
                            let tx = self.action_tx.clone();
                            tokio::spawn(async move {
                                match client.get_zone_settings(&zone_id).await {
                                    Ok(settings) => {
                                        let _ = tx.send(Action::ZoneSettingsLoaded(settings));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(format!("Zone settings error: {e}")));
                                    }
                                }
                            });
                        }
                        self.zone_inspector = Some(inspector);
                    }
                }
            }
            Action::CloseZoneInspector => {
                self.zone_inspector = None;
            }
            Action::ZoneSettingsLoaded(settings) => {
                if let Some(inspector) = &mut self.zone_inspector {
                    inspector.settings = settings;
                    inspector.is_loading = false;
                    inspector.action_feedback = Some("Live settings loaded from Cloudflare".to_string());
                }
            }
            Action::CycleZoneSsl => {
                if let Some(inspector) = &mut self.zone_inspector {
                    let cur = inspector.settings.ssl.as_ref().map(|s| s.value.as_str()).unwrap_or("flexible");
                    let next_mode = match cur {
                        "off" => "flexible",
                        "flexible" => "full",
                        "full" => "strict",
                        "strict" => "off",
                        _ => "flexible",
                    };
                    inspector.settings.ssl = Some(SettingValue { value: next_mode.to_string() });
                    inspector.action_feedback = Some(format!("SSL mode updated to {}", next_mode.to_uppercase()));
                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let zone_id = inspector.zone.id.clone();
                        let mode = next_mode.to_string();
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            match client.set_zone_ssl(&zone_id, &mode).await {
                                Ok(_) => {
                                    let _ = tx.send(Action::StatusMessage(format!("SSL mode set to {}", mode.to_uppercase())));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!("Cloudflare API error: {e}")));
                                }
                            }
                        });
                    }
                }
            }
            Action::ToggleZoneDevMode => {
                if let Some(inspector) = &mut self.zone_inspector {
                    let cur = inspector.settings.development_mode.as_ref().map(|s| s.value.as_str()).unwrap_or("off");
                    let enabled = cur != "on";
                    let next_str = if enabled { "on" } else { "off" };
                    inspector.settings.development_mode = Some(SettingValue { value: next_str.to_string() });
                    inspector.action_feedback = Some(format!("Dev mode toggled {}", next_str.to_uppercase()));
                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let zone_id = inspector.zone.id.clone();
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            match client.set_zone_dev_mode(&zone_id, enabled).await {
                                Ok(_) => {
                                    let _ = tx.send(Action::StatusMessage(format!("Dev Mode set to {}", next_str.to_uppercase())));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!("Cloudflare API error: {e}")));
                                }
                            }
                        });
                    }
                }
            }
            Action::ToggleZoneUnderAttack => {
                if let Some(inspector) = &mut self.zone_inspector {
                    let cur = inspector.settings.security_level.as_ref().map(|s| s.value.as_str()).unwrap_or("medium");
                    let next_level = if cur == "under_attack" { "medium" } else { "under_attack" };
                    inspector.settings.security_level = Some(SettingValue { value: next_level.to_string() });
                    let label = if next_level == "under_attack" { "UNDER ATTACK MODE ACTIVATED" } else { "Security Level: MEDIUM" };
                    inspector.action_feedback = Some(label.to_string());
                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let zone_id = inspector.zone.id.clone();
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            match client.set_zone_security_level(&zone_id, next_level).await {
                                Ok(_) => {
                                    let _ = tx.send(Action::StatusMessage(format!("Security level set to {}", next_level.to_uppercase())));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!("Cloudflare API error: {e}")));
                                }
                            }
                        });
                    }
                }
            }
            Action::PurgeZoneCache => {
                if let Some(inspector) = &mut self.zone_inspector {
                    inspector.action_feedback = Some("Purging all edge cache...".to_string());
                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let zone_id = inspector.zone.id.clone();
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            match client.purge_zone_cache(&zone_id).await {
                                Ok(_) => {
                                    let _ = tx.send(Action::StatusMessage("Cloudflare edge cache purged!".to_string()));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!("Purge cache error: {e}")));
                                }
                            }
                        });
                    } else {
                        inspector.action_feedback = Some("Purge simulated (no active Cloudflare token)".to_string());
                    }
                }
            }

            // DNS Record Modal Actions
            Action::OpenCreateDnsModal => {
                if self.header.active_tab == 0 {
                    if let Some(zone) = self.dns_view.selected_zone() {
                        self.dns_modal = Some(DnsModal::new_create(zone.id.clone()));
                    }
                }
            }
            Action::OpenEditDnsModal => {
                if self.header.active_tab == 0 {
                    if let Some(rec) = self.dns_view.selected_record() {
                        self.dns_modal = Some(DnsModal::new_edit(
                            rec.zone_id.clone(),
                            rec.id.clone(),
                            rec.record_type.clone(),
                            rec.name.clone(),
                            rec.content.clone(),
                            rec.ttl as u32,
                            rec.proxied,
                        ));
                    }
                }
            }
            Action::CloseDnsModal => {
                self.dns_modal = None;
            }
            Action::DnsModalNextField => {
                if let Some(modal) = &mut self.dns_modal {
                    modal.next_field();
                }
            }
            Action::DnsModalPrevField => {
                if let Some(modal) = &mut self.dns_modal {
                    modal.prev_field();
                }
            }
            Action::DnsModalCycleType(next) => {
                if let Some(modal) = &mut self.dns_modal {
                    modal.cycle_type(next);
                }
            }
            Action::DnsModalInputChar(c) => {
                if let Some(modal) = &mut self.dns_modal {
                    modal.handle_char(c);
                }
            }
            Action::DnsModalInputBackspace => {
                if let Some(modal) = &mut self.dns_modal {
                    modal.handle_backspace();
                }
            }
            Action::DnsModalSubmit => {
                if let Some(modal) = &mut self.dns_modal {
                    if modal.name.trim().is_empty() {
                        modal.error_msg = Some("Record name cannot be empty".to_string());
                        return Ok(());
                    }
                    if modal.content.trim().is_empty() {
                        modal.error_msg = Some("Record content/IP cannot be empty".to_string());
                        return Ok(());
                    }

                    let new_record = modal.to_new_record();
                    let is_edit = modal.is_edit;
                    let record_id = modal.record_id.clone();
                    let zone_id = modal.zone_id.clone();

                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let db = self.db.clone();
                        let tx = self.action_tx.clone();

                        tokio::spawn(async move {
                            if is_edit {
                                if let Some(rec_id) = record_id {
                                    match client.update_dns_record(&zone_id, &rec_id, &new_record).await {
                                        Ok(res) => {
                                            let local_rec = DnsRecord {
                                                id: res.id,
                                                zone_id: res.zone_id.unwrap_or_else(|| zone_id.clone()),
                                                record_type: res.record_type,
                                                name: res.name.clone(),
                                                content: res.content,
                                                proxied: res.proxied,
                                                ttl: res.ttl,
                                            };
                                            let _ = db.upsert_dns_record(&local_rec);
                                            let _ = tx.send(Action::StatusMessage(format!("Updated DNS record {}", res.name)));
                                            let _ = tx.send(Action::CloseDnsModal);
                                            let _ = tx.send(Action::SyncCompleted("Record updated".to_string()));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::Error(format!("Cloudflare API error: {e}")));
                                        }
                                    }
                                }
                            } else {
                                match client.create_dns_record(&zone_id, &new_record).await {
                                    Ok(res) => {
                                        let local_rec = DnsRecord {
                                            id: res.id,
                                            zone_id: res.zone_id.unwrap_or_else(|| zone_id.clone()),
                                            record_type: res.record_type,
                                            name: res.name.clone(),
                                            content: res.content,
                                            proxied: res.proxied,
                                            ttl: res.ttl,
                                        };
                                        let _ = db.upsert_dns_record(&local_rec);
                                        let _ = tx.send(Action::StatusMessage(format!("Created DNS record {}", res.name)));
                                        let _ = tx.send(Action::CloseDnsModal);
                                        let _ = tx.send(Action::SyncCompleted("Record created".to_string()));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(format!("Cloudflare API error: {e}")));
                                    }
                                }
                            }
                        });
                    } else {
                        let id = record_id.unwrap_or_else(|| {
                            format!(
                                "rec_{}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_millis())
                                    .unwrap_or(0)
                            )
                        });
                        let local_rec = DnsRecord {
                            id,
                            zone_id,
                            record_type: new_record.record_type,
                            name: new_record.name.clone(),
                            content: new_record.content,
                            proxied: new_record.proxied,
                            ttl: new_record.ttl,
                        };
                        let _ = self.db.upsert_dns_record(&local_rec);
                        self.dns_modal = None;
                        self.dns_view.reload_records();
                        self.status_bar.set_status(format!("Saved DNS record {}", local_rec.name));
                    }
                }
            }

            // Delete Confirmation Guard
            Action::OpenDeleteDnsModal => {
                if self.header.active_tab == 0 {
                    if let Some(rec) = self.dns_view.selected_record().cloned() {
                        self.delete_modal = Some(ConfirmDeleteModal::new(rec));
                    }
                }
            }
            Action::CloseDeleteDnsModal => {
                self.delete_modal = None;
            }
            Action::ConfirmDeleteDns => {
                if let Some(modal) = self.delete_modal.take() {
                    let rec = modal.record;
                    let _ = self.db.delete_dns_record(&rec.id);
                    self.dns_view.reload_records();
                    self.status_bar.set_status(format!("Deleted DNS record {}", rec.name));

                    if let Some(client) = &self.cf_client {
                        let client = client.clone();
                        let zone_id = rec.zone_id.clone();
                        let rec_id = rec.id.clone();
                        let name = rec.name.clone();
                        let tx = self.action_tx.clone();

                        tokio::spawn(async move {
                            match client.delete_dns_record(&zone_id, &rec_id).await {
                                Ok(_) => {
                                    let _ = tx.send(Action::StatusMessage(format!("Deleted record {name} on Cloudflare edge")));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!("Cloudflare delete failed: {e}")));
                                }
                            }
                        });
                    }
                }
            }

            // Pages Deployments, Rollback, and Logs
            Action::OpenPagesDeployments => {
                if self.header.active_tab == 2 {
                    if let Some(project) = self.pages_view.selected_project().cloned() {
                        let mut modal = PagesDeploymentsModal::new(project.name.clone());

                        // 1. Load cached deployments from SQLite
                        if let Ok(cached) = self.db.get_pages_deployments(&project.name) {
                            if !cached.is_empty() {
                                modal.deployments = cached
                                    .into_iter()
                                    .map(|c| crate::cloudflare::client::PagesDeploymentResult {
                                        id: c.id.clone(),
                                        short_id: Some(if c.id.len() >= 8 { c.id[..8].to_string() } else { c.id }),
                                        project_id: None,
                                        project_name: Some(c.project_name),
                                        environment: Some(c.environment),
                                        url: Some(c.url),
                                        created_on: Some(c.created_on),
                                        modified_on: None,
                                        latest_stage: Some(crate::cloudflare::client::PagesStage { status: c.status }),
                                        deployment_trigger: Some(crate::cloudflare::client::PagesDeploymentTrigger {
                                            trigger_type: Some("push".to_string()),
                                            metadata: Some(crate::cloudflare::client::PagesTriggerMetadata {
                                                branch: Some(c.branch),
                                                commit_hash: Some(c.commit_hash),
                                                commit_message: Some(c.commit_message),
                                            }),
                                        }),
                                    })
                                    .collect();
                            }
                        }

                        // 2. Fetch live deployments from Cloudflare if connected
                        if let Some(client) = &self.cf_client {
                            modal.is_loading = true;
                            let client = client.clone();
                            let config = AuthManager::load_config();
                            let tx = self.action_tx.clone();
                            let proj_name = project.name.clone();

                            tokio::spawn(async move {
                                let account_id = match config.account_id {
                                    Some(id) => id,
                                    None => {
                                        let accounts = client.get_accounts().await.unwrap_or_default();
                                        accounts.first().map(|a| a.id.clone()).unwrap_or_default()
                                    }
                                };

                                if !account_id.is_empty() {
                                    match client.get_pages_deployments(&account_id, &proj_name).await {
                                        Ok(deps) => {
                                            let _ = tx.send(Action::PagesDeploymentsLoaded(deps));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::Error(format!("Failed to load deployments: {e}")));
                                        }
                                    }
                                }
                            });
                        }

                        self.pages_deployments = Some(modal);
                    }
                }
            }
            Action::ClosePagesDeployments => {
                self.pages_deployments = None;
            }
            Action::PagesDeploymentsLoaded(deps) => {
                if let Some(modal) = &mut self.pages_deployments {
                    let cached_records: Vec<crate::db::CachedDeployment> = deps
                        .iter()
                        .map(|d| {
                            let branch = d
                                .deployment_trigger
                                .as_ref()
                                .and_then(|t| t.metadata.as_ref())
                                .and_then(|m| m.branch.clone())
                                .unwrap_or_else(|| "main".to_string());
                            let commit_hash = d
                                .deployment_trigger
                                .as_ref()
                                .and_then(|t| t.metadata.as_ref())
                                .and_then(|m| m.commit_hash.clone())
                                .unwrap_or_default();
                            let commit_msg = d
                                .deployment_trigger
                                .as_ref()
                                .and_then(|t| t.metadata.as_ref())
                                .and_then(|m| m.commit_message.clone())
                                .unwrap_or_default();
                            let status = d
                                .latest_stage
                                .as_ref()
                                .map(|s| s.status.clone())
                                .unwrap_or_else(|| "active".to_string());

                            crate::db::CachedDeployment {
                                id: d.id.clone(),
                                project_name: modal.project_name.clone(),
                                environment: d.environment.clone().unwrap_or_else(|| "preview".to_string()),
                                url: d.url.clone().unwrap_or_default(),
                                branch,
                                commit_hash,
                                commit_message: commit_msg,
                                status,
                                created_on: d.created_on.clone().unwrap_or_default(),
                            }
                        })
                        .collect();
                    let _ = self.db.replace_pages_deployments(&modal.project_name, &cached_records);

                    modal.deployments = deps;
                    modal.is_loading = false;
                    modal.action_feedback = Some("Live deployments synced from Cloudflare".to_string());
                }
            }
            Action::PagesDeploymentsNavUp => {
                if let Some(modal) = &mut self.pages_deployments {
                    modal.nav_up();
                }
            }
            Action::PagesDeploymentsNavDown => {
                if let Some(modal) = &mut self.pages_deployments {
                    modal.nav_down();
                }
            }
            Action::OpenRollbackModal => {
                if let Some(dep_modal) = &self.pages_deployments {
                    if let Some(sel) = dep_modal.selected_deployment() {
                        let short_id = sel.short_id.clone().unwrap_or_else(|| {
                            if sel.id.len() >= 8 {
                                sel.id[..8].to_string()
                            } else {
                                sel.id.clone()
                            }
                        });
                        let branch = sel
                            .deployment_trigger
                            .as_ref()
                            .and_then(|t| t.metadata.as_ref())
                            .and_then(|m| m.branch.clone())
                            .unwrap_or_else(|| "main".to_string());
                        let msg = sel
                            .deployment_trigger
                            .as_ref()
                            .and_then(|t| t.metadata.as_ref())
                            .and_then(|m| m.commit_message.clone())
                            .unwrap_or_else(|| "Automated build".to_string());

                        self.pages_rollback_modal = Some(ConfirmRollbackModal::new(
                            dep_modal.project_name.clone(),
                            sel.id.clone(),
                            short_id,
                            branch,
                            msg,
                        ));
                    }
                }
            }
            Action::CloseRollbackModal => {
                self.pages_rollback_modal = None;
            }
            Action::ConfirmRollback => {
                if let Some(modal) = self.pages_rollback_modal.take() {
                    let proj_name = modal.project_name.clone();
                    let dep_id = modal.deployment_id.clone();
                    let short_id = modal.short_id.clone();

                    if let Some(client) = &self.cf_client {
                        self.status_bar.set_status(format!("Rolling back {} to {}...", proj_name, short_id));
                        let client = client.clone();
                        let config = AuthManager::load_config();
                        let tx = self.action_tx.clone();

                        tokio::spawn(async move {
                            let account_id = match config.account_id {
                                Some(id) => id,
                                None => {
                                    let accounts = client.get_accounts().await.unwrap_or_default();
                                    accounts.first().map(|a| a.id.clone()).unwrap_or_default()
                                }
                            };

                            if !account_id.is_empty() {
                                match client.rollback_pages_deployment(&account_id, &proj_name, &dep_id).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::StatusMessage(format!(
                                            "🚀 Promoted build {} to LIVE production for {}!",
                                            short_id, proj_name
                                        )));
                                        let _ = tx.send(Action::OpenPagesDeployments);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(format!("Rollback error: {e}")));
                                    }
                                }
                            }
                        });
                    } else {
                        self.status_bar.set_status(format!("Rollback simulated (no live API connection)"));
                    }
                }
            }
            Action::OpenPagesLogs => {
                if let Some(dep_modal) = &self.pages_deployments {
                    if let Some(sel) = dep_modal.selected_deployment() {
                        let short_id = sel.short_id.clone().unwrap_or_else(|| {
                            if sel.id.len() >= 8 {
                                sel.id[..8].to_string()
                            } else {
                                sel.id.clone()
                            }
                        });
                        let mut log_modal = PagesLogsModal::new(dep_modal.project_name.clone(), sel.id.clone(), short_id);

                        if let Some(client) = &self.cf_client {
                            let client = client.clone();
                            let config = AuthManager::load_config();
                            let tx = self.action_tx.clone();
                            let proj_name = dep_modal.project_name.clone();
                            let dep_id = sel.id.clone();

                            tokio::spawn(async move {
                                let account_id = match config.account_id {
                                    Some(id) => id,
                                    None => {
                                        let accounts = client.get_accounts().await.unwrap_or_default();
                                        accounts.first().map(|a| a.id.clone()).unwrap_or_default()
                                    }
                                };

                                if !account_id.is_empty() {
                                    match client.get_pages_build_logs(&account_id, &proj_name, &dep_id).await {
                                        Ok(logs) => {
                                            let _ = tx.send(Action::PagesLogsLoaded(logs));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::Error(format!("Logs error: {e}")));
                                        }
                                    }
                                }
                            });
                        } else {
                            log_modal.is_loading = false;
                        }

                        self.pages_logs_modal = Some(log_modal);
                    }
                }
            }
            Action::ClosePagesLogs => {
                self.pages_logs_modal = None;
            }
            Action::PagesLogsLoaded(logs) => {
                if let Some(modal) = &mut self.pages_logs_modal {
                    modal.logs = logs;
                    modal.is_loading = false;
                }
            }
            Action::PagesLogsScrollUp => {
                if let Some(modal) = &mut self.pages_logs_modal {
                    modal.scroll_up();
                }
            }
            Action::PagesLogsScrollDown => {
                if let Some(modal) = &mut self.pages_logs_modal {
                    modal.scroll_down();
                }
            }

            // Export BIND Zone
            Action::ExportZoneBind => {
                if self.header.active_tab == 0 {
                    if let Some(zone) = self.dns_view.selected_zone() {
                        match ZoneExporter::export_bind_zone(&zone.name, &self.dns_view.records) {
                            Ok(path) => {
                                self.status_bar.set_status(format!("Exported BIND zone: {}", path.display()));
                            }
                            Err(e) => {
                                self.status_bar.set_error(format!("Export error: {e}"));
                            }
                        }
                    }
                }
            }
            // Security & WAF Actions
            Action::OpenSecurityInspector => {
                if self.header.active_tab == 5 {
                    if let Some(event) = self.security_view.selected_event() {
                        let matched_rule = if event.rule_id.is_empty() {
                            None
                        } else {
                            self.security_view
                                .rules
                                .iter()
                                .find(|r| r.id == event.rule_id)
                                .cloned()
                        };
                        self.security_inspector = Some(SecurityInspectorModal::new(event, matched_rule));
                    }
                }
            }
            Action::CloseSecurityInspector => {
                self.security_inspector = None;
            }
            Action::SecurityInspectorNextTab => {
                if let Some(modal) = &mut self.security_inspector {
                    modal.next_tab();
                }
            }
            Action::SecurityInspectorPrevTab => {
                if let Some(modal) = &mut self.security_inspector {
                    modal.prev_tab();
                }
            }
            Action::SecurityInspectorScrollUp => {
                if let Some(modal) = &mut self.security_inspector {
                    modal.scroll_up();
                }
            }
            Action::SecurityInspectorScrollDown => {
                if let Some(modal) = &mut self.security_inspector {
                    modal.scroll_down();
                }
            }
            Action::SecurityInspectorCopyIp => {
                if let Some(modal) = &mut self.security_inspector {
                    modal.copy_ip();
                    if let Some(fb) = &modal.action_feedback {
                        self.status_bar.set_status(fb.clone());
                    }
                }
            }
            Action::SecurityInspectorCopyRayId => {
                if let Some(modal) = &mut self.security_inspector {
                    modal.copy_ray_id();
                    if let Some(fb) = &modal.action_feedback {
                        self.status_bar.set_status(fb.clone());
                    }
                }
            }
            Action::SecurityViewNextTab => {
                if self.header.active_tab == 5 {
                    self.security_view.next_tab();
                }
            }
            Action::SecurityViewPrevTab => {
                if self.header.active_tab == 5 {
                    self.security_view.prev_tab();
                }
            }
            Action::SecurityViewCycleFilter => {
                if self.header.active_tab == 5 {
                    self.security_view.cycle_action_filter();
                    let filter_label = self
                        .security_view
                        .action_filter
                        .as_deref()
                        .unwrap_or("ALL")
                        .to_uppercase();
                    self.status_bar
                        .set_status(format!("WAF Event Filter: {filter_label}"));
                }
            }
            Action::SecurityViewToggleWafRule => {
                if self.header.active_tab == 5 && self.security_view.active_tab == crate::components::views::security_view::SecurityTab::WafRules {
                    if let Some((rule_id, paused)) = self.security_view.toggle_selected_rule_pause() {
                        let status_str = if paused { "Paused" } else { "Activated" };
                        self.status_bar.set_status(format!("WAF Rule {rule_id}: {status_str}"));
                        if let (Some(client), Some(zone_id)) = (&self.cf_client, &self.security_view.current_zone_id) {
                            let client = client.clone();
                            let zone_id = zone_id.clone();
                            tokio::spawn(async move {
                                let _ = client.update_waf_rule_state(&zone_id, &rule_id, Some(paused), None).await;
                            });
                        }
                    }
                }
            }
            Action::SecurityViewShiftWafPriority(delta) => {
                if self.header.active_tab == 5 && self.security_view.active_tab == crate::components::views::security_view::SecurityTab::WafRules {
                    if let Some((rule_id, new_prio)) = self.security_view.shift_selected_rule_priority(delta) {
                        self.status_bar.set_status(format!("WAF Rule {rule_id}: Priority updated to {new_prio}"));
                        if let (Some(client), Some(zone_id)) = (&self.cf_client, &self.security_view.current_zone_id) {
                            let client = client.clone();
                            let zone_id = zone_id.clone();
                            tokio::spawn(async move {
                                let _ = client.update_waf_rule_state(&zone_id, &rule_id, None, Some(new_prio)).await;
                            });
                        }
                    }
                }
            }

            // Command Palette Actions (Ctrl+P)
            Action::OpenCommandPalette => {
                self.command_palette_modal = Some(CommandPaletteModal::new(PaletteEngine::new()));
            }
            Action::CloseCommandPalette => {
                self.command_palette_modal = None;
            }
            Action::CommandPaletteInputChar(c) => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.input_char(c);
                }
            }
            Action::CommandPaletteInputBackspace => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.backspace();
                }
            }
            Action::CommandPaletteClearQuery => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.clear_query();
                }
            }
            Action::CommandPaletteNavUp => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.nav_up();
                }
            }
            Action::CommandPaletteNavDown => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.nav_down();
                }
            }
            Action::CommandPaletteNavTop => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.nav_top();
                }
            }
            Action::CommandPaletteNavBottom => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.nav_bottom();
                }
            }
            Action::CommandPaletteNextCategory => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.next_category_filter();
                }
            }
            Action::CommandPalettePrevCategory => {
                if let Some(modal) = &mut self.command_palette_modal {
                    modal.prev_category_filter();
                }
            }
            Action::CommandPaletteExecute => {
                if let Some(modal) = self.command_palette_modal.take() {
                    if let Some(cmd) = modal.selected_command() {
                        match cmd.action {
                            PaletteAction::SwitchView(idx) => {
                                let _ = self.action_tx.send(Action::SelectTab(idx));
                            }
                            PaletteAction::TriggerSync => {
                                let _ = self.action_tx.send(Action::Refresh);
                            }
                            PaletteAction::PurgeCache => {
                                if let Some(zone) = self.dns_view.selected_zone().cloned() {
                                    if let Some(client) = &self.cf_client {
                                        let client = client.clone();
                                        let zone_id = zone.id.clone();
                                        let zone_name = zone.name.clone();
                                        let tx = self.action_tx.clone();
                                        tokio::spawn(async move {
                                            match client.purge_zone_cache(&zone_id).await {
                                                Ok(_) => {
                                                    let _ = tx.send(Action::StatusMessage(format!("Purged edge cache for {zone_name}!")));
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(Action::Error(format!("Purge cache error: {e}")));
                                                }
                                            }
                                        });
                                    } else {
                                        self.status_bar.set_status("Purge simulated (no active Cloudflare token)".to_string());
                                    }
                                } else {
                                    self.status_bar.set_error("No zone selected to purge cache".to_string());
                                }
                            }
                            PaletteAction::OpenD1Console => {
                                let _ = self.action_tx.send(Action::SelectTab(3));
                                let _ = self.action_tx.send(Action::OpenD1Console);
                            }
                            PaletteAction::OpenKvModal => {
                                let _ = self.action_tx.send(Action::SelectTab(3));
                                let _ = self.action_tx.send(Action::OpenKvModal);
                            }
                            PaletteAction::OpenSecurityInspector => {
                                let _ = self.action_tx.send(Action::SelectTab(5));
                                let _ = self.action_tx.send(Action::OpenSecurityInspector);
                            }
                            PaletteAction::OpenProfileSwitcher => {
                                let _ = self.action_tx.send(Action::OpenProfileSwitcher);
                            }
                            PaletteAction::OpenOAuthLogin => {
                                let _ = self.action_tx.send(Action::StartOAuthLogin);
                            }
                            PaletteAction::Logout => {
                                let _ = self.action_tx.send(Action::Logout);
                            }
                            PaletteAction::OpenHelp => {
                                let _ = self.action_tx.send(Action::ToggleHelp);
                            }
                            PaletteAction::SetTheme(theme_name) => {
                                let preset = match theme_name.as_str() {
                                    "Catppuccin Mocha" => ThemePreset::CatppuccinMocha,
                                    "Tokyo Night" => ThemePreset::TokyoNight,
                                    "Nord" => ThemePreset::Nord,
                                    _ => ThemePreset::CloudflareDark,
                                };
                                self.theme = Theme::from_preset(preset);
                                self.status_bar.set_status(format!("Applied theme: {}", preset.name()));
                            }
                            PaletteAction::Quit => {
                                let _ = self.action_tx.send(Action::Quit);
                            }
                            PaletteAction::Custom(msg) => {
                                self.status_bar.set_status(format!("Command executed: {msg}"));
                            }
                        }
                    }
                }
            }

            // Multi-Account Profile Actions
            Action::OpenProfileSwitcher => {
                self.profile_modal = Some(ProfileModal::new());
            }
            Action::CloseProfileSwitcher => {
                self.profile_modal = None;
            }
            Action::ProfileSwitcherNavUp => {
                if let Some(modal) = &mut self.profile_modal {
                    modal.nav_up();
                }
            }
            Action::ProfileSwitcherNavDown => {
                if let Some(modal) = &mut self.profile_modal {
                    modal.nav_down();
                }
            }
            Action::ProfileSwitcherSelect => {
                if let Some(modal) = &mut self.profile_modal {
                    if let Some(profile) = modal.switch_to_selected() {
                        let _ = self.action_tx.send(Action::ProfileSwitched(profile.name));
                    }
                }
            }
            Action::ProfileSwitcherDelete => {
                if let Some(modal) = &mut self.profile_modal {
                    if modal.delete_selected() {
                        if let Some(msg) = &modal.status_msg {
                            self.status_bar.set_status(msg.clone());
                        }
                    } else if let Some(msg) = &modal.status_msg {
                        self.status_bar.set_error(msg.clone());
                    }
                }
            }
            Action::ProfileSwitched(name) => {
                self.profile_modal = None;
                if let Some(token) = AuthManager::load_token() {
                    self.cf_client = Some(CloudflareClient::new(token));
                    let config = AuthManager::load_config();
                    self.header.account_name = config.account_name.unwrap_or_else(|| name.clone());
                    self.header.is_live = true;
                    self.status_bar
                        .set_status(format!("Active profile switched to '{name}'. Refreshing..."));
                    let _ = self.action_tx.send(Action::Refresh);
                } else {
                    self.header.account_name = name.clone();
                    self.header.is_live = false;
                    self.cf_client = None;
                    self.status_bar
                        .set_status(format!("Active profile switched to '{name}' (no API token)"));
                    let _ = self.action_tx.send(Action::Refresh);
                }
            }

            Action::Render => {}
        }
        Ok(())
    }

    pub fn render(&self, f: &mut Frame) {
        let area = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Min(8),    // Active View
                Constraint::Length(1), // Status Bar
            ])
            .split(area);

        // 1. Render Header
        self.header.render(f, chunks[0], &self.theme);

        // 2. Render Active View
        match self.header.active_tab {
            0 => self.dns_view.render(f, chunks[1], &self.theme),
            1 => self.workers_view.render(f, chunks[1], &self.theme),
            2 => self.pages_view.render(f, chunks[1], &self.theme),
            3 => self.storage_view.render(f, chunks[1], &self.theme),
            4 => self.tail_view.render(f, chunks[1], &self.theme),
            5 => self.security_view.render(f, chunks[1], &self.theme),
            _ => {}
        }

        // 3. Render Status Bar
        self.status_bar.render(f, chunks[2], &self.theme, self.header.active_tab);

        // 4. Modal Overlays
        if self.show_help {
            HelpModal::render(f, area, &self.theme);
        } else if self.show_auth_modal {
            self.auth_modal.render(f, area, &self.theme);
        } else if let Some(cmd_palette) = &self.command_palette_modal {
            cmd_palette.render(f, area, &self.theme);
        } else if let Some(prof_modal) = &self.profile_modal {
            prof_modal.render(f, area, &self.theme);
        } else if let Some(sec_insp) = &self.security_inspector {
            sec_insp.render(f, area, &self.theme);
        } else if let Some(tail_log) = &self.tail_log_modal {
            tail_log.render(f, area, &self.theme);
        } else if let Some(worker_insp) = &self.worker_inspector {
            worker_insp.render(f, area, &self.theme);
        } else if let Some(logs) = &self.pages_logs_modal {
            logs.render(f, area, &self.theme);
        } else if let Some(rb) = &self.pages_rollback_modal {
            rb.render(f, area, &self.theme);
        } else if let Some(deps) = &self.pages_deployments {
            deps.render(f, area, &self.theme);
        } else if let Some(inspector) = &self.zone_inspector {
            inspector.render(f, area, &self.theme);
        } else if let Some(dns_modal) = &self.dns_modal {
            dns_modal.render(f, area, &self.theme);
        } else if let Some(del_modal) = &self.delete_modal {
            del_modal.render(f, area, &self.theme);
        } else if let Some(d1_console) = &self.d1_console_modal {
            d1_console.render(f, area, &self.theme);
        } else if let Some(kv_modal) = &self.kv_modal {
            kv_modal.render(f, area, &self.theme);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::views::security_view::SecurityTab;
    use crate::db::{CachedSecurityEvent, CachedWafRule, Zone};

    #[test]
    fn test_app_security_view_tab_and_actions() {
        let db = CacheDb::open_in_memory().unwrap();
        let zone = Zone {
            id: "test-zone-sec".to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            plan: "Enterprise".to_string(),
            created_on: "2026-01-01T00:00:00Z".to_string(),
        };
        db.upsert_zone(&zone).unwrap();

        let event = CachedSecurityEvent {
            ray_id: "ray-test-001".to_string(),
            zone_id: "test-zone-sec".to_string(),
            timestamp: "2026-02-10T14:30:00Z".to_string(),
            action: "block".to_string(),
            source: "waf".to_string(),
            rule_id: "rule-100".to_string(),
            rule_description: "Block SQL injection".to_string(),
            client_ip: "198.51.100.42".to_string(),
            client_country: "US".to_string(),
            client_asn: 13335,
            client_as_org: "Example ISP".to_string(),
            http_method: "POST".to_string(),
            http_host: "example.com".to_string(),
            http_path: "/api/login".to_string(),
            user_agent: "curl/7.88.1".to_string(),
        };
        db.replace_security_events("test-zone-sec", &[event]).unwrap();

        let rule = CachedWafRule {
            id: "rule-100".to_string(),
            zone_id: "test-zone-sec".to_string(),
            description: "Block SQL injection".to_string(),
            action: "block".to_string(),
            expression: "(http.request.uri.path contains \"/login\")".to_string(),
            paused: false,
            priority: 1,
            ref_code: "custom-sql".to_string(),
        };
        db.replace_waf_rules("test-zone-sec", &[rule]).unwrap();

        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(db, tx);

        // Switch to tab 5 (Security)
        app.update(Action::SelectTab(5)).unwrap();
        assert_eq!(app.header.active_tab, 5);

        // Security view subtab navigation
        assert_eq!(app.security_view.active_tab, SecurityTab::Events);
        app.update(Action::SecurityViewNextTab).unwrap();
        assert_eq!(app.security_view.active_tab, SecurityTab::WafRules);

        // Security filter cycle
        app.update(Action::SecurityViewCycleFilter).unwrap();
        assert_eq!(app.security_view.action_filter.as_deref(), Some("block"));

        // Open Security Inspector
        app.security_view.active_tab = SecurityTab::Events;
        app.update(Action::OpenSecurityInspector).unwrap();
        assert!(app.is_security_inspector_active());

        // Inspector tab switching
        app.update(Action::SecurityInspectorNextTab).unwrap();
        assert_eq!(
            app.security_inspector.as_ref().unwrap().active_tab,
            crate::components::security_inspector::InspectorTab::Rule
        );

        // Close Security Inspector
        app.update(Action::CloseSecurityInspector).unwrap();
        assert!(!app.is_security_inspector_active());
    }

    #[test]
    fn test_app_command_palette_actions() {
        let db = CacheDb::open_in_memory().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(db, tx);

        assert!(!app.is_command_palette_active());

        // Open palette
        app.update(Action::OpenCommandPalette).unwrap();
        assert!(app.is_command_palette_active());

        // Search input
        app.update(Action::CommandPaletteInputChar('d')).unwrap();
        app.update(Action::CommandPaletteInputChar('1')).unwrap();
        assert_eq!(app.command_palette_modal.as_ref().unwrap().query, "d1");

        // Clear search
        app.update(Action::CommandPaletteClearQuery).unwrap();
        assert_eq!(app.command_palette_modal.as_ref().unwrap().query, "");

        // Close palette
        app.update(Action::CloseCommandPalette).unwrap();
        assert!(!app.is_command_palette_active());
    }

    #[test]
    fn test_app_profile_switcher_actions() {
        let db = CacheDb::open_in_memory().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(db, tx);

        assert!(!app.is_profile_switcher_active());

        // Open profile switcher
        app.update(Action::OpenProfileSwitcher).unwrap();
        assert!(app.is_profile_switcher_active());

        // Navigation
        app.update(Action::ProfileSwitcherNavDown).unwrap();

        // Close profile switcher
        app.update(Action::CloseProfileSwitcher).unwrap();
        assert!(!app.is_profile_switcher_active());
    }

    #[test]
    fn test_app_auth_oauth_and_logout_actions() {
        let db = CacheDb::open_in_memory().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(db, tx);

        // Toggle Auth Modal
        assert!(!app.is_auth_active());
        app.update(Action::ToggleAuth).unwrap();
        assert!(app.is_auth_active());

        // Mode Switching
        assert_eq!(app.auth_modal.mode, AuthMode::OAuth);
        app.update(Action::AuthSwitchMode(AuthMode::Token)).unwrap();
        assert_eq!(app.auth_modal.mode, AuthMode::Token);
        app.update(Action::AuthSwitchMode(AuthMode::OAuth)).unwrap();
        assert_eq!(app.auth_modal.mode, AuthMode::OAuth);

        // Cancel OAuth
        app.auth_modal.oauth_status = OAuthStatus::Waiting {
            port: 8976,
            auth_url: "http://127.0.0.1:8976/callback".to_string(),
            state: "xyz".to_string(),
        };
        app.update(Action::CancelOAuthLogin).unwrap();
        assert_eq!(app.auth_modal.oauth_status, OAuthStatus::Idle);

        // OAuth Success
        app.update(Action::OAuthSuccess {
            token: "cfut_test_oauth_success".to_string(),
            account_name: Some("Test OAuth Account".to_string()),
            account_id: Some("acc_test_123".to_string()),
        }).unwrap();

        assert!(app.cf_client.is_some());
        assert!(app.header.is_live);
        assert_eq!(app.header.account_name, "Test OAuth Account");
        assert!(!app.is_auth_active());

        // Logout
        app.update(Action::Logout).unwrap();
        assert!(app.cf_client.is_none());
        assert!(!app.header.is_live);
        assert_eq!(app.header.account_name, "Logged Out");
    }
}
