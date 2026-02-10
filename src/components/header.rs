use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::theme::Theme;

pub struct Header {
    pub active_tab: usize,
    pub tabs: Vec<&'static str>,
    pub account_name: String,
    pub is_live: bool,
}

impl Header {
    pub fn new() -> Self {
        Self {
            active_tab: 0,
            tabs: vec![
                "1: Zones & DNS",
                "2: Workers",
                "3: Pages",
                "4: D1 & KV",
                "5: Live Tail",
                "6: Security & WAF",
            ],
            account_name: "Personal (kidus)".to_string(),
            is_live: false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3)])
            .split(area);

        // Top info bar
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(14),
                Constraint::Min(25),
                Constraint::Length(28),
            ])
            .split(chunks[0]);

        // Logo
        let logo = Paragraph::new(Line::from(vec![
            Span::styled(" ☁ flare", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("TUI ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));
        f.render_widget(logo, top_chunks[0]);

        // Account / Environment badge
        let live_badge = if self.is_live {
            Span::styled("● Cloudflare Live", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("○ Cached Mode (Press 'a' to auth)", Style::default().fg(theme.warning))
        };

        let badge = Paragraph::new(Line::from(vec![
            Span::styled("👤 Account: ", Style::default().fg(theme.text_dim)),
            Span::styled(&self.account_name, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            live_badge,
        ]));
        f.render_widget(badge, top_chunks[1]);

        // Cache mode indicator
        let cache_info = Paragraph::new(Line::from(vec![
            Span::styled("⚡ Cache: ", Style::default().fg(theme.text_dim)),
            Span::styled("SQLite Local", Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
        ]))
        .alignment(ratatui::layout::Alignment::Right);
        f.render_widget(cache_info, top_chunks[2]);

        // Navigation Tabs
        let titles: Vec<Line> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, &tab)| {
                if i == self.active_tab {
                    Line::from(Span::styled(
                        format!(" {tab} "),
                        Style::default()
                            .fg(Color::Black)
                            .bg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(
                        format!(" {tab} "),
                        Style::default().fg(theme.text_dim),
                    ))
                }
            })
            .collect();

        let tabs_widget = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.border)),
            )
            .select(self.active_tab);

        f.render_widget(tabs_widget, chunks[1]);
    }
}
