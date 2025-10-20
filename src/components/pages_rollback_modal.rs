use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::theme::Theme;

pub struct ConfirmRollbackModal {
    pub project_name: String,
    pub deployment_id: String,
    pub short_id: String,
    pub branch: String,
    pub commit_message: String,
}

impl ConfirmRollbackModal {
    pub fn new(
        project_name: String,
        deployment_id: String,
        short_id: String,
        branch: String,
        commit_message: String,
    ) -> Self {
        Self {
            project_name,
            deployment_id,
            short_id,
            branch,
            commit_message,
        }
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

        let block = Block::default()
            .title(" ⚠️ Confirm Production Rollback ")
            .title_style(Style::default().fg(theme.warning).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(theme.warning));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Warning text
                Constraint::Length(4), // Deployment details
                Constraint::Length(2), // Prompt
            ])
            .margin(1)
            .split(popup_area);

        let warning = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Notice: ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                Span::styled(
                    "This will immediately promote this deployment to LIVE production traffic on Cloudflare edge.",
                    Style::default().fg(Color::White),
                ),
            ]),
        ]);
        f.render_widget(warning, chunks[0]);

        let details = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Project: ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.project_name, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled("Target Build: ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.short_id, Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({})", self.branch), Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::styled("Commit:  ", Style::default().fg(theme.text_dim)),
                Span::styled(&self.commit_message, Style::default().fg(Color::White)),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(details, chunks[1]);

        let prompt = Paragraph::new(Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.text_dim)),
            Span::styled("[y]", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled(" to Confirm Rollback   ", Style::default().fg(theme.text_dim)),
            Span::styled("[n / Esc]", Style::default().fg(theme.secondary)),
            Span::styled(" Cancel", Style::default().fg(theme.text_dim)),
        ]));
        f.render_widget(prompt, chunks[2]);
    }
}
