use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::db::{CacheDb, WorkerScript};
use crate::theme::Theme;

pub struct WorkersView {
    pub db: CacheDb,
    pub workers: Vec<WorkerScript>,
    pub selected_idx: usize,
}

impl WorkersView {
    pub fn new(db: CacheDb) -> Self {
        let workers = db.get_workers().unwrap_or_default();
        Self {
            db,
            workers,
            selected_idx: 0,
        }
    }

    pub fn reload(&mut self) {
        self.workers = self.db.get_workers().unwrap_or_default();
        if self.selected_idx >= self.workers.len() && !self.workers.is_empty() {
            self.selected_idx = self.workers.len() - 1;
        }
    }

    pub fn nav_up(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn nav_down(&mut self) {
        if !self.workers.is_empty() && self.selected_idx + 1 < self.workers.len() {
            self.selected_idx += 1;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(8)])
            .split(area);

        // Workers Table
        let rows: Vec<Row> = self
            .workers
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let is_selected = i == self.selected_idx;
                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    theme.selected_row()
                } else {
                    Style::default().fg(theme.text)
                };

                let status_style = match w.status.as_str() {
                    "Healthy" => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
                };

                Row::new(vec![
                    Cell::from(format!("{prefix}{}", w.name))
                        .style(Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Cell::from(w.routes.clone()),
                    Cell::from(w.last_deployed.clone()).style(Style::default().fg(theme.text_dim)),
                    Cell::from(w.compatibility_date.clone()).style(Style::default().fg(theme.text_dim)),
                    Cell::from(w.status.clone()).style(status_style),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(24),
                Constraint::Min(28),
                Constraint::Length(18),
                Constraint::Length(14),
                Constraint::Length(12),
            ],
        )
        .header(
            Row::new(vec!["Worker Script", "Active Routes", "Last Deployed", "Compat Date", "Status"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(format!(" Cloudflare Workers (Total: {}) ", self.workers.len()))
                .title_style(theme.title())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_focus)),
        );
        f.render_widget(table, chunks[0]);

        // Details Panel for Selected Worker
        if let Some(w) = self.workers.get(self.selected_idx) {
            let details_text = vec![
                Line::from(vec![
                    Span::styled("Script Name: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(&w.name, Style::default().fg(Color::White)),
                    Span::raw("    "),
                    Span::styled("Status: ", Style::default().fg(theme.primary)),
                    Span::styled(&w.status, Style::default().fg(theme.success)),
                ]),
                Line::from(vec![
                    Span::styled("Routes: ", Style::default().fg(theme.text_dim)),
                    Span::styled(&w.routes, Style::default().fg(theme.info)),
                ]),
                Line::from(vec![
                    Span::styled("Bindings: ", Style::default().fg(theme.text_dim)),
                    Span::styled("KV: AUTH_STORE  |  D1: PROD_DB  |  Vars: [API_SECRET=••••, ENV=production]", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Action: ", Style::default().fg(theme.secondary)),
                    Span::styled("Press 'l' to attach Real-time Live Tail Log Console via SSE", Style::default().fg(theme.primary).add_modifier(Modifier::ITALIC)),
                ]),
            ];

            let details_widget = Paragraph::new(details_text).block(
                Block::default()
                    .title(format!(" Worker Details & Bindings: {} ", w.name))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border)),
            );
            f.render_widget(details_widget, chunks[1]);
        }
    }
}
