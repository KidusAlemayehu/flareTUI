use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::action::Action;
use crate::components::{
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
    pub dns_view: DnsView,
    pub workers_view: WorkersView,
    pub pages_view: PagesView,
    pub storage_view: StorageView,
    pub tail_view: TailView,
    pub show_help: bool,
    pub should_quit: bool,
    pub tick_counter: usize,
}

impl App {
    pub fn new(db: CacheDb) -> Self {
        Self {
            theme: Theme::default(),
            header: Header::new(),
            status_bar: StatusBar::new(),
            dns_view: DnsView::new(db.clone()),
            workers_view: WorkersView::new(db.clone()),
            pages_view: PagesView::new(db),
            storage_view: StorageView::new(),
            tail_view: TailView::new(),
            show_help: false,
            should_quit: false,
            tick_counter: 0,
        }
    }

    pub fn is_search_active(&self) -> bool {
        self.header.active_tab == 0 && self.dns_view.search_active
    }

    pub fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Tick => {
                self.tick_counter += 1;
                self.tail_view.add_tick_log(self.tick_counter);
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
                    if let Some(msg) = self.dns_view.toggle_proxy() {
                        self.status_bar.set_status(msg);
                    }
                }
            }
            Action::Refresh => {
                match self.header.active_tab {
                    0 => self.dns_view.reload_records(),
                    1 => self.workers_view.reload(),
                    2 => self.pages_view.reload(),
                    _ => {}
                }
                self.status_bar.set_status("Cache data refreshed");
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

        // 4. Modal Overlays (Help)
        if self.show_help {
            HelpModal::render(f, area, &self.theme);
        }
    }
}
