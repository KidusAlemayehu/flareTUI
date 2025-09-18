use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};

use crate::theme::Theme;

pub struct StorageView {
    pub selected_d1_idx: usize,
    pub selected_kv_idx: usize,
    pub is_kv_focused: bool,
}

impl StorageView {
    pub fn new() -> Self {
        Self {
            selected_d1_idx: 0,
            selected_kv_idx: 0,
            is_kv_focused: false,
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
            if self.selected_kv_idx < 2 {
                self.selected_kv_idx += 1;
            }
        } else if self.selected_d1_idx < 2 {
            self.selected_d1_idx += 1;
        }
    }

    pub fn switch_pane(&mut self) {
        self.is_kv_focused = !self.is_kv_focused;
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // D1 Databases Table
        let d1_rows = vec![
            Row::new(vec![
                Cell::from(if self.selected_d1_idx == 0 && !self.is_kv_focused { "▶ prod-customer-db" } else { "  prod-customer-db" }),
                Cell::from("0f8821ad-44b2-4d56-a67b-401cf69be201"),
                Cell::from("14 tables"),
                Cell::from("4.8 MB"),
                Cell::from("Active"),
            ]),
            Row::new(vec![
                Cell::from(if self.selected_d1_idx == 1 && !self.is_kv_focused { "▶ auth-sessions-d1" } else { "  auth-sessions-d1" }),
                Cell::from("c34891ff-12a8-4e12-b789-9821bb44a102"),
                Cell::from("3 tables"),
                Cell::from("1.2 MB"),
                Cell::from("Active"),
            ]),
            Row::new(vec![
                Cell::from(if self.selected_d1_idx == 2 && !self.is_kv_focused { "▶ telemetry-analytics" } else { "  telemetry-analytics" }),
                Cell::from("e91452ca-34e8-4691-8fa3-7153aef11890"),
                Cell::from("8 tables"),
                Cell::from("18.4 MB"),
                Cell::from("Active"),
            ]),
        ];

        let d1_focused = !self.is_kv_focused;
        let d1_table = Table::new(
            d1_rows,
            [
                Constraint::Length(24),
                Constraint::Length(38),
                Constraint::Length(12),
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
                .title(" D1 Databases (SQLite on Edge) - Press 's' to switch ")
                .title_style(if d1_focused { theme.title() } else { Style::default().fg(theme.text_dim) })
                .borders(Borders::ALL)
                .border_type(if d1_focused { BorderType::Thick } else { BorderType::Rounded })
                .border_style(theme.block_border(d1_focused)),
        );
        f.render_widget(d1_table, chunks[0]);

        // KV Namespaces Table
        let kv_rows = vec![
            Row::new(vec![
                Cell::from(if self.selected_kv_idx == 0 && self.is_kv_focused { "▶ AUTH_STORE" } else { "  AUTH_STORE" }),
                Cell::from("6b74e449266144e1837a7f457ecaa87b"),
                Cell::from("1,420 keys"),
                Cell::from("Active"),
            ]),
            Row::new(vec![
                Cell::from(if self.selected_kv_idx == 1 && self.is_kv_focused { "▶ RATE_LIMIT_CACHE" } else { "  RATE_LIMIT_CACHE" }),
                Cell::from("3fa85f64-5717-4562-b3fc-2c963f66afa6"),
                Cell::from("8,940 keys"),
                Cell::from("Active"),
            ]),
            Row::new(vec![
                Cell::from(if self.selected_kv_idx == 2 && self.is_kv_focused { "▶ FEATURE_FLAGS" } else { "  FEATURE_FLAGS" }),
                Cell::from("9bb82410-1882-4112-9ab2-8812cfa55912"),
                Cell::from("48 keys"),
                Cell::from("Active"),
            ]),
        ];

        let kv_focused = self.is_kv_focused;
        let kv_table = Table::new(
            kv_rows,
            [
                Constraint::Length(24),
                Constraint::Length(38),
                Constraint::Length(14),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec!["KV Namespace", "Namespace ID", "Total Keys", "Status"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(" Workers KV Namespaces ")
                .title_style(if kv_focused { theme.title() } else { Style::default().fg(theme.text_dim) })
                .borders(Borders::ALL)
                .border_type(if kv_focused { BorderType::Thick } else { BorderType::Rounded })
                .border_style(theme.block_border(kv_focused)),
        );
        f.render_widget(kv_table, chunks[1]);
    }
}
