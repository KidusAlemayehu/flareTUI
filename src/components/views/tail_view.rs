use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::theme::Theme;

pub struct TailView {
    pub logs: Vec<LogEntry>,
    pub is_paused: bool,
    pub target_worker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogEntry {
    pub timestamp: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub duration_ms: u32,
    pub message: String,
    pub raw_json: Option<String>,
    pub exceptions: Vec<String>,
    pub log_level: Option<String>,
    pub outcome: String,
}

impl TailView {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            is_paused: false,
            target_worker: "Live Stream".to_string(),
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn clear(&mut self) {
        self.logs.clear();
    }

    pub fn add_tick_log(&mut self, _tick_count: usize) {
        // Real-time logs are pushed via WebSocket events
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let stream_status = if self.is_paused {
            Span::styled("⏸ PAUSED", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("● LISTENING", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
        };

        let lines: Vec<Line> = if self.logs.is_empty() {
            vec![
                Line::from(vec![
                    Span::styled(" ⏳ Waiting for incoming Worker requests...", Style::default().fg(theme.info)),
                ]),
                Line::from(vec![
                    Span::styled("    Live HTTP requests, console.log outputs, and error traces will appear here automatically.", Style::default().fg(theme.text_dim)),
                ]),
            ]
        } else {
            self.logs
            .iter()
            .map(|entry| {
                let status_style = match entry.status {
                    200..=299 => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
                    400..=499 => Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
                };

                let method_style = match entry.method.as_str() {
                    "GET" => Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                    "POST" => Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                };

                Line::from(vec![
                    Span::styled(format!("[{}] ", entry.timestamp), Style::default().fg(theme.text_dim)),
                    Span::styled(format!("{:<5} ", entry.method), method_style),
                    Span::styled(format!("{:<3} ", entry.status), status_style),
                    Span::styled(format!("{:<4}ms ", entry.duration_ms), Style::default().fg(theme.text_dim)),
                    Span::styled(format!("{:<20} ", entry.url), Style::default().fg(Color::White)),
                    Span::styled("│ ", Style::default().fg(theme.border)),
                    Span::styled(&entry.message, Style::default().fg(theme.text)),
                ])
            })
            .collect()
        };

        let title = Line::from(vec![
            Span::raw(" Live Tail Console: "),
            Span::styled(&self.target_worker, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" ["),
            stream_status,
            Span::raw("] (Press 'Space' to toggle pause) "),
        ]);

        let log_widget = Paragraph::new(lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_focus)),
        );

        f.render_widget(log_widget, area);
    }
}
