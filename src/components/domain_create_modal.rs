use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
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
        // Responsive popup sizing: adapt dynamically based on window size
        let popup_area = crate::components::centered_rect(area, 75, 64, 96, 75, 16, 26);
        let popup_width = popup_area.width;
        let popup_height = popup_area.height;

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Register Domain & Onboard Zone ")
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        let is_compact_height = popup_height < 21;
        let subtitle_len = if popup_height < 18 {
            0
        } else if is_compact_height {
            1
        } else {
            2
        };
        let toggles_len = if is_compact_height { 5 } else { 6 };
        let footer_len = if is_compact_height { 1 } else { 2 };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(subtitle_len), // Subtitle
                Constraint::Length(3),            // Domain Name
                Constraint::Length(3),            // Plan Selection
                Constraint::Length(toggles_len),  // Toggles (JumpStart, AutoRenew, Privacy)
                Constraint::Length(2),            // Error or status
                Constraint::Length(footer_len),   // Keybindings Footer
            ])
            .margin(1)
            .split(popup_area);

        // Subtitle (with text wrapping and responsive phrasing)
        if subtitle_len > 0 {
            let subtitle_text = if popup_width >= 72 {
                "Add an existing domain to Cloudflare Edge DNS or register via Cloudflare Registrar"
            } else {
                "Add domain to Cloudflare DNS or register with Registrar"
            };
            let subtitle = Paragraph::new(Line::from(vec![
                Span::styled(subtitle_text, Style::default().fg(theme.text_dim)),
            ]))
            .wrap(Wrap { trim: true });
            f.render_widget(subtitle, chunks[0]);
        }

        // Domain Name Input
        let domain_border_style = if self.active_field == DomainModalField::DomainName {
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.secondary)
        };

        let cursor_char = if self.active_field == DomainModalField::DomainName { "█" } else { "" };
        let placeholder = if popup_width >= 70 {
            "example.com (e.g. myapp.dev)"
        } else {
            "example.com"
        };

        let domain_input = Paragraph::new(Line::from(vec![
            Span::styled(" Name: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(
                if self.domain_name.is_empty() { placeholder } else { &self.domain_name },
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

        let plan_hint = if popup_width >= 74 {
            "  (Press Space to cycle tier: Free -> Pro -> Business)"
        } else if popup_width >= 58 {
            "  (Space to cycle)"
        } else {
            ""
        };

        let plan_p = Paragraph::new(Line::from(vec![
            Span::styled(" Plan: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(format!("◀ {} ▶", self.plan_name()), Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
            Span::styled(plan_hint, Style::default().fg(theme.text_dim)),
        ]))
        .block(
            Block::default()
                .title(" Cloudflare Plan Tier ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(plan_border_style),
        );
        f.render_widget(plan_p, chunks[2]);

        // Configuration Options (Stacked responsive rows for 100% readability)
        let js_cursor = if self.active_field == DomainModalField::JumpStart { " ➔ " } else { "    " };
        let ar_cursor = if self.active_field == DomainModalField::AutoRenew { " ➔ " } else { "    " };
        let pv_cursor = if self.active_field == DomainModalField::Privacy { " ➔ " } else { "    " };

        let js_check = if self.jump_start { "[✔] " } else { "[ ] " };
        let ar_check = if self.auto_renew { "[✔] " } else { "[ ] " };
        let pv_check = if self.privacy { "[✔] " } else { "[ ] " };

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

        let (js_label, ar_label, pv_label) = if popup_width >= 74 {
            (
                "Jump Start  : Scan & import existing DNS records automatically",
                "Auto-Renew  : Automatically renew domain registration annually",
                "WHOIS Privacy: Redact registrant personal info from public WHOIS",
            )
        } else if popup_width >= 60 {
            (
                "Jump Start  : Scan & import existing DNS records",
                "Auto-Renew  : Automatically renew domain annually",
                "WHOIS Privacy: Redact registrant WHOIS info",
            )
        } else {
            (
                "Jump Start (Scan DNS)",
                "Auto-Renew (Annual)",
                "WHOIS Privacy (Redacted)",
            )
        };

        let mut toggle_lines = vec![
            Line::from(vec![
                Span::styled(js_cursor, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(js_check, js_style),
                Span::styled(js_label, js_style),
            ]),
            Line::from(vec![
                Span::styled(ar_cursor, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(ar_check, ar_style),
                Span::styled(ar_label, ar_style),
            ]),
            Line::from(vec![
                Span::styled(pv_cursor, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(pv_check, pv_style),
                Span::styled(pv_label, pv_style),
            ]),
        ];

        if !is_compact_height {
            let nav_hint = if popup_width >= 70 {
                "    [Tab/Arrows] Navigate fields    [Space] Toggle selected checkbox"
            } else {
                "    [Tab] Navigate    [Space] Toggle"
            };
            toggle_lines.push(Line::from(vec![
                Span::styled(nav_hint, Style::default().fg(theme.text_dim)),
            ]));
        }

        let toggles_p = Paragraph::new(toggle_lines)
            .block(
                Block::default()
                    .title(" Configuration Options ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.secondary)),
            );
        f.render_widget(toggles_p, chunks[3]);

        // Error message or submitting status (with text wrapping)
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
                Span::styled(
                    if popup_width >= 70 {
                        " [Ready] Enter domain details and press [Enter] to submit"
                    } else {
                        " [Ready] Press [Enter] to submit"
                    },
                    Style::default().fg(theme.text_dim)
                ),
            ])
        };
        f.render_widget(Paragraph::new(status_line).wrap(Wrap { trim: true }), chunks[4]);

        // Keybindings footer (responsive phrasing based on width)
        let footer_spans = if popup_width >= 78 {
            vec![
                Span::styled("[Tab/↓] Next  ", Style::default().fg(theme.text_dim)),
                Span::styled("[Shift+Tab/↑] Prev  ", Style::default().fg(theme.text_dim)),
                Span::styled("[Space] Toggle/Cycle  ", Style::default().fg(theme.secondary)),
                Span::styled("[Enter] Create & Onboard  ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled("[Esc] Cancel", Style::default().fg(theme.text_dim)),
            ]
        } else {
            vec![
                Span::styled("[Tab] Next  ", Style::default().fg(theme.text_dim)),
                Span::styled("[Space] Toggle  ", Style::default().fg(theme.secondary)),
                Span::styled("[Enter] Onboard  ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled("[Esc] Cancel", Style::default().fg(theme.text_dim)),
            ]
        };
        let footer = Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
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

    #[test]
    fn test_domain_create_modal_rendering_compact_and_narrow() {
        let mut modal = DomainCreateModal::new();
        modal.domain_name = "subdomain.example-production.org".to_string();
        modal.error_msg = Some("Domain validation warning: please check TLD syntax".to_string());
        let theme = Theme::default();

        // Narrow terminal (64 cols x 20 rows)
        let backend_narrow = ratatui::backend::TestBackend::new(64, 20);
        let mut terminal_narrow = ratatui::Terminal::new(backend_narrow).unwrap();
        terminal_narrow.draw(|f| modal.render(f, f.area(), &theme)).unwrap();

        // Compact height terminal (80 cols x 16 rows)
        let backend_compact = ratatui::backend::TestBackend::new(80, 16);
        let mut terminal_compact = ratatui::Terminal::new(backend_compact).unwrap();
        terminal_compact.draw(|f| modal.render(f, f.area(), &theme)).unwrap();
    }
}
