use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::components::views::tail_view::LogEntry;
use crate::theme::Theme;

pub struct TailLogModal {
    pub log: LogEntry,
    pub scroll_offset: usize,
}

impl TailLogModal {
    pub fn new(log: LogEntry) -> Self {
        Self {
            log,
            scroll_offset: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 96.min(area.width.saturating_sub(4));
        let popup_height = 26.min(area.height.saturating_sub(2));

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
                Constraint::Length(4), // Summary header
                Constraint::Min(12),   // Body (JSON / Exceptions / Messages)
                Constraint::Length(2), // Key hints
            ])
            .margin(1)
            .split(popup_area);

        let block = Block::default()
            .title(format!(" 🔍 Worker Log Inspector: {} {} ", self.log.method, self.log.url))
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        // 1. Summary Header
        let status_style = match self.log.status {
            200..=299 => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
            400..=499 => Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
        };

        let summary_lines = vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{}", self.log.status), status_style),
                Span::raw("    "),
                Span::styled("Method: ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.log.method, Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled("Latency: ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{}ms", self.log.duration_ms), Style::default().fg(Color::White)),
                Span::raw("    "),
                Span::styled("Outcome: ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.log.outcome, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled("Time: ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.log.timestamp, Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::styled("Request URL: ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.log.url, Style::default().fg(Color::White)),
            ]),
        ];

        let summary_widget = Paragraph::new(summary_lines).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(summary_widget, chunks[0]);

        // 2. Body: Raw JSON & Exceptions
        let mut body_lines = Vec::new();
        if !self.log.exceptions.is_empty() {
            body_lines.push(Line::from(vec![
                Span::styled("💥 Exceptions & Error Traces:", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
            ]));
            for exc in &self.log.exceptions {
                body_lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(exc, Style::default().fg(theme.error)),
                ]));
            }
            body_lines.push(Line::raw(""));
        }

        body_lines.push(Line::from(vec![
            Span::styled("📜 Log Output / Console Message:", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]));
        body_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(&self.log.message, Style::default().fg(Color::White)),
        ]));
        body_lines.push(Line::raw(""));

        if let Some(raw) = &self.log.raw_json {
            body_lines.push(Line::from(vec![
                Span::styled("📦 Complete Cloudflare Tail JSON Payload:", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            ]));
            for line in raw.lines() {
                body_lines.push(Line::from(vec![
                    Span::styled(format!("  {line}"), Style::default().fg(theme.text_dim)),
                ]));
            }
        }

        let body_widget = Paragraph::new(body_lines)
            .scroll((self.scroll_offset as u16, 0))
            .block(
                Block::default()
                    .title(" Payload & Traces ")
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.border)),
            );
        f.render_widget(body_widget, chunks[1]);

        // 3. Footer hints
        let hints = Line::from(vec![
            Span::styled("[j/k] or [↑/↓] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("Scroll Payload   ", Style::default().fg(theme.text)),
            Span::styled("[Esc] or [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("Close Inspector", Style::default().fg(theme.text)),
        ]);
        f.render_widget(Paragraph::new(hints), chunks[2]);
    }
}
