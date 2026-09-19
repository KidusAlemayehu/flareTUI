use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};

use crate::db::{CacheDb, CachedD1Database, CachedKvNamespace, CachedR2Bucket, CachedR2Object};
use crate::theme::Theme;

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "-".to_string();
    }
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoragePane {
    D1 = 0,
    Kv = 1,
    R2 = 2,
}

pub struct StorageView {
    pub db: CacheDb,
    pub d1_databases: Vec<CachedD1Database>,
    pub kv_namespaces: Vec<CachedKvNamespace>,
    pub r2_buckets: Vec<CachedR2Bucket>,
    pub r2_objects: Vec<CachedR2Object>,
    pub selected_d1_idx: usize,
    pub selected_kv_idx: usize,
    pub selected_r2_idx: usize,
    pub active_pane: StoragePane,
    pub is_kv_focused: bool,
}

impl StorageView {
    pub fn new(db: CacheDb) -> Self {
        let d1_databases = db.get_d1_databases().unwrap_or_default();
        let kv_namespaces = db.get_kv_namespaces().unwrap_or_default();
        let r2_buckets = db.get_r2_buckets("").unwrap_or_default();
        Self {
            db,
            d1_databases,
            kv_namespaces,
            r2_buckets,
            r2_objects: Vec::new(),
            selected_d1_idx: 0,
            selected_kv_idx: 0,
            selected_r2_idx: 0,
            active_pane: StoragePane::D1,
            is_kv_focused: false,
        }
    }

    pub fn reload(&mut self) {
        self.d1_databases = self.db.get_d1_databases().unwrap_or_default();
        self.kv_namespaces = self.db.get_kv_namespaces().unwrap_or_default();
        self.r2_buckets = self.db.get_r2_buckets("").unwrap_or_default();
        if self.selected_d1_idx >= self.d1_databases.len() && !self.d1_databases.is_empty() {
            self.selected_d1_idx = self.d1_databases.len() - 1;
        }
        if self.selected_kv_idx >= self.kv_namespaces.len() && !self.kv_namespaces.is_empty() {
            self.selected_kv_idx = self.kv_namespaces.len() - 1;
        }
        if self.selected_r2_idx >= self.r2_buckets.len() && !self.r2_buckets.is_empty() {
            self.selected_r2_idx = self.r2_buckets.len() - 1;
        }
    }

    pub fn nav_up(&mut self) {
        match self.active_pane {
            StoragePane::D1 => {
                if self.selected_d1_idx > 0 {
                    self.selected_d1_idx -= 1;
                }
            }
            StoragePane::Kv => {
                if self.selected_kv_idx > 0 {
                    self.selected_kv_idx -= 1;
                }
            }
            StoragePane::R2 => {
                if self.selected_r2_idx > 0 {
                    self.selected_r2_idx -= 1;
                }
            }
        }
    }

    pub fn nav_down(&mut self) {
        match self.active_pane {
            StoragePane::D1 => {
                if !self.d1_databases.is_empty()
                    && self.selected_d1_idx + 1 < self.d1_databases.len()
                {
                    self.selected_d1_idx += 1;
                }
            }
            StoragePane::Kv => {
                if !self.kv_namespaces.is_empty()
                    && self.selected_kv_idx + 1 < self.kv_namespaces.len()
                {
                    self.selected_kv_idx += 1;
                }
            }
            StoragePane::R2 => {
                if !self.r2_buckets.is_empty() && self.selected_r2_idx + 1 < self.r2_buckets.len() {
                    self.selected_r2_idx += 1;
                }
            }
        }
    }

    pub fn switch_pane(&mut self) {
        self.active_pane = match self.active_pane {
            StoragePane::D1 => StoragePane::Kv,
            StoragePane::Kv => StoragePane::R2,
            StoragePane::R2 => StoragePane::D1,
        };
        self.is_kv_focused = self.active_pane == StoragePane::Kv;
    }

    pub fn selected_d1(&self) -> Option<&CachedD1Database> {
        self.d1_databases.get(self.selected_d1_idx)
    }

    pub fn selected_kv(&self) -> Option<&CachedKvNamespace> {
        self.kv_namespaces.get(self.selected_kv_idx)
    }

    pub fn selected_r2(&self) -> Option<&CachedR2Bucket> {
        self.r2_buckets.get(self.selected_r2_idx)
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area);

        // 1. D1 Databases Table
        let d1_focused = self.active_pane == StoragePane::D1;
        let mut d1_rows = Vec::new();

        if self.d1_databases.is_empty() {
            d1_rows.push(
                Row::new(vec![
                    Cell::from("  No D1 databases provisioned or cached (Press 'r' to sync)"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("Inactive"),
                ])
                .style(Style::default().fg(theme.text_dim)),
            );
        } else {
            for (idx, db) in self.d1_databases.iter().enumerate() {
                let is_selected = d1_focused && idx == self.selected_d1_idx;
                let marker = if is_selected { "▶ " } else { "  " };
                let name = format!("{}{}", marker, db.name);
                let size = format_bytes(db.file_size);

                let row_style = if is_selected {
                    theme.selected_row()
                } else {
                    Style::default().fg(theme.text)
                };

                d1_rows.push(
                    Row::new(vec![
                        Cell::from(name),
                        Cell::from(db.uuid.clone()),
                        Cell::from(db.num_tables.to_string()),
                        Cell::from(size),
                        Cell::from("Active"),
                    ])
                    .style(row_style),
                );
            }
        }

        let d1_title = format!(
            " D1 Databases (SQLite on Edge) [{}] - [Enter] SQL Console | [s] Switch Pane ",
            self.d1_databases.len()
        );

        let d1_table = Table::new(
            d1_rows,
            [
                Constraint::Length(35),
                Constraint::Length(38),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec!["D1 Database", "UUID", "Tables", "Size", "Status"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(d1_title)
                .title_style(if d1_focused {
                    theme.title()
                } else {
                    Style::default().fg(theme.text_dim)
                })
                .borders(Borders::ALL)
                .border_type(if d1_focused {
                    BorderType::Thick
                } else {
                    BorderType::Rounded
                })
                .border_style(theme.block_border(d1_focused)),
        );
        f.render_widget(d1_table, chunks[0]);

        // 2. KV Namespaces Table
        let kv_focused = self.active_pane == StoragePane::Kv;
        let mut kv_rows = Vec::new();

        if self.kv_namespaces.is_empty() {
            kv_rows.push(
                Row::new(vec![
                    Cell::from("  No KV namespaces provisioned or cached (Press 'r' to sync)"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("Inactive"),
                ])
                .style(Style::default().fg(theme.text_dim)),
            );
        } else {
            for (idx, kv) in self.kv_namespaces.iter().enumerate() {
                let is_selected = kv_focused && idx == self.selected_kv_idx;
                let marker = if is_selected { "▶ " } else { "  " };
                let title = format!("{}{}", marker, kv.title);

                let row_style = if is_selected {
                    theme.selected_row()
                } else {
                    Style::default().fg(theme.text)
                };

                let url_encoding_status = if kv.supports_url_encoding {
                    "Supported"
                } else {
                    "None"
                };

                kv_rows.push(
                    Row::new(vec![
                        Cell::from(title),
                        Cell::from(kv.id.clone()),
                        Cell::from(url_encoding_status),
                        Cell::from("Active"),
                    ])
                    .style(row_style),
                );
            }
        }

        let kv_title = format!(
            " Workers KV Namespaces [{}] - [Enter] Key Browser | [s] Switch Pane ",
            self.kv_namespaces.len()
        );

        let kv_table = Table::new(
            kv_rows,
            [
                Constraint::Length(35),
                Constraint::Length(38),
                Constraint::Length(14),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec![
                "KV Namespace",
                "Namespace ID",
                "URL Encoding",
                "Status",
            ])
            .style(theme.header_style())
            .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(kv_title)
                .title_style(if kv_focused {
                    theme.title()
                } else {
                    Style::default().fg(theme.text_dim)
                })
                .borders(Borders::ALL)
                .border_type(if kv_focused {
                    BorderType::Thick
                } else {
                    BorderType::Rounded
                })
                .border_style(theme.block_border(kv_focused)),
        );
        f.render_widget(kv_table, chunks[1]);

        // 3. R2 Buckets Table
        let r2_focused = self.active_pane == StoragePane::R2;
        let mut r2_rows = Vec::new();

        if self.r2_buckets.is_empty() {
            r2_rows.push(
                Row::new(vec![
                    Cell::from("  No R2 buckets provisioned or cached (Press 'r' to sync)"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("Inactive"),
                ])
                .style(Style::default().fg(theme.text_dim)),
            );
        } else {
            for (idx, bucket) in self.r2_buckets.iter().enumerate() {
                let is_selected = r2_focused && idx == self.selected_r2_idx;
                let marker = if is_selected { "▶ " } else { "  " };
                let name = format!("{}{}", marker, bucket.name);

                let row_style = if is_selected {
                    theme.selected_row()
                } else {
                    Style::default().fg(theme.text)
                };

                let loc = if bucket.location.is_empty() {
                    "auto"
                } else {
                    &bucket.location
                };

                r2_rows.push(
                    Row::new(vec![
                        Cell::from(name),
                        Cell::from(loc.to_string()),
                        Cell::from(bucket.creation_date.clone()),
                        Cell::from("Active"),
                    ])
                    .style(row_style),
                );
            }
        }

        let r2_title = format!(
            " R2 Object Storage [{}] - [s] Switch Pane ",
            self.r2_buckets.len()
        );

        let r2_table = Table::new(
            r2_rows,
            [
                Constraint::Length(35),
                Constraint::Length(16),
                Constraint::Length(28),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec!["R2 Bucket", "Location", "Creation Date", "Status"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(r2_title)
                .title_style(if r2_focused {
                    theme.title()
                } else {
                    Style::default().fg(theme.text_dim)
                })
                .borders(Borders::ALL)
                .border_type(if r2_focused {
                    BorderType::Thick
                } else {
                    BorderType::Rounded
                })
                .border_style(theme.block_border(r2_focused)),
        );
        f.render_widget(r2_table, chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "-");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(1048576 * 5), "5.0 MB");
    }

    #[test]
    fn test_storage_view_navigation_and_pane_switching() {
        let db = CacheDb::open_in_memory().unwrap();
        db.upsert_d1_database(&CachedD1Database {
            uuid: "db-1".to_string(),
            name: "analytics".to_string(),
            version: "beta".to_string(),
            created_at: "2025-01-01".to_string(),
            num_tables: 3,
            file_size: 4096,
        })
        .unwrap();
        db.upsert_d1_database(&CachedD1Database {
            uuid: "db-2".to_string(),
            name: "users".to_string(),
            version: "beta".to_string(),
            created_at: "2025-01-02".to_string(),
            num_tables: 5,
            file_size: 16384,
        })
        .unwrap();

        db.upsert_kv_namespace(&CachedKvNamespace {
            id: "kv-1".to_string(),
            title: "sessions".to_string(),
            supports_url_encoding: true,
        })
        .unwrap();

        let mut view = StorageView::new(db);
        assert_eq!(view.d1_databases.len(), 2);
        assert_eq!(view.kv_namespaces.len(), 1);
        assert!(!view.is_kv_focused);

        assert_eq!(view.selected_d1().unwrap().name, "analytics");
        view.nav_down();
        assert_eq!(view.selected_d1().unwrap().name, "users");

        view.switch_pane();
        assert!(view.is_kv_focused);
        assert_eq!(view.selected_kv().unwrap().title, "sessions");

        view.switch_pane();
        assert_eq!(view.active_pane, StoragePane::R2);
        view.switch_pane();
        assert_eq!(view.active_pane, StoragePane::D1);
    }

    #[test]
    fn test_storage_view_r2_buckets_rendering() {
        let db = CacheDb::open_in_memory().unwrap();
        db.replace_r2_buckets(
            "",
            &[CachedR2Bucket {
                name: "media-assets".to_string(),
                account_id: "acc-1".to_string(),
                creation_date: "2026-03-01".to_string(),
                location: "weur".to_string(),
            }],
        )
        .unwrap();

        let mut view = StorageView::new(db);
        assert_eq!(view.r2_buckets.len(), 1);
        assert_eq!(view.selected_r2().unwrap().name, "media-assets");

        view.active_pane = StoragePane::R2;
        let backend = ratatui::backend::TestBackend::new(120, 40);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let theme = Theme::default();
        terminal.draw(|f| view.render(f, f.area(), &theme)).unwrap();
    }
}
