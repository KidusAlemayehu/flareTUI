use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::db::{CacheDb, DnsRecord, Zone};
use crate::theme::Theme;

#[derive(PartialEq, Eq)]
pub enum DnsFocus {
    Zones,
    Records,
}

pub struct DnsView {
    pub db: CacheDb,
    pub zones: Vec<Zone>,
    pub selected_zone_idx: usize,
    pub records: Vec<DnsRecord>,
    pub selected_record_idx: usize,
    pub focus: DnsFocus,
    pub search_query: String,
    pub search_active: bool,
    pub sync_warning: Option<String>,
}

impl DnsView {
    pub fn new(db: CacheDb) -> Self {
        let zones = db.get_zones().unwrap_or_default();
        let mut view = Self {
            db,
            zones,
            selected_zone_idx: 0,
            records: Vec::new(),
            selected_record_idx: 0,
            focus: DnsFocus::Records,
            search_query: String::new(),
            search_active: false,
            sync_warning: None,
        };
        view.reload_records();
        view
    }

    pub fn set_sync_warning(&mut self, warning: Option<String>) {
        self.sync_warning = warning;
    }

    pub fn reload_records(&mut self) {
        if let Some(zone) = self.zones.get(self.selected_zone_idx) {
            self.records = self.db.get_dns_records(&zone.id).unwrap_or_default();
            if self.selected_record_idx >= self.records.len() && !self.records.is_empty() {
                self.selected_record_idx = self.records.len() - 1;
            }
        } else {
            self.records.clear();
            self.selected_record_idx = 0;
        }
    }

    pub fn nav_up(&mut self) {
        match self.focus {
            DnsFocus::Zones => {
                if self.selected_zone_idx > 0 {
                    self.selected_zone_idx -= 1;
                    self.reload_records();
                }
            }
            DnsFocus::Records => {
                if self.selected_record_idx > 0 {
                    self.selected_record_idx -= 1;
                }
            }
        }
    }

    pub fn nav_down(&mut self) {
        match self.focus {
            DnsFocus::Zones => {
                if !self.zones.is_empty() && self.selected_zone_idx + 1 < self.zones.len() {
                    self.selected_zone_idx += 1;
                    self.reload_records();
                }
            }
            DnsFocus::Records => {
                let filtered_count = self.filtered_records().len();
                if filtered_count > 0 && self.selected_record_idx + 1 < filtered_count {
                    self.selected_record_idx += 1;
                }
            }
        }
    }

    pub fn switch_pane(&mut self) {
        self.focus = match self.focus {
            DnsFocus::Zones => DnsFocus::Records,
            DnsFocus::Records => DnsFocus::Zones,
        };
    }

    pub fn toggle_proxy(&mut self) -> Option<String> {
        let filtered = self.filtered_records();
        if let Some(rec) = filtered.get(self.selected_record_idx) {
            if let Ok(new_status) = self.db.toggle_dns_proxy(&rec.id) {
                let name = rec.name.clone();
                self.reload_records();
                let status_str = if new_status { "🟠 Proxied" } else { "⚪ DNS Only" };
                return Some(format!("Updated {name} to {status_str}"));
            }
        }
        None
    }

    pub fn selected_record(&self) -> Option<&DnsRecord> {
        let filtered = self.filtered_records();
        filtered.get(self.selected_record_idx).copied()
    }

    pub fn selected_zone(&self) -> Option<&Zone> {
        self.zones.get(self.selected_zone_idx)
    }

    fn filtered_records(&self) -> Vec<&DnsRecord> {
        if self.search_query.is_empty() {
            self.records.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            self.records
                .iter()
                .filter(|r| {
                    r.name.to_lowercase().contains(&q)
                        || r.content.to_lowercase().contains(&q)
                        || r.record_type.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let main_chunks = if self.search_active || !self.search_query.is_empty() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(3)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5)])
                .split(area)
        };

        // Split left (Zones) and right (DNS Records)
        let split_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(32), Constraint::Min(45)])
            .split(main_chunks[0]);

        // Left Pane: Zones List
        let zones_focused = self.focus == DnsFocus::Zones;
        let zone_rows: Vec<Row> = if self.zones.is_empty() {
            vec![Row::new(vec![
                Cell::from("  No zones on account"),
                Cell::from("-"),
            ])
            .style(Style::default().fg(theme.text_dim))]
        } else {
            self.zones
                .iter()
                .enumerate()
                .map(|(i, z)| {
                    let is_selected = i == self.selected_zone_idx;
                    let prefix = if is_selected { "▶ " } else { "  " };
                    let style = if is_selected {
                        theme.selected_row()
                    } else {
                        Style::default().fg(theme.text)
                    };

                    let plan_style = match z.plan.as_str() {
                        "Enterprise" => Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
                        "Pro" => Style::default().fg(theme.info),
                        _ => Style::default().fg(theme.text_dim),
                    };

                    Row::new(vec![
                        Cell::from(format!("{prefix}{}", z.name)),
                        Cell::from(z.plan.clone()).style(plan_style),
                    ])
                    .style(style)
                })
                .collect()
        };

        let zones_table = Table::new(zone_rows, [Constraint::Min(18), Constraint::Length(10)])
            .header(
                Row::new(vec!["Zone Name", "Plan"])
                    .style(theme.header_style())
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .title(" Zones (Press 's' to switch) ")
                    .title_style(if zones_focused { theme.title() } else { Style::default().fg(theme.text_dim) })
                    .borders(Borders::ALL)
                    .border_type(if zones_focused { BorderType::Thick } else { BorderType::Rounded })
                    .border_style(theme.block_border(zones_focused)),
            );
        f.render_widget(zones_table, split_chunks[0]);

        // Right Pane: DNS Records Table
        let records_focused = self.focus == DnsFocus::Records;
        let active_zone_name = self
            .zones
            .get(self.selected_zone_idx)
            .map(|z| z.name.as_str())
            .unwrap_or("None");

        let filtered = self.filtered_records();
        let record_rows: Vec<Row> = if filtered.is_empty() {
            if let Some(warn) = &self.sync_warning {
                vec![
                    Row::new(vec![
                        Cell::from("  ⚠️"),
                        Cell::from("Sync Notice:"),
                        Cell::from(warn.as_str()),
                        Cell::from("-"),
                        Cell::from("-"),
                    ])
                    .style(Style::default().fg(theme.warning)),
                    Row::new(vec![
                        Cell::from("  ℹ️"),
                        Cell::from("Token Permission:"),
                        Cell::from("Zone -> DNS -> Read / Edit (User API Token)"),
                        Cell::from("-"),
                        Cell::from("-"),
                    ])
                    .style(Style::default().fg(theme.info)),
                    Row::new(vec![
                        Cell::from("  💡"),
                        Cell::from("Action:"),
                        Cell::from("Create token at dash.cloudflare.com/profile/api-tokens ('Edit zone DNS')"),
                        Cell::from("-"),
                        Cell::from("-"),
                    ])
                    .style(Style::default().fg(theme.text_dim)),
                ]
            } else {
                vec![Row::new(vec![
                    Cell::from("  -"),
                    Cell::from(if self.zones.is_empty() { "No zones loaded" } else { "No records in this zone" }),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                ])
                .style(Style::default().fg(theme.text_dim))]
            }
        } else {
            filtered
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let is_selected = i == self.selected_record_idx;
                    let prefix = if is_selected { "▶ " } else { "  " };
                    let style = if is_selected {
                        theme.selected_row()
                    } else {
                        Style::default().fg(theme.text)
                    };

                    let type_style = match r.record_type.as_str() {
                        "A" => Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                        "AAAA" => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        "CNAME" => Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
                        "TXT" => Style::default().fg(theme.text_dim),
                        _ => Style::default().fg(Color::Magenta),
                    };

                    let proxy_span = if r.proxied {
                        Span::styled("🟠 Proxied", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                    } else {
                        Span::styled("⚪ DNS Only", Style::default().fg(theme.text_dim))
                    };

                    let ttl_str = if r.ttl == 1 { "Auto".to_string() } else { format!("{}s", r.ttl) };

                    Row::new(vec![
                        Cell::from(format!("{prefix}{}", r.record_type)).style(type_style),
                        Cell::from(r.name.clone()),
                        Cell::from(r.content.clone()),
                        Cell::from(Line::from(proxy_span)),
                        Cell::from(ttl_str).style(Style::default().fg(theme.text_dim)),
                    ])
                    .style(style)
                })
                .collect()
        };

        let warn_suffix = if self.sync_warning.is_some() {
            " ⚠️ [Sync Notice]"
        } else {
            ""
        };

        let records_table = Table::new(
            record_rows,
            [
                Constraint::Length(10),
                Constraint::Length(24),
                Constraint::Min(20),
                Constraint::Length(14),
                Constraint::Length(8),
            ],
        )
        .header(
            Row::new(vec!["Type", "Name", "Content / Target", "Proxy Status", "TTL"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(format!(" DNS Records: {active_zone_name} (Total: {}){warn_suffix} ", filtered.len()))
                .title_style(if records_focused { theme.title() } else { Style::default().fg(theme.text_dim) })
                .borders(Borders::ALL)
                .border_type(if records_focused { BorderType::Thick } else { BorderType::Rounded })
                .border_style(theme.block_border(records_focused)),
        );
        f.render_widget(records_table, split_chunks[1]);

        // Optional Search Bar at Bottom
        if main_chunks.len() > 1 {
            let search_border = if self.search_active {
                theme.primary
            } else {
                theme.border
            };
            let search_bar = Paragraph::new(Line::from(vec![
                Span::styled("🔍 Filter: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(&self.search_query, Style::default().fg(Color::White)),
                if self.search_active {
                    Span::styled("█", Style::default().fg(theme.primary))
                } else {
                    Span::raw("")
                },
            ]))
            .block(
                Block::default()
                    .title(" Search Records (Esc to exit) ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(search_border)),
            );
            f.render_widget(search_bar, main_chunks[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_view_sync_warning_state() {
        let db = CacheDb::open_in_memory().expect("in-memory db");
        let mut view = DnsView::new(db);
        assert!(view.sync_warning.is_none());

        view.set_sync_warning(Some("Token lacks 'Zone -> DNS' permission".to_string()));
        assert_eq!(
            view.sync_warning.as_deref(),
            Some("Token lacks 'Zone -> DNS' permission")
        );

        view.set_sync_warning(None);
        assert!(view.sync_warning.is_none());
    }

    #[test]
    fn test_dns_view_preserves_records_on_reload() {
        let db = CacheDb::open_in_memory().expect("in-memory db");
        let zone = Zone {
            id: "z-test".to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            plan: "Free".to_string(),
            created_on: "2024-01-01".to_string(),
        };
        db.upsert_zone(&zone).expect("upsert zone");

        let rec = DnsRecord {
            id: "r1".to_string(),
            zone_id: "z-test".to_string(),
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            content: "1.2.3.4".to_string(),
            proxied: true,
            ttl: 1,
        };
        db.upsert_dns_record(&rec).expect("upsert record");

        let view = DnsView::new(db);
        assert_eq!(view.zones.len(), 1);
        assert_eq!(view.records.len(), 1);
        assert_eq!(view.records[0].name, "example.com");
    }
}
