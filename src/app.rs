use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::cloudflare::{
    auth::AuthManager,
    client::CloudflareClient,
    sync::CloudflareSync,
    tail::TailStream,
};
use crate::components::{
    auth_modal::AuthModal,
    header::Header,
    help::HelpModal,
    status_bar::StatusBar,
    views::{
        dns_view::DnsView,
        pages_view::PagesView,
        storage_view::StorageView,
        tail_view::TailView,
        workers_view::WorkersView,
    },
};
use crate::db::CacheDb;
use crate::theme::Theme;

pub struct App {
    pub theme: Theme,
    pub header: Header,
    pub status_bar: StatusBar,
    pub auth_modal: AuthModal,
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
        }
    }

    pub fn is_search_active(&self) -> bool {
        self.header.active_tab == 0 && self.dns_view.search_active
    }

    pub fn is_auth_active(&self) -> bool {
        self.show_auth_modal
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
                                let msg = format!(
                                    "Synced {} zones, {} records, {} workers, {} pages",
                                    rep.zones_count, rep.records_count, rep.workers_count, rep.pages_count
                                );
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
                                    let msg = format!(
                                        "Connected! Synced {} zones, {} DNS records, {} workers",
                                        rep.zones_count, rep.records_count, rep.workers_count
                                    );
                                    let _ = tx.send(Action::SyncCompleted(msg));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!("Sync notice: {e}")));
                                }
                            }

                            // Start live tail stream if possible
                            if let (Some(acc_id), Some(worker)) = (account_id, db.get_workers().ok().and_then(|w| w.first().cloned())) {
                                if let Ok(tail_res) = client.create_worker_tail(&acc_id, &worker.name).await {
                                    let (tail_tx, mut tail_rx) = tokio::sync::mpsc::unbounded_channel();
                                    let action_tx = tx.clone();
                                    tokio::spawn(async move {
                                        while let Some(log) = tail_rx.recv().await {
                                            let _ = action_tx.send(Action::TailLog(log));
                                        }
                                    });
                                    let _ = TailStream::start(tail_res.url, tail_tx).await;
                                }
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
                self.tail_view.logs.push(entry);
                if self.tail_view.logs.len() > 200 {
                    self.tail_view.logs.remove(0);
                }
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
        }
    }
}
