use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::cloudflare::client::PagesBuildLogEntry;
use crate::theme::Theme;

pub struct PagesLogsModal {
    pub project_name: String,
    pub deployment_id: String,
    pub short_id: String,
    pub logs: Vec<PagesBuildLogEntry>,
    pub scroll_offset: usize,
    pub is_loading: bool,
}

impl PagesLogsModal {
    pub fn new(project_name: String, deployment_id: String, short_id: String) -> Self {
        Self {
            project_name,
            deployment_id,
            short_id,
            logs: Vec::new(),
            scroll_offset: 0,
            is_loading: true,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    pub fn scroll_down(&mut self) {
        if !self.logs.is_empty() && self.scroll_offset + 1 < self.logs.len() {
            self.scroll_offset += 1;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 100.min(area.width.saturating_sub(4));
        let popup_height = 24.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(format!(" 📋 Build Logs: {} [{}] ", self.project_name, self.short_id))
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Logs viewport
                Constraint::Length(1), // Footer hotkeys
            ])
            .margin(1)
            .split(popup_area);

        if self.is_loading {
            let loading = Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("   ⏳ Fetching build execution history from Cloudflare v4 API...", Style::default().fg(theme.warning)),
                ]),
            ]);
            f.render_widget(loading, chunks[0]);
        } else if self.logs.is_empty() {
            let empty = Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("   No historical log lines returned for this build.", Style::default().fg(theme.text_dim)),
                ]),
                Line::from(vec![
                    Span::styled("   (Deployments triggered via direct upload or older builds may not retain build logs)", Style::default().fg(theme.text_dim)),
                ]),
            ]);
            f.render_widget(empty, chunks[0]);
        } else {
            let visible_height = chunks[0].height as usize;
            let total_logs = self.logs.len();
            let start = self.scroll_offset.min(total_logs.saturating_sub(1));
            let end = (start + visible_height).min(total_logs);

            let log_lines: Vec<Line> = self.logs[start..end]
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let line_no = start + i + 1;
                    let ts_str = entry.ts.as_deref().unwrap_or("");
                    let display_ts = if ts_str.len() >= 19 {
                        &ts_str[11..19]
                    } else {
                        ts_str
                    };

                    let text = entry.line.as_deref().unwrap_or("");
                    let text_style = if text.contains("Success") || text.contains("Finished") || text.contains("Complete") {
                        Style::default().fg(theme.success)
                    } else if text.contains("Error") || text.contains("Failed") || text.contains("ERR!") {
                        Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
                    } else if text.contains("WARN") || text.contains("warning") {
                        Style::default().fg(theme.warning)
                    } else if text.starts_with('[') || text.contains("Step") {
                        Style::default().fg(theme.info)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    Line::from(vec![
                        Span::styled(format!("{line_no:>4} │ "), Style::default().fg(theme.text_dim)),
                        Span::styled(format!("{display_ts:<9} "), Style::default().fg(theme.text_dim)),
                        Span::styled(text, text_style),
                    ])
                })
                .collect();

            let log_widget = Paragraph::new(log_lines);
            f.render_widget(log_widget, chunks[0]);
        }

        // Footer
        let footer_text = Line::from(vec![
            Span::styled(" [j/k or ↑/↓] ", Style::default().fg(theme.primary)),
            Span::styled("Scroll Logs   ", Style::default().fg(Color::White)),
            Span::styled(format!("Line {} of {}   ", self.scroll_offset + 1, self.logs.len().max(1)), Style::default().fg(theme.text_dim)),
            Span::styled(" [Esc] ", Style::default().fg(theme.secondary)),
            Span::styled("Close Logs", Style::default().fg(theme.text_dim)),
        ]);
        let footer = Paragraph::new(footer_text);
        f.render_widget(footer, chunks[1]);
    }
}
