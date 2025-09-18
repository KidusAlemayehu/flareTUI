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

pub struct LogEntry {
    pub timestamp: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub duration_ms: u32,
    pub message: String,
}

impl TailView {
    pub fn new() -> Self {
        let initial_logs = vec![
            LogEntry {
                timestamp: "03:14:02.102".to_string(),
                method: "GET".to_string(),
                url: "/v1/auth/session".to_string(),
                status: 200,
                duration_ms: 12,
                message: "Session token validated successfully [cache-hit: true]".to_string(),
            },
            LogEntry {
                timestamp: "03:14:05.441".to_string(),
                method: "POST".to_string(),
                url: "/v1/auth/login".to_string(),
                status: 200,
                duration_ms: 48,
                message: "User authentication succeeded: kidusalemayehu705@gmail.com".to_string(),
            },
            LogEntry {
                timestamp: "03:14:09.890".to_string(),
                method: "GET".to_string(),
                url: "/v1/auth/unknown".to_string(),
                status: 404,
                duration_ms: 4,
                message: "Route not matched in worker router".to_string(),
            },
            LogEntry {
                timestamp: "03:14:14.215".to_string(),
                method: "POST".to_string(),
                url: "/v1/auth/refresh".to_string(),
                status: 200,
                duration_ms: 22,
                message: "Rotated refresh token for session s-9841".to_string(),
            },
        ];

        Self {
            logs: initial_logs,
            is_paused: false,
            target_worker: "edge-auth-guard".to_string(),
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn clear(&mut self) {
        self.logs.clear();
    }

    pub fn add_tick_log(&mut self, tick_count: usize) {
        if self.is_paused {
            return;
        }

        // Add periodic realistic tail entry
        if tick_count % 8 == 0 {
            let methods = ["GET", "POST", "PUT"];
            let paths = ["/api/health", "/v1/auth/session", "/v1/users/me", "/static/asset.js"];
            let m = methods[tick_count % methods.len()];
            let p = paths[tick_count % paths.len()];

            self.logs.push(LogEntry {
                timestamp: format!("03:14:{:02}.{:03}", (tick_count % 60), (tick_count * 13) % 1000),
                method: m.to_string(),
                url: p.to_string(),
                status: 200,
                duration_ms: (15 + (tick_count % 35)) as u32,
                message: format!("Worker invocation completed [trace: cf-trace-{tick_count}]"),
            });

            if self.logs.len() > 100 {
                self.logs.remove(0);
            }
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let stream_status = if self.is_paused {
            Span::styled("⏸ PAUSED", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("● LIVE STREAMING", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
        };

        let lines: Vec<Line> = self
            .logs
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
            .collect();

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
