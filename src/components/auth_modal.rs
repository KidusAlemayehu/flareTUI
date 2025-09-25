use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::theme::Theme;

pub struct AuthModal {
    pub input_token: String,
    pub is_validating: bool,
    pub error_msg: Option<String>,
}

impl AuthModal {
    pub fn new() -> Self {
        Self {
            input_token: String::new(),
            is_validating: false,
            error_msg: None,
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if !c.is_control() {
            self.input_token.push(c);
            self.error_msg = None;
        }
    }

    pub fn handle_backspace(&mut self) {
        self.input_token.pop();
        self.error_msg = None;
    }

    pub fn clear(&mut self) {
        self.input_token.clear();
        self.is_validating = false;
        self.error_msg = None;
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 72.min(area.width.saturating_sub(4));
        let popup_height = 14.min(area.height.saturating_sub(2));

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
                Constraint::Length(4), // Instructions
                Constraint::Length(3), // Input box
                Constraint::Length(3), // Status & Hints
            ])
            .margin(1)
            .split(popup_area);

        let block = Block::default()
            .title(" 🔐 Connect Cloudflare Account (API Token) ")
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        // 1. Instructions
        let instructions = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Create an API Token with Zone & Worker read/edit permissions at:", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::styled("👉 https://dash.cloudflare.com/profile/api-tokens", Style::default().fg(theme.info).add_modifier(Modifier::UNDERLINED)),
            ]),
        ]);
        f.render_widget(instructions, chunks[0]);

        // 2. Input Box
        let masked_input = if self.input_token.is_empty() {
            Span::styled("Paste or type your Cloudflare API Token here...", Style::default().fg(theme.text_dim))
        } else {
            // Mask first characters, show last 4
            let len = self.input_token.len();
            if len <= 8 {
                Span::styled("•".repeat(len), Style::default().fg(Color::White))
            } else {
                let masked = format!("{}...{}", "•".repeat(12), &self.input_token[len - 4..]);
                Span::styled(masked, Style::default().fg(Color::White))
            }
        };

        let input_box = Paragraph::new(Line::from(vec![
            Span::styled(" Token: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            masked_input,
            Span::styled("█", Style::default().fg(theme.primary)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.secondary)),
        );
        f.render_widget(input_box, chunks[1]);

        // 3. Status & Hints
        let status_line = if let Some(err) = &self.error_msg {
            Line::from(vec![
                Span::styled("❌ ", Style::default().fg(theme.error)),
                Span::styled(err, Style::default().fg(theme.error)),
            ])
        } else if self.is_validating {
            Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(theme.warning)),
                Span::styled("Validating token against Cloudflare API v4...", Style::default().fg(theme.warning)),
            ])
        } else {
            Line::from(vec![
                Span::styled("[Enter] Connect & Sync   [Esc] Cancel", Style::default().fg(theme.text_dim)),
            ])
        };

        let footer = Paragraph::new(status_line);
        f.render_widget(footer, chunks[2]);
    }
}
