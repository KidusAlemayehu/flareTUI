use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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
    pub search_query: String,
    pub search_active: bool,
    pub follow_mode: bool,
    pub selected_idx: usize,
    pub scroll_offset: usize,
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
            search_query: String::new(),
            search_active: false,
            follow_mode: true,
            selected_idx: 0,
            scroll_offset: 0,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn toggle_follow(&mut self) {
        self.follow_mode = !self.follow_mode;
        if self.follow_mode {
            let total = self.filtered_logs().len();
            if total > 0 {
                self.selected_idx = total - 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.logs.clear();
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    pub fn toggle_search(&mut self) {
        self.search_active = !self.search_active;
    }

    pub fn input_char(&mut self, c: char) {
        self.search_query.push(c);
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    pub fn input_backspace(&mut self) {
        self.search_query.pop();
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_active = false;
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    pub fn filtered_logs(&self) -> Vec<(usize, &LogEntry)> {
        if self.search_query.trim().is_empty() {
            self.logs.iter().enumerate().collect()
        } else {
            let q = self.search_query.to_lowercase();
            self.logs
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    entry.url.to_lowercase().contains(&q)
                        || entry.message.to_lowercase().contains(&q)
                        || entry.method.to_lowercase().contains(&q)
                        || entry.status.to_string().contains(&q)
                        || entry.outcome.to_lowercase().contains(&q)
                        || entry.exceptions.iter().any(|e| e.to_lowercase().contains(&q))
                })
                .collect()
        }
    }

    pub fn selected_log(&self) -> Option<&LogEntry> {
        let filtered = self.filtered_logs();
        filtered.get(self.selected_idx).map(|(_, entry)| *entry)
    }

    pub fn nav_up(&mut self) {
        self.follow_mode = false;
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
        if self.selected_idx < self.scroll_offset {
            self.scroll_offset = self.selected_idx;
        }
    }

    pub fn nav_down(&mut self, visible_lines: usize) {
        let total = self.filtered_logs().len();
        if total > 0 && self.selected_idx + 1 < total {
            self.selected_idx += 1;
            if self.selected_idx >= self.scroll_offset + visible_lines.saturating_sub(1) {
                self.scroll_offset = self.selected_idx.saturating_sub(visible_lines.saturating_sub(2));
            }
            if self.selected_idx + 1 == total {
                self.follow_mode = true;
            }
        }
    }

    pub fn add_tick_log(&mut self, _tick_count: usize) {
        // Real-time logs are pushed via WebSocket events
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let (search_area, log_area) = if self.search_active || !self.search_query.is_empty() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(6)])
                .split(area);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, area)
        };

        if let Some(s_area) = search_area {
            let search_border_color = if self.search_active {
                theme.primary
            } else {
                theme.border
            };
            let cursor = if self.search_active { "█" } else { "" };
            let search_bar = Paragraph::new(Line::from(vec![
                Span::styled(" Filter Logs: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(&self.search_query, Style::default().fg(Color::White)),
                Span::styled(cursor, Style::default().fg(theme.primary)),
                Span::raw("   "),
                Span::styled("[Esc: Clear / Close]", Style::default().fg(theme.text_dim)),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(search_border_color)),
            );
            f.render_widget(search_bar, s_area);
        }

        let stream_status = if self.is_paused {
            Span::styled("⏸ PAUSED", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("● LISTENING", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
        };

        let follow_status = if self.follow_mode {
            Span::styled("⬇ FOLLOW", Style::default().fg(theme.info).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("⇕ MANUAL", Style::default().fg(theme.text_dim))
        };

        let filtered = self.filtered_logs();
        let total_count = self.logs.len();
        let visible_lines = log_area.height.saturating_sub(2) as usize;

        // Auto-follow: pin selection and scroll offset to bottom if follow mode is active
        let effective_selected_idx = if self.follow_mode && !filtered.is_empty() {
            filtered.len().saturating_sub(1)
        } else {
            self.selected_idx.min(filtered.len().saturating_sub(1))
        };

        let effective_scroll_offset = if self.follow_mode && !filtered.is_empty() {
            if filtered.len() > visible_lines {
                filtered.len() - visible_lines
            } else {
                0
            }
        } else {
            self.scroll_offset
        };

        let lines: Vec<Line> = if filtered.is_empty() {
            if total_count == 0 {
                vec![
                    Line::from(vec![
                        Span::styled(" ⏳ Waiting for incoming Worker requests...", Style::default().fg(theme.info)),
                    ]),
                    Line::from(vec![
                        Span::styled("    Live HTTP requests, console.log outputs, and error traces will appear here automatically.", Style::default().fg(theme.text_dim)),
                    ]),
                ]
            } else {
                vec![
                    Line::from(vec![
                        Span::styled(format!(" No log entries matched filter: '{}'", self.search_query), Style::default().fg(theme.warning)),
                    ]),
                    Line::from(vec![
                        Span::styled(" Press Esc to clear filter and view all incoming logs.", Style::default().fg(theme.text_dim)),
                    ]),
                ]
            }
        } else {
            filtered
                .iter()
                .skip(effective_scroll_offset)
                .take(visible_lines)
                .enumerate()
                .map(|(rel_idx, &(_orig_idx, entry))| {
                    let actual_idx = effective_scroll_offset + rel_idx;
                    let is_selected = actual_idx == effective_selected_idx;
                    let prefix = if is_selected { "▶ " } else { "  " };

                    let status_style = match entry.status {
                        100..=199 => Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                        200..=299 => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
                        300..=399 => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        400..=499 => Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
                        _ => Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
                    };

                    let method_style = match entry.method.as_str() {
                        "GET" => Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                        "POST" => Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
                        "PUT" | "PATCH" => Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
                        "DELETE" => Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
                        "SYS" | "ATTACH" => Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
                        _ => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                    };

                    let row_style = if is_selected {
                        theme.selected_row()
                    } else {
                        Style::default()
                    };

                    Line::from(vec![
                        Span::styled(prefix, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("[{}] ", entry.timestamp), Style::default().fg(theme.text_dim)),
                        Span::styled(format!("{:<6} ", entry.method), method_style),
                        Span::styled(format!("{:<3} ", entry.status), status_style),
                        Span::styled(format!("{:<4}ms ", entry.duration_ms), Style::default().fg(theme.text_dim)),
                        Span::styled(format!("{:<22} ", entry.url), Style::default().fg(Color::White)),
                        Span::styled("│ ", Style::default().fg(theme.border)),
                        Span::styled(&entry.message, Style::default().fg(theme.text)),
                    ])
                    .style(row_style)
                })
                .collect()
        };

        let title = Line::from(vec![
            Span::raw(" Live Tail Console: "),
            Span::styled(&self.target_worker, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" ["),
            stream_status,
            Span::raw("] ["),
            follow_status,
            Span::raw("] ["),
            Span::styled(format!("{}/{}", filtered.len(), total_count), Style::default().fg(Color::White)),
            Span::raw(" entries] "),
        ]);

        let log_widget = Paragraph::new(lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_focus)),
        );

        f.render_widget(log_widget, log_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tail_view_filtering() {
        let mut view = TailView::new();
        view.logs.push(LogEntry {
            timestamp: "12:00:00".to_string(),
            method: "GET".to_string(),
            url: "/api/users".to_string(),
            status: 200,
            duration_ms: 12,
            message: "User queried".to_string(),
            ..Default::default()
        });
        view.logs.push(LogEntry {
            timestamp: "12:00:01".to_string(),
            method: "POST".to_string(),
            url: "/api/login".to_string(),
            status: 500,
            duration_ms: 45,
            message: "Auth failed".to_string(),
            ..Default::default()
        });

        assert_eq!(view.filtered_logs().len(), 2);

        view.search_query = "login".to_string();
        let filtered = view.filtered_logs();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1.url, "/api/login");

        view.search_query = "500".to_string();
        assert_eq!(view.filtered_logs().len(), 1);

        view.clear_search();
        assert_eq!(view.filtered_logs().len(), 2);
    }

    #[test]
    fn test_tail_view_follow_mode() {
        let mut view = TailView::new();
        assert!(view.follow_mode);

        view.nav_up();
        assert!(!view.follow_mode);

        view.toggle_follow();
        assert!(view.follow_mode);
    }
}
