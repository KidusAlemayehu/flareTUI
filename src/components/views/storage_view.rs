use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};

use crate::db::{CacheDb, CachedD1Database, CachedKvNamespace};
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

pub struct StorageView {
    pub db: CacheDb,
    pub d1_databases: Vec<CachedD1Database>,
    pub kv_namespaces: Vec<CachedKvNamespace>,
    pub selected_d1_idx: usize,
    pub selected_kv_idx: usize,
    pub is_kv_focused: bool,
}

impl StorageView {
    pub fn new(db: CacheDb) -> Self {
        let d1_databases = db.get_d1_databases().unwrap_or_default();
        let kv_namespaces = db.get_kv_namespaces().unwrap_or_default();
        Self {
            db,
            d1_databases,
            kv_namespaces,
            selected_d1_idx: 0,
            selected_kv_idx: 0,
            is_kv_focused: false,
        }
    }

    pub fn reload(&mut self) {
        self.d1_databases = self.db.get_d1_databases().unwrap_or_default();
        self.kv_namespaces = self.db.get_kv_namespaces().unwrap_or_default();
        if self.selected_d1_idx >= self.d1_databases.len() && !self.d1_databases.is_empty() {
            self.selected_d1_idx = self.d1_databases.len() - 1;
        }
        if self.selected_kv_idx >= self.kv_namespaces.len() && !self.kv_namespaces.is_empty() {
            self.selected_kv_idx = self.kv_namespaces.len() - 1;
        }
    }

    pub fn nav_up(&mut self) {
        if self.is_kv_focused {
            if self.selected_kv_idx > 0 {
                self.selected_kv_idx -= 1;
            }
        } else if self.selected_d1_idx > 0 {
            self.selected_d1_idx -= 1;
        }
    }

    pub fn nav_down(&mut self) {
        if self.is_kv_focused {
            if !self.kv_namespaces.is_empty()
                && self.selected_kv_idx + 1 < self.kv_namespaces.len()
            {
                self.selected_kv_idx += 1;
            }
        } else if !self.d1_databases.is_empty()
            && self.selected_d1_idx + 1 < self.d1_databases.len()
        {
            self.selected_d1_idx += 1;
        }
    }

    pub fn switch_pane(&mut self) {
        self.is_kv_focused = !self.is_kv_focused;
    }

    pub fn selected_d1(&self) -> Option<&CachedD1Database> {
        self.d1_databases.get(self.selected_d1_idx)
    }

    pub fn selected_kv(&self) -> Option<&CachedKvNamespace> {
        self.kv_namespaces.get(self.selected_kv_idx)
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // 1. D1 Databases Table
        let d1_focused = !self.is_kv_focused;
        let mut d1_rows = Vec::new();

        if self.d1_databases.is_empty() {
            d1_rows.push(Row::new(vec![
                Cell::from("  No D1 databases provisioned or cached (Press 'r' to sync)"),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("Inactive"),
            ]).style(Style::default().fg(theme.text_dim)));
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

                d1_rows.push(Row::new(vec![
                    Cell::from(name),
                    Cell::from(db.uuid.clone()),
                    Cell::from(db.num_tables.to_string()),
                    Cell::from(size),
                    Cell::from("Active"),
                ]).style(row_style));
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
        let kv_focused = self.is_kv_focused;
        let mut kv_rows = Vec::new();

        if self.kv_namespaces.is_empty() {
            kv_rows.push(Row::new(vec![
                Cell::from("  No KV namespaces provisioned or cached (Press 'r' to sync)"),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("Inactive"),
            ]).style(Style::default().fg(theme.text_dim)));
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

                kv_rows.push(Row::new(vec![
                    Cell::from(title),
                    Cell::from(kv.id.clone()),
                    Cell::from(url_encoding_status),
                    Cell::from("Active"),
                ]).style(row_style));
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
            Row::new(vec!["KV Namespace", "Namespace ID", "URL Encoding", "Status"])
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
        }).unwrap();
        db.upsert_d1_database(&CachedD1Database {
            uuid: "db-2".to_string(),
            name: "users".to_string(),
            version: "beta".to_string(),
            created_at: "2025-01-02".to_string(),
            num_tables: 5,
            file_size: 16384,
        }).unwrap();

        db.upsert_kv_namespace(&CachedKvNamespace {
            id: "kv-1".to_string(),
            title: "sessions".to_string(),
            supports_url_encoding: true,
        }).unwrap();

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
    }
}
