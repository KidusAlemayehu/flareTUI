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
    sync::CloudflareSync,
    tail::TailStream,
};
use crate::components::{
    auth_modal::AuthModal,
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
        storage_view::StorageView,
        tail_view::TailView,
        workers_view::WorkersView,
    },
    zone_inspector::ZoneInspectorModal,
};
use crate::db::{CacheDb, DnsRecord};
use crate::theme::Theme;

pub struct App {
    pub theme: Theme,
    pub header: Header,
    pub status_bar: StatusBar,
    pub auth_modal: AuthModal,
    pub zone_inspector: Option<ZoneInspectorModal>,
    pub dns_modal: Option<DnsModal>,
    pub delete_modal: Option<ConfirmDeleteModal>,
    pub pages_deployments: Option<PagesDeploymentsModal>,
    pub pages_rollback_modal: Option<ConfirmRollbackModal>,
    pub pages_logs_modal: Option<PagesLogsModal>,
    pub dns_view: DnsView,
    pub workers_view: WorkersView,
    pub pages_view: PagesView,
    pub storage_view: StorageView,
    pub tail_view: TailView,
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
            let _ = action_tx.send(Action::SyncStarted);
        }

        Self {
            theme: Theme::default(),
            header,
            status_bar: StatusBar::new(),
            auth_modal: AuthModal::new(),
            zone_inspector: None,
            dns_modal: None,
            delete_modal: None,
            pages_deployments: None,
            pages_rollback_modal: None,
            pages_logs_modal: None,
            dns_view: DnsView::new(db.clone()),
            workers_view: WorkersView::new(db.clone()),
            pages_view: PagesView::new(db.clone()),
            storage_view: StorageView::new(),
            tail_view: TailView::new(),
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
        }
    }

    pub fn is_search_active(&self) -> bool {
        self.header.active_tab == 0 && self.dns_view.search_active
    }

    pub fn is_auth_active(&self) -> bool {
        self.show_auth_modal
    }

    pub fn is_inspector_active(&self) -> bool {
        self.zone_inspector.is_some()
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
                self.status_bar.set_status(format!("Switched to {}", self.header.tabs[self.header.active_tab]));
            }
            Action::PrevTab => {
                if self.header.active_tab == 0 {
                    self.header.active_tab = self.header.tabs.len() - 1;
                } else {
                    self.header.active_tab -= 1;
                }
                self.status_bar.set_status(format!("Switched to {}", self.header.tabs[self.header.active_tab]));
            }
            Action::SelectTab(idx) => {
                if idx < self.header.tabs.len() {
                    self.header.active_tab = idx;
                    self.status_bar.set_status(format!("Switched to {}", self.header.tabs[idx]));
                }
            }
            Action::NavUp => match self.header.active_tab {
                0 => self.dns_view.nav_up(),
                1 => self.workers_view.nav_up(),
                2 => self.pages_view.nav_up(),
                3 => self.storage_view.nav_up(),
                _ => {}
            },
            Action::NavDown => match self.header.active_tab {
                0 => self.dns_view.nav_down(),
                1 => self.workers_view.nav_down(),
                2 => self.pages_view.nav_down(),
                3 => self.storage_view.nav_down(),
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
                                    format!(
                                        "Synced {} zones, {} records, {} workers, {} pages",
                                        rep.zones_count, rep.records_count, rep.workers_count, rep.pages_count
                                    )
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
                }
            }
            Action::ToggleAuth => {
                self.show_auth_modal = !self.show_auth_modal;
                if self.show_auth_modal {
                    self.auth_modal.clear();
                }
            }
            Action::AuthInputChar(c) => {
                self.auth_modal.handle_char(c);
            }
            Action::AuthInputBackspace => {
                self.auth_modal.handle_backspace();
            }
            Action::AuthSubmit => {
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
                self.dns_view.reload_records();
                self.workers_view.reload();
                self.pages_view.reload();
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
            Action::AttachWorkerTail(worker_name) => {
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
            Action::InputChar(c) => {
                if self.header.active_tab == 0 && self.dns_view.search_active {
                    self.dns_view.search_query.push(c);
                }
            }
            Action::InputBackspace => {
                if self.header.active_tab == 0 && self.dns_view.search_active {
                    self.dns_view.search_query.pop();
                }
            }
            Action::InputSubmit => {
                if self.header.active_tab == 0 {
                    self.dns_view.search_active = false;
                }
            }
            Action::ClearSearch => {
                if self.header.active_tab == 0 {
                    self.dns_view.search_query.clear();
                    self.dns_view.search_active = false;
                }
            }
            Action::StatusMessage(msg) => {
                self.status_bar.set_status(msg);
            }
            Action::Error(err) => {
                self.auth_modal.is_validating = false;
                self.auth_modal.error_msg = Some(err.clone());
                self.status_bar.set_error(err);
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
                                                zone_id: res.zone_id,
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
                                            zone_id: res.zone_id,
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
            _ => {}
        }

        // 3. Render Status Bar
        self.status_bar.render(f, chunks[2], &self.theme, self.header.active_tab);

        // 4. Modal Overlays
        if self.show_help {
            HelpModal::render(f, area, &self.theme);
        } else if self.show_auth_modal {
            self.auth_modal.render(f, area, &self.theme);
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
        }
    }
}
