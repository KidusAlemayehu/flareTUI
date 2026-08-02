use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Table, Row, Cell, Wrap},
    Frame,
};

use crate::cloudflare::client::ZoneSettings;
use crate::db::Zone;
use crate::theme::Theme;

pub struct ZoneInspectorModal {
    pub zone: Zone,
    pub settings: ZoneSettings,
    pub is_loading: bool,
    pub action_feedback: Option<String>,
}

impl ZoneInspectorModal {
    pub fn new(zone: Zone) -> Self {
        Self {
            zone,
            settings: ZoneSettings::default(),
            is_loading: false,
            action_feedback: None,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = crate::components::centered_rect(area, 78, 68, 110, 75, 18, 28);

        f.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Overview table
                Constraint::Length(5), // Settings cards
                Constraint::Min(4),    // Action key hints
            ])
            .margin(1)
            .split(popup_area);

        let block = Block::default()
            .title(format!(" 🌐 Zone Inspector: {} ", self.zone.name))
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        // 1. Overview Table
        let status_style = match self.zone.status.as_str() {
            "active" => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
        };

        let overview_rows = vec![
            Row::new(vec![
                Cell::from("Zone ID:").style(Style::default().fg(theme.text_dim)),
                Cell::from(self.zone.id.clone()).style(Style::default().fg(Color::White)),
                Cell::from("Plan:").style(Style::default().fg(theme.text_dim)),
                Cell::from(self.zone.plan.clone()).style(Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            ]),
            Row::new(vec![
                Cell::from("Status:").style(Style::default().fg(theme.text_dim)),
                Cell::from(format!("● {}", self.zone.status)).style(status_style),
                Cell::from("Created:").style(Style::default().fg(theme.text_dim)),
                Cell::from(self.zone.created_on.clone()).style(Style::default().fg(theme.text_dim)),
            ]),
        ];

        let overview_table = Table::new(
            overview_rows,
            [
                Constraint::Length(10),
                Constraint::Length(34),
                Constraint::Length(10),
                Constraint::Length(16),
            ],
        )
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(overview_table, chunks[0]);

        // 2. Settings Grid
        let ssl_val = self.settings.ssl.as_ref().map(|s| s.value.as_str()).unwrap_or("flexible");
        let dev_val = self.settings.development_mode.as_ref().map(|s| s.value.as_str()).unwrap_or("off");
        let sec_val = self.settings.security_level.as_ref().map(|s| s.value.as_str()).unwrap_or("medium");
        let tls_val = self.settings.min_tls_version.as_ref().map(|s| s.value.as_str()).unwrap_or("1.2");

        let ssl_style = match ssl_val {
            "strict" | "full" => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
            "flexible" => Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(theme.error),
        };

        let dev_style = if dev_val == "on" {
            Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_dim)
        };

        let sec_style = if sec_val == "under_attack" {
            Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.info)
        };

        let settings_paragraph = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("🔒 SSL/TLS Mode: ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{:<14}", ssl_val.to_uppercase()), ssl_style),
                Span::styled("🛡️ Security Level: ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{:<14}", sec_val.to_uppercase()), sec_style),
            ]),
            Line::from(vec![
                Span::styled("⚡ Dev Mode:     ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{:<14}", dev_val.to_uppercase()), dev_style),
                Span::styled("📜 Min TLS:        ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{:<14}", tls_val), Style::default().fg(Color::White)),
            ]),
        ])
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Edge Settings & Security ")
                .title_style(theme.title())
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(settings_paragraph, chunks[1]);

        // 3. Operational Actions & Feedback
        let feedback_line = if let Some(fb) = &self.action_feedback {
            Line::from(vec![
                Span::styled("● ", Style::default().fg(theme.success)),
                Span::styled(fb, Style::default().fg(Color::White)),
            ])
        } else if self.is_loading {
            Line::from(vec![
                Span::styled("⏳ Communicating with Cloudflare v4 API...", Style::default().fg(theme.warning)),
            ])
        } else {
            Line::from(vec![
                Span::styled("Press hotkey to trigger edge changes live on Cloudflare:", Style::default().fg(theme.text_dim)),
            ])
        };

        let actions_text = vec![
            feedback_line,
            Line::from(vec![
                Span::styled(" [s] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled("Cycle SSL Mode   ", Style::default().fg(Color::White)),
                Span::styled(" [d] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled("Toggle Dev Mode   ", Style::default().fg(Color::White)),
                Span::styled(" [u] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled("Under Attack Mode", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(" [p] ", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
                Span::styled("Purge Everything (Edge Cache)    ", Style::default().fg(theme.error)),
                Span::styled(" [Esc] ", Style::default().fg(theme.secondary)),
                Span::styled("Close Inspector", Style::default().fg(theme.text_dim)),
            ]),
        ];

        let actions_widget = Paragraph::new(actions_text).wrap(Wrap { trim: true });
        f.render_widget(actions_widget, chunks[2]);
    }
}
