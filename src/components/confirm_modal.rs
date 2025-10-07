use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::db::DnsRecord;
use crate::theme::Theme;

pub struct ConfirmDeleteModal {
    pub record: DnsRecord,
}

impl ConfirmDeleteModal {
    pub fn new(record: DnsRecord) -> Self {
        Self { record }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 64.min(area.width.saturating_sub(4));
        let popup_height = 12.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" ⚠️ Delete DNS Record Confirmation ")
            .title_style(Style::default().fg(theme.error).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(theme.error));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Warning text
                Constraint::Length(3), // Record details
                Constraint::Length(2), // Prompt
            ])
            .margin(1)
            .split(popup_area);

        let warning = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Warning: ", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
                Span::styled("This will immediately delete the record from Cloudflare edge.", Style::default().fg(Color::White)),
            ]),
        ]);
        f.render_widget(warning, chunks[0]);

        let record_details = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Target: ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{:<6}", self.record.record_type), Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:<22}", self.record.name), Style::default().fg(Color::White)),
                Span::styled(format!("➔ {}", self.record.content), Style::default().fg(theme.info)),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(record_details, chunks[1]);

        let prompt = Paragraph::new(Line::from(vec![
            Span::styled("Press [y] to Confirm Delete   [n / Esc] Cancel", Style::default().fg(theme.text_dim)),
        ]));
        f.render_widget(prompt, chunks[2]);
    }
}
