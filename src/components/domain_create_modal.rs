use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainModalField {
    DomainName,
    Plan,
    JumpStart,
    AutoRenew,
    Privacy,
}

pub struct DomainCreateModal {
    pub domain_name: String,
    pub selected_plan_idx: usize,
    pub jump_start: bool,
    pub auto_renew: bool,
    pub privacy: bool,
    pub active_field: DomainModalField,
    pub error_msg: Option<String>,
    pub is_submitting: bool,
}

impl DomainCreateModal {
    pub fn new() -> Self {
        Self {
            domain_name: String::new(),
            selected_plan_idx: 0,
            jump_start: true,
            auto_renew: true,
            privacy: true,
            active_field: DomainModalField::DomainName,
            error_msg: None,
            is_submitting: false,
        }
    }

    pub fn plan_name(&self) -> &'static str {
        match self.selected_plan_idx {
            0 => "Free ($0/mo)",
            1 => "Pro ($20/mo)",
            2 => "Business ($200/mo)",
            _ => "Free ($0/mo)",
        }
    }

    pub fn cycle_plan(&mut self, forward: bool) {
        if forward {
            self.selected_plan_idx = (self.selected_plan_idx + 1) % 3;
        } else if self.selected_plan_idx == 0 {
            self.selected_plan_idx = 2;
        } else {
            self.selected_plan_idx -= 1;
        }
    }

    pub fn next_field(&mut self) {
        self.active_field = match self.active_field {
            DomainModalField::DomainName => DomainModalField::Plan,
            DomainModalField::Plan => DomainModalField::JumpStart,
            DomainModalField::JumpStart => DomainModalField::AutoRenew,
            DomainModalField::AutoRenew => DomainModalField::Privacy,
            DomainModalField::Privacy => DomainModalField::DomainName,
        };
    }

    pub fn prev_field(&mut self) {
        self.active_field = match self.active_field {
            DomainModalField::DomainName => DomainModalField::Privacy,
            DomainModalField::Plan => DomainModalField::DomainName,
            DomainModalField::JumpStart => DomainModalField::Plan,
            DomainModalField::AutoRenew => DomainModalField::JumpStart,
            DomainModalField::Privacy => DomainModalField::AutoRenew,
        };
    }

    pub fn handle_char(&mut self, c: char) {
        match self.active_field {
            DomainModalField::DomainName => {
                if c.is_alphanumeric() || c == '.' || c == '-' {
                    self.domain_name.push(c.to_ascii_lowercase());
                    self.error_msg = None;
                }
            }
            DomainModalField::Plan => {
                if c == ' ' {
                    self.cycle_plan(true);
                }
            }
            DomainModalField::JumpStart => {
                if c == ' ' {
                    self.jump_start = !self.jump_start;
                }
            }
            DomainModalField::AutoRenew => {
                if c == ' ' {
                    self.auto_renew = !self.auto_renew;
                }
            }
            DomainModalField::Privacy => {
                if c == ' ' {
                    self.privacy = !self.privacy;
                }
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.active_field == DomainModalField::DomainName {
            self.domain_name.pop();
            self.error_msg = None;
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let name = self.domain_name.trim();
        if name.is_empty() {
            return Err("Domain name cannot be empty".to_string());
        }
        if !name.contains('.') {
            return Err("Domain must include a valid top-level domain (e.g., example.com)".to_string());
        }
        if name.starts_with('.') || name.ends_with('.') || name.starts_with('-') || name.ends_with('-') {
            return Err("Domain cannot start or end with a dot or hyphen".to_string());
        }
        if name.chars().any(|c| !c.is_alphanumeric() && c != '.' && c != '-') {
            return Err("Domain contains invalid characters".to_string());
        }
        Ok(())
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 72.min(area.width.saturating_sub(4));
        let popup_height = 20.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Register Domain & Onboard Zone ")
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Subtitle
                Constraint::Length(3), // Domain Name
                Constraint::Length(3), // Plan Selection
                Constraint::Length(4), // Toggles (JumpStart, AutoRenew, Privacy)
                Constraint::Length(2), // Error or status
                Constraint::Length(2), // Keybindings Footer
            ])
            .margin(1)
            .split(popup_area);

        // Subtitle
        let subtitle = Paragraph::new(Line::from(vec![
            Span::styled("Add a new domain to Cloudflare Edge DNS or register via Cloudflare Registrar", Style::default().fg(theme.text_dim)),
        ]));
        f.render_widget(subtitle, chunks[0]);

        // Domain Name Input
        let domain_border_style = if self.active_field == DomainModalField::DomainName {
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.secondary)
        };

        let cursor_char = if self.active_field == DomainModalField::DomainName { "█" } else { "" };
        let domain_input = Paragraph::new(Line::from(vec![
            Span::styled(" Name: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(
                if self.domain_name.is_empty() { "example.com" } else { &self.domain_name },
                if self.domain_name.is_empty() { Style::default().fg(theme.text_dim) } else { Style::default().fg(Color::White) },
            ),
            Span::styled(cursor_char, Style::default().fg(theme.primary)),
        ]))
        .block(
            Block::default()
                .title(" Domain Name (FQDN) ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(domain_border_style),
        );
        f.render_widget(domain_input, chunks[1]);

        // Plan Selection
        let plan_border_style = if self.active_field == DomainModalField::Plan {
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.secondary)
        };

        let plan_p = Paragraph::new(Line::from(vec![
            Span::styled(" Plan: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(format!("◀ {} ▶", self.plan_name()), Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
            Span::styled("  (Press Space to cycle plan tier)", Style::default().fg(theme.text_dim)),
        ]))
        .block(
            Block::default()
                .title(" Cloudflare Plan Tier ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(plan_border_style),
        );
        f.render_widget(plan_p, chunks[2]);

        // Toggles Box
        let js_checkbox = if self.jump_start { "[✔] Scan Existing DNS (Jump Start)" } else { "[ ] Scan Existing DNS (Jump Start)" };
        let ar_checkbox = if self.auto_renew { "[✔] Enable Auto-Renew" } else { "[ ] Enable Auto-Renew" };
        let pv_checkbox = if self.privacy { "[✔] WHOIS Privacy Protection" } else { "[ ] WHOIS Privacy Protection" };

        let js_style = if self.active_field == DomainModalField::JumpStart {
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
        } else if self.jump_start {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.text_dim)
        };

        let ar_style = if self.active_field == DomainModalField::AutoRenew {
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
        } else if self.auto_renew {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.text_dim)
        };

        let pv_style = if self.active_field == DomainModalField::Privacy {
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
        } else if self.privacy {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.text_dim)
        };

        let toggles_p = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(format!("  {}    ", js_checkbox), js_style),
                Span::styled(format!("  {}    ", ar_checkbox), ar_style),
                Span::styled(format!("  {}", pv_checkbox), pv_style),
            ]),
            Line::from(vec![
                Span::styled("  Navigate with Tab/Arrows, toggle checkboxes with [Space]", Style::default().fg(theme.text_dim)),
            ]),
        ])
        .block(
            Block::default()
                .title(" Configuration Options ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.secondary)),
        );
        f.render_widget(toggles_p, chunks[3]);

        // Error message or submitting status
        let status_line = if let Some(err) = &self.error_msg {
            Line::from(vec![
                Span::styled(" Error: ", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
                Span::styled(err, Style::default().fg(theme.error)),
            ])
        } else if self.is_submitting {
            Line::from(vec![
                Span::styled(" Onboarding domain to Cloudflare Edge API v4...", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            ])
        } else {
            Line::from(vec![
                Span::styled(" [Ready] Enter domain details and press [Enter] to submit", Style::default().fg(theme.text_dim)),
            ])
        };
        f.render_widget(Paragraph::new(status_line), chunks[4]);

        // Keybindings footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[Tab]/[↓] Next  ", Style::default().fg(theme.text_dim)),
            Span::styled("[Shift+Tab]/[↑] Prev  ", Style::default().fg(theme.text_dim)),
            Span::styled("[Space] Toggle/Cycle  ", Style::default().fg(theme.secondary)),
            Span::styled("[Enter] Create & Onboard  ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled("[Esc] Cancel", Style::default().fg(theme.text_dim)),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(footer, chunks[5]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_create_modal_fields_navigation() {
        let mut modal = DomainCreateModal::new();
        assert_eq!(modal.active_field, DomainModalField::DomainName);

        modal.next_field();
        assert_eq!(modal.active_field, DomainModalField::Plan);

        modal.next_field();
        assert_eq!(modal.active_field, DomainModalField::JumpStart);

        modal.next_field();
        assert_eq!(modal.active_field, DomainModalField::AutoRenew);

        modal.next_field();
        assert_eq!(modal.active_field, DomainModalField::Privacy);

        modal.next_field();
        assert_eq!(modal.active_field, DomainModalField::DomainName);

        modal.prev_field();
        assert_eq!(modal.active_field, DomainModalField::Privacy);
    }

    #[test]
    fn test_domain_create_modal_plan_cycling_and_toggles() {
        let mut modal = DomainCreateModal::new();
        assert_eq!(modal.selected_plan_idx, 0);
        assert!(modal.plan_name().contains("Free"));

        modal.cycle_plan(true);
        assert_eq!(modal.selected_plan_idx, 1);
        assert!(modal.plan_name().contains("Pro"));

        modal.cycle_plan(true);
        assert_eq!(modal.selected_plan_idx, 2);
        assert!(modal.plan_name().contains("Business"));

        modal.cycle_plan(true);
        assert_eq!(modal.selected_plan_idx, 0);

        modal.active_field = DomainModalField::JumpStart;
        assert!(modal.jump_start);
        modal.handle_char(' ');
        assert!(!modal.jump_start);

        modal.active_field = DomainModalField::AutoRenew;
        assert!(modal.auto_renew);
        modal.handle_char(' ');
        assert!(!modal.auto_renew);

        modal.active_field = DomainModalField::Privacy;
        assert!(modal.privacy);
        modal.handle_char(' ');
        assert!(!modal.privacy);
    }

    #[test]
    fn test_domain_create_modal_validation() {
        let mut modal = DomainCreateModal::new();
        assert!(modal.validate().is_err());

        modal.handle_char('m');
        modal.handle_char('y');
        modal.handle_char('s');
        modal.handle_char('i');
        modal.handle_char('t');
        modal.handle_char('e');
        assert_eq!(modal.domain_name, "mysite");
        assert!(modal.validate().is_err()); // missing dot/TLD

        modal.handle_char('.');
        modal.handle_char('c');
        modal.handle_char('o');
        modal.handle_char('m');
        assert_eq!(modal.domain_name, "mysite.com");
        assert!(modal.validate().is_ok());

        modal.handle_backspace();
        assert_eq!(modal.domain_name, "mysite.co");
    }

    #[test]
    fn test_domain_create_modal_rendering() {
        let modal = DomainCreateModal::new();
        let backend = ratatui::backend::TestBackend::new(100, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal.draw(|f| modal.render(f, f.area(), &theme)).unwrap();
    }
}
