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
                Cell::from("  (No D1 databases provisioned on this account)"),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("Inactive"),
            ]),
        ];

        let d1_focused = !self.is_kv_focused;
        let d1_table = Table::new(
            d1_rows,
            [
                Constraint::Length(45),
                Constraint::Length(20),
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
                Cell::from("  (No KV namespaces provisioned on this account)"),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("Inactive"),
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
