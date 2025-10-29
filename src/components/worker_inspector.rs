use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::cloudflare::client::WorkerRouteDetail;
use crate::db::CachedWorkerBinding;
use crate::theme::Theme;

pub struct WorkerInspectorModal {
    pub worker_name: String,
    pub status: String,
    pub compatibility_date: String,
    pub compatibility_flags: Vec<String>,
    pub usage_model: String,
    pub handlers: Vec<String>,
    pub routes: Vec<WorkerRouteDetail>,
    pub bindings: Vec<CachedWorkerBinding>,
    pub selected_binding_idx: usize,
    pub is_loading: bool,
    pub action_feedback: Option<String>,
}

impl WorkerInspectorModal {
    pub fn new(worker_name: String, status: String, compatibility_date: String) -> Self {
        Self {
            worker_name,
            status,
            compatibility_date,
            compatibility_flags: Vec::new(),
            usage_model: "standard".to_string(),
            handlers: Vec::new(),
            routes: Vec::new(),
            bindings: Vec::new(),
            selected_binding_idx: 0,
            is_loading: false,
            action_feedback: None,
        }
    }

    pub fn nav_up(&mut self) {
        if self.selected_binding_idx > 0 {
            self.selected_binding_idx -= 1;
        }
    }

    pub fn nav_down(&mut self) {
        if !self.bindings.is_empty() && self.selected_binding_idx + 1 < self.bindings.len() {
            self.selected_binding_idx += 1;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 86.min(area.width.saturating_sub(4));
        let popup_height = 24.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Metadata overview
                Constraint::Min(8),    // Bindings table
                Constraint::Length(4), // Routes & Subdomain
                Constraint::Length(2), // Key hints & feedback
            ])
            .margin(1)
            .split(popup_area);

        let block = Block::default()
            .title(format!(" ⚡ Worker Script Inspector: {} ", self.worker_name))
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        // 1. Metadata overview
        let status_style = match self.status.as_str() {
            "Healthy" | "active" => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
        };

        let flags_display = if self.compatibility_flags.is_empty() {
            "none".to_string()
        } else {
            self.compatibility_flags.join(", ")
        };

        let handlers_display = if self.handlers.is_empty() {
            "fetch".to_string()
        } else {
            self.handlers.join(", ")
        };

        let overview_lines = vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("● {}", self.status), status_style),
                Span::raw("    "),
                Span::styled("Compat Date: ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.compatibility_date, Style::default().fg(Color::White)),
                Span::raw("    "),
                Span::styled("Usage Model: ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.usage_model, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Handlers: ", Style::default().fg(theme.text_dim)),
                Span::styled(handlers_display, Style::default().fg(theme.info)),
                Span::raw("    "),
                Span::styled("Compat Flags: ", Style::default().fg(theme.text_dim)),
                Span::styled(flags_display, Style::default().fg(theme.warning)),
            ]),
        ];

        let overview = Paragraph::new(overview_lines).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(overview, chunks[0]);

        // 2. Bindings table
        let binding_rows: Vec<Row> = if self.bindings.is_empty() {
            vec![Row::new(vec![
                Cell::from("  No configured bindings (KV, D1, Queues, Secrets)"),
                Cell::from("-"),
                Cell::from("-"),
            ])
            .style(Style::default().fg(theme.text_dim))]
        } else {
            self.bindings
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    let is_selected = i == self.selected_binding_idx;
                    let prefix = if is_selected { "▶ " } else { "  " };
                    let style = if is_selected {
                        theme.selected_row()
                    } else {
                        Style::default().fg(theme.text)
                    };

                    let type_style = match b.binding_type.as_str() {
                        "d1" => Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                        "kv_namespace" => Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
                        "r2_bucket" => Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
                        "secret_text" => Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
                        _ => Style::default().fg(Color::White),
                    };

                    let detail_display = if b.binding_type == "secret_text" {
                        "●●●●●●●● [Encrypted Secret]".to_string()
                    } else {
                        b.details.clone()
                    };

                    Row::new(vec![
                        Cell::from(format!("{prefix}{}", b.name))
                            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Cell::from(b.binding_type.clone()).style(type_style),
                        Cell::from(detail_display).style(Style::default().fg(theme.text_dim)),
                    ])
                    .style(style)
                })
                .collect()
        };

        let bindings_table = Table::new(
            binding_rows,
            [
                Constraint::Length(22),
                Constraint::Length(16),
                Constraint::Min(36),
            ],
        )
        .header(
            Row::new(vec!["Binding Name", "Type", "Resource Target / Configuration"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(format!(" Resource Bindings ({}) ", self.bindings.len()))
                .title_style(theme.title())
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(bindings_table, chunks[1]);

        // 3. Routes / Subdomain
        let routes_text = if self.routes.is_empty() {
            vec![Line::from(vec![
                Span::styled("Routes: ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{}.*.workers.dev (workers.dev default zone)", self.worker_name), Style::default().fg(theme.info)),
            ])]
        } else {
            self.routes
                .iter()
                .map(|r| {
                    Line::from(vec![
                        Span::styled("Route: ", Style::default().fg(theme.text_dim)),
                        Span::styled(&r.pattern, Style::default().fg(theme.success)),
                        Span::styled(format!(" (Zone: {})", r.zone_name.as_deref().unwrap_or("Custom")), Style::default().fg(theme.text_dim)),
                    ])
                })
                .collect()
        };
        let routes_widget = Paragraph::new(routes_text).block(
            Block::default()
                .title(" Active Triggers & Routes ")
                .title_style(theme.title())
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(routes_widget, chunks[2]);

        // 4. Feedback & Key Hints
        let feedback_line = if self.is_loading {
            Line::from(vec![
                Span::styled("⏳ Loading live script metadata from Cloudflare API...", Style::default().fg(theme.warning)),
            ])
        } else if let Some(fb) = &self.action_feedback {
            Line::from(vec![
                Span::styled(format!("✓ {fb}"), Style::default().fg(theme.success)),
            ])
        } else {
            Line::from(vec![
                Span::styled("[t/Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled("Stream Live Tail   ", Style::default().fg(theme.text)),
                Span::styled("[j/k] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled("Navigate Bindings   ", Style::default().fg(theme.text)),
                Span::styled("[Esc] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled("Close", Style::default().fg(theme.text)),
            ])
        };

        f.render_widget(Paragraph::new(feedback_line), chunks[3]);
    }
}
