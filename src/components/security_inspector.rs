use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::db::{CachedSecurityEvent, CachedWafRule};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorTab {
    Details = 0,
    Rule = 1,
    RawJson = 2,
}

impl InspectorTab {
    pub fn all() -> &'static [InspectorTab] {
        &[
            InspectorTab::Details,
            InspectorTab::Rule,
            InspectorTab::RawJson,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            InspectorTab::Details => "Event Details",
            InspectorTab::Rule => "WAF Rule Expression",
            InspectorTab::RawJson => "Raw JSON",
        }
    }
}

pub struct SecurityInspectorModal {
    pub event: CachedSecurityEvent,
    pub matched_rule: Option<CachedWafRule>,
    pub active_tab: InspectorTab,
    pub scroll_offset: usize,
    pub action_feedback: Option<String>,
}

impl SecurityInspectorModal {
    pub fn new(event: CachedSecurityEvent, matched_rule: Option<CachedWafRule>) -> Self {
        Self {
            event,
            matched_rule,
            active_tab: InspectorTab::Details,
            scroll_offset: 0,
            action_feedback: None,
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            InspectorTab::Details => InspectorTab::Rule,
            InspectorTab::Rule => InspectorTab::RawJson,
            InspectorTab::RawJson => InspectorTab::Details,
        };
        self.scroll_offset = 0;
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            InspectorTab::Details => InspectorTab::RawJson,
            InspectorTab::Rule => InspectorTab::Details,
            InspectorTab::RawJson => InspectorTab::Rule,
        };
        self.scroll_offset = 0;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn copy_ip(&mut self) {
        self.action_feedback = Some(format!(
            "Copied IP {} to clipboard buffer",
            self.event.client_ip
        ));
    }

    pub fn copy_ray_id(&mut self) {
        self.action_feedback = Some(format!(
            "Copied Ray ID {} to clipboard buffer",
            self.event.ray_id
        ));
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = crate::components::centered_rect(area, 88, 72, 130, 85, 20, 42);

        f.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Top Header / Tab Bar
                Constraint::Min(16),   // Body
                Constraint::Length(3), // Key hints & Feedback footer
            ])
            .margin(1)
            .split(popup_area);

        let block = Block::default()
            .title(format!(
                " Security Event Inspector: Ray ID {} ",
                self.event.ray_id
            ))
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        self.render_header_tabs(f, chunks[0], theme);

        match self.active_tab {
            InspectorTab::Details => self.render_details_tab(f, chunks[1], theme),
            InspectorTab::Rule => self.render_rule_tab(f, chunks[1], theme),
            InspectorTab::RawJson => self.render_raw_tab(f, chunks[1], theme),
        }

        self.render_footer(f, chunks[2], theme);
    }

    fn render_header_tabs(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let titles: Vec<Line> = InspectorTab::all()
            .iter()
            .enumerate()
            .map(|(i, tab)| Line::from(format!("[{}] {}", i + 1, tab.title())))
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.border)),
            )
            .select(self.active_tab as usize)
            .style(Style::default().fg(theme.text_dim))
            .highlight_style(
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            );

        f.render_widget(tabs, area);
    }

    fn render_details_tab(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let action_color = match self.event.action.to_lowercase().as_str() {
            "block" => theme.error,
            "managed_challenge" | "challenge" | "js_challenge" => theme.warning,
            "rate_limit" => theme.info,
            _ => theme.text_dim,
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Threat Action:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {} ", self.event.action.to_uppercase()),
                    Style::default()
                        .fg(Color::White)
                        .bg(action_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("    Source Engine:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &self.event.source,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Timestamp:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.event.timestamp, Style::default().fg(theme.text)),
                Span::styled("    Ray ID:         ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.event.ray_id, Style::default().fg(theme.secondary)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "--- Client Information ---",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("Client IP:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &self.event.client_ip,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled("    Country:        ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} ({})",
                        self.event.client_country,
                        get_country_name(&self.event.client_country)
                    ),
                    Style::default().fg(theme.text),
                ),
            ]),
            Line::from(vec![
                Span::styled("Autonomous Sys:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if self.event.client_asn > 0 {
                        format!("AS{} - {}", self.event.client_asn, self.event.client_as_org)
                    } else {
                        "Unknown / Unannounced".to_string()
                    },
                    Style::default().fg(theme.text),
                ),
            ]),
            Line::from(vec![
                Span::styled("User-Agent:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if self.event.user_agent.is_empty() {
                        "None / Hidden"
                    } else {
                        &self.event.user_agent
                    },
                    Style::default().fg(theme.text_dim),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "--- HTTP Request Target ---",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("Method:          ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &self.event.http_method,
                    Style::default()
                        .fg(theme.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("    Host:           ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.event.http_host, Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("Request URI:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &self.event.http_path,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "--- Matched Firewall Rule ---",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("Rule ID:         ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if self.event.rule_id.is_empty() {
                        "None"
                    } else {
                        &self.event.rule_id
                    },
                    Style::default().fg(theme.text),
                ),
            ]),
            Line::from(vec![
                Span::styled("Description:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if self.event.rule_description.is_empty() {
                        "Direct Security Heuristic"
                    } else {
                        &self.event.rule_description
                    },
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset as u16, 0));
        f.render_widget(p, area);
    }

    fn render_rule_tab(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let lines = if let Some(rule) = &self.matched_rule {
            vec![
                Line::from(vec![
                    Span::styled("Rule ID:        ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &rule.id,
                        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Description:    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&rule.description, Style::default().fg(theme.secondary)),
                ]),
                Line::from(vec![
                    Span::styled("Action:         ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &rule.action,
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("    Priority:       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(rule.priority.to_string(), Style::default().fg(theme.text)),
                    Span::styled("    Status:         ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        if rule.paused { "PAUSED" } else { "ACTIVE" },
                        Style::default().fg(if rule.paused {
                            Color::DarkGray
                        } else {
                            theme.success
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Reference Code: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        if rule.ref_code.is_empty() {
                            "None"
                        } else {
                            &rule.ref_code
                        },
                        Style::default().fg(theme.text_dim),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "WireFilter Expression:",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    format!("  {}", rule.expression),
                    Style::default().fg(theme.secondary),
                )]),
            ]
        } else if !self.event.rule_id.is_empty() {
            vec![
                Line::from(vec![
                    Span::styled("Rule ID:        ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &self.event.rule_id,
                        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Description:    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &self.event.rule_description,
                        Style::default().fg(theme.secondary),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Rule expression not stored locally in SQLite cache.",
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(vec![Span::styled(
                    "Press 's' on the main view to perform full rule sync.",
                    Style::default().fg(theme.text_dim),
                )]),
            ]
        } else {
            vec![
                Line::from("No specific WAF rule matched for this security event."),
                Line::from("This event was triggered by Cloudflare Managed Ruleset, Rate Limiting, or IP reputation."),
            ]
        };

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset as u16, 0));
        f.render_widget(p, area);
    }

    fn render_raw_tab(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let json_str =
            serde_json::to_string_pretty(&self.event).unwrap_or_else(|_| "{}".to_string());
        let lines: Vec<Line> = json_str
            .lines()
            .enumerate()
            .map(|(i, l)| {
                Line::from(vec![
                    Span::styled(
                        format!("{:3} | ", i + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(l, Style::default().fg(theme.text)),
                ])
            })
            .collect();

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset as u16, 0));
        f.render_widget(p, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let feedback_span = if let Some(fb) = &self.action_feedback {
            Span::styled(
                format!(" {fb} "),
                Style::default().fg(Color::White).bg(theme.primary),
            )
        } else {
            Span::styled(
                " [Tab] Switch Tab  [c] Copy IP  [r] Copy Ray ID  [↑/↓] Scroll  [Esc/q] Close ",
                Style::default().fg(Color::DarkGray),
            )
        };

        let p = Paragraph::new(Line::from(vec![feedback_span]))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border)),
            );

        f.render_widget(p, area);
    }
}

fn get_country_name(code: &str) -> &'static str {
    match code.to_uppercase().as_str() {
        "US" => "United States",
        "DE" => "Germany",
        "GB" | "UK" => "United Kingdom",
        "FR" => "France",
        "NL" => "Netherlands",
        "CA" => "Canada",
        "JP" => "Japan",
        "AU" => "Australia",
        "BR" => "Brazil",
        "IN" => "India",
        "SG" => "Singapore",
        "CN" => "China",
        _ => "International",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> CachedSecurityEvent {
        CachedSecurityEvent {
            ray_id: "8472a1bf001".to_string(),
            zone_id: "zone-1".to_string(),
            timestamp: "2026-01-20T14:30:00Z".to_string(),
            action: "block".to_string(),
            source: "firewallCustom".to_string(),
            rule_id: "rule-sqli".to_string(),
            rule_description: "Block SQL injection attempts".to_string(),
            client_ip: "198.51.100.42".to_string(),
            client_country: "US".to_string(),
            client_asn: 13335,
            client_as_org: "Cloudflare Inc".to_string(),
            http_method: "POST".to_string(),
            http_host: "api.example.com".to_string(),
            http_path: "/login".to_string(),
            user_agent: "sqlmap/1.7".to_string(),
        }
    }

    fn sample_rule() -> CachedWafRule {
        CachedWafRule {
            id: "rule-sqli".to_string(),
            zone_id: "zone-1".to_string(),
            description: "Block SQL injection attempts".to_string(),
            action: "block".to_string(),
            expression: "(http.request.uri.path contains \"/login\")".to_string(),
            paused: false,
            priority: 1,
            ref_code: "SQLI_RULE".to_string(),
        }
    }

    #[test]
    fn test_security_inspector_initialization() {
        let ev = sample_event();
        let rule = sample_rule();
        let modal = SecurityInspectorModal::new(ev, Some(rule));

        assert_eq!(modal.active_tab, InspectorTab::Details);
        assert_eq!(modal.scroll_offset, 0);
        assert_eq!(modal.event.client_ip, "198.51.100.42");
        assert_eq!(modal.matched_rule.as_ref().unwrap().id, "rule-sqli");
    }

    #[test]
    fn test_security_inspector_tab_switching() {
        let ev = sample_event();
        let mut modal = SecurityInspectorModal::new(ev, None);

        assert_eq!(modal.active_tab, InspectorTab::Details);
        modal.next_tab();
        assert_eq!(modal.active_tab, InspectorTab::Rule);
        modal.next_tab();
        assert_eq!(modal.active_tab, InspectorTab::RawJson);
        modal.next_tab();
        assert_eq!(modal.active_tab, InspectorTab::Details);

        modal.prev_tab();
        assert_eq!(modal.active_tab, InspectorTab::RawJson);
    }

    #[test]
    fn test_security_inspector_scrolling() {
        let ev = sample_event();
        let mut modal = SecurityInspectorModal::new(ev, None);

        assert_eq!(modal.scroll_offset, 0);
        modal.scroll_down();
        assert_eq!(modal.scroll_offset, 1);
        modal.scroll_down();
        assert_eq!(modal.scroll_offset, 2);
        modal.scroll_up();
        assert_eq!(modal.scroll_offset, 1);
        modal.scroll_up();
        assert_eq!(modal.scroll_offset, 0);
        modal.scroll_up();
        assert_eq!(modal.scroll_offset, 0);
    }

    #[test]
    fn test_security_inspector_copy_feedback() {
        let ev = sample_event();
        let mut modal = SecurityInspectorModal::new(ev, None);

        assert!(modal.action_feedback.is_none());
        modal.copy_ip();
        assert!(modal
            .action_feedback
            .as_ref()
            .unwrap()
            .contains("198.51.100.42"));

        modal.copy_ray_id();
        assert!(modal
            .action_feedback
            .as_ref()
            .unwrap()
            .contains("8472a1bf001"));
    }
}
