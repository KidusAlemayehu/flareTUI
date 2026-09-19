use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::db::{CacheDb, CachedRegistrarDomain};
use crate::theme::Theme;

pub struct DomainsView {
    pub db: CacheDb,
    pub domains: Vec<CachedRegistrarDomain>,
    pub selected_idx: usize,
    pub search_query: String,
    pub search_active: bool,
    pub action_feedback: Option<String>,
}

impl DomainsView {
    pub fn new(db: CacheDb) -> Self {
        let domains = db.get_registrar_domains("").unwrap_or_default();
        Self {
            db,
            domains,
            selected_idx: 0,
            search_query: String::new(),
            search_active: false,
            action_feedback: None,
        }
    }

    pub fn reload(&mut self, account_id: &str) {
        self.domains = self
            .db
            .get_registrar_domains(account_id)
            .unwrap_or_default();
        let total = self.filtered_domains().len();
        if self.selected_idx >= total && total > 0 {
            self.selected_idx = total - 1;
        } else if total == 0 {
            self.selected_idx = 0;
        }
    }

    pub fn filtered_domains(&self) -> Vec<&CachedRegistrarDomain> {
        let q = self.search_query.trim().to_lowercase();
        if q.is_empty() {
            self.domains.iter().collect()
        } else {
            self.domains
                .iter()
                .filter(|d| {
                    d.name.to_lowercase().contains(&q) || d.status.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn selected_domain(&self) -> Option<&CachedRegistrarDomain> {
        let filtered = self.filtered_domains();
        filtered.get(self.selected_idx).copied()
    }

    pub fn nav_up(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn nav_down(&mut self) {
        let total = self.filtered_domains().len();
        if total > 0 && self.selected_idx < total - 1 {
            self.selected_idx += 1;
        }
    }

    pub fn toggle_search(&mut self) {
        self.search_active = !self.search_active;
        if !self.search_active {
            self.search_query.clear();
        }
    }

    pub fn input_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.selected_idx = 0;
    }

    pub fn input_search_backspace(&mut self) {
        self.search_query.pop();
        self.selected_idx = 0;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.selected_idx = 0;
    }

    pub fn set_feedback(&mut self, msg: String) {
        self.action_feedback = Some(msg);
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let search_height = if self.search_active || !self.search_query.is_empty() {
            3
        } else {
            0
        };

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(search_height),
                Constraint::Percentage(55),
                Constraint::Percentage(45),
            ])
            .split(area);

        // 1. Search Bar
        if search_height > 0 {
            let search_border_style = if self.search_active {
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.secondary)
            };

            let search_input = Paragraph::new(Line::from(vec![
                Span::styled(
                    " Filter Domains: ",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&self.search_query, Style::default().fg(Color::White)),
                if self.search_active {
                    Span::styled("█", Style::default().fg(theme.primary))
                } else {
                    Span::raw("")
                },
            ]))
            .block(
                Block::default()
                    .title(" Search Filter ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(search_border_style),
            );
            f.render_widget(search_input, main_chunks[0]);
        }

        // 2. Domains Table
        self.render_domains_table(f, main_chunks[1], theme);

        // 3. Domain Inspector Card
        self.render_domain_inspector(f, main_chunks[2], theme);
    }

    fn render_domains_table(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let filtered = self.filtered_domains();

        let header_cells = [
            "Status",
            "Domain Name",
            "Auto-Renew",
            "Lock",
            "Privacy",
            "Expires At",
            "Nameservers",
        ]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = filtered.iter().enumerate().map(|(idx, d)| {
            let is_selected = idx == self.selected_idx;

            let (status_icon, status_color) = match d.status.to_lowercase().as_str() {
                "active" => ("● Active", theme.success),
                "expiring_soon" => ("▲ Expiring", theme.warning),
                "expired" => ("✖ Expired", theme.error),
                _ => ("○ Unknown", theme.text_dim),
            };

            let autorenew_span = if d.auto_renew {
                Span::styled("✔ Enabled", Style::default().fg(theme.success))
            } else {
                Span::styled("✖ Disabled", Style::default().fg(theme.warning))
            };

            let lock_span = if d.locked {
                Span::styled("🔒 Locked", Style::default().fg(theme.info))
            } else {
                Span::styled("🔓 Unlocked", Style::default().fg(theme.warning))
            };

            let privacy_span = if d.privacy {
                Span::styled("🛡 Protected", Style::default().fg(theme.success))
            } else {
                Span::styled("○ Public", Style::default().fg(theme.text_dim))
            };

            let expires = if d.expires_at.len() >= 10 {
                &d.expires_at[..10]
            } else if d.expires_at.is_empty() {
                "N/A"
            } else {
                &d.expires_at
            };

            let ns_summary = if !d.name_servers.is_empty() {
                d.name_servers.join(", ")
            } else {
                "Cloudflare Nameservers".to_string()
            };

            let row_style = if is_selected {
                theme.selected_row()
            } else {
                Style::default().fg(theme.text)
            };

            Row::new(vec![
                Cell::from(Span::styled(status_icon, Style::default().fg(status_color))),
                Cell::from(d.name.clone()),
                Cell::from(autorenew_span),
                Cell::from(lock_span),
                Cell::from(privacy_span),
                Cell::from(expires),
                Cell::from(ns_summary),
            ])
            .style(row_style)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(13),
                Constraint::Length(26),
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Min(25),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(format!(
                    " Cloudflare Registrar Domains ({}) ",
                    filtered.len()
                ))
                .title_style(theme.title())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.secondary)),
        );

        f.render_widget(table, area);
    }

    fn render_domain_inspector(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        if let Some(domain) = self.selected_domain() {
            // Left Card: Domain Registration Info
            let (status_text, status_color) = match domain.status.to_lowercase().as_str() {
                "active" => ("● Active & In Good Standing", theme.success),
                "expiring_soon" => ("▲ Renewal Required Soon", theme.warning),
                "expired" => ("✖ Expired / Grace Period", theme.error),
                _ => ("○ Pending Registrar Verification", theme.text_dim),
            };

            let created = if domain.created_at.len() >= 10 {
                &domain.created_at[..10]
            } else if domain.created_at.is_empty() {
                "2024-01-01"
            } else {
                &domain.created_at
            };

            let expires = if domain.expires_at.len() >= 10 {
                &domain.expires_at[..10]
            } else if domain.expires_at.is_empty() {
                "2027-01-01"
            } else {
                &domain.expires_at
            };

            let domain_info = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(" Domain Name   : ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        &domain.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(" Registry Status: ", Style::default().fg(theme.text_dim)),
                    Span::styled(status_text, Style::default().fg(status_color)),
                ]),
                Line::from(vec![
                    Span::styled(" Registrar     : ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        "Cloudflare Registrar (At-Cost Pricing)",
                        Style::default().fg(theme.primary),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(" Registered On : ", Style::default().fg(theme.text_dim)),
                    Span::styled(created, Style::default().fg(theme.text)),
                    Span::styled("    Expires On: ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        expires,
                        Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        " Quick Actions : ",
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("[a] Auto-Renew  ", Style::default().fg(theme.secondary)),
                    Span::styled("[l] Transfer Lock  ", Style::default().fg(theme.info)),
                    Span::styled("[p] Privacy  ", Style::default().fg(theme.success)),
                    Span::styled("[n] New Domain", Style::default().fg(theme.warning)),
                ]),
            ])
            .block(
                Block::default()
                    .title(format!(" Domain Details: {} ", domain.name))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.primary)),
            );
            f.render_widget(domain_info, chunks[0]);

            // Right Card: Nameservers & Security Flags
            let ns_lines: Vec<Line> = if domain.name_servers.is_empty() {
                vec![
                    Line::from(vec![
                        Span::styled("  • ", Style::default().fg(theme.secondary)),
                        Span::styled("eva.ns.cloudflare.com", Style::default().fg(Color::White)),
                    ]),
                    Line::from(vec![
                        Span::styled("  • ", Style::default().fg(theme.secondary)),
                        Span::styled("kurt.ns.cloudflare.com", Style::default().fg(Color::White)),
                    ]),
                ]
            } else {
                domain
                    .name_servers
                    .iter()
                    .map(|ns| {
                        Line::from(vec![
                            Span::styled("  • ", Style::default().fg(theme.secondary)),
                            Span::styled(ns, Style::default().fg(Color::White)),
                        ])
                    })
                    .collect()
            };

            let mut right_lines = vec![Line::from(vec![Span::styled(
                " Assigned Nameservers:",
                Style::default().fg(theme.text_dim),
            )])];
            right_lines.extend(ns_lines);
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled(" Security Settings : ", Style::default().fg(theme.text_dim)),
                if domain.locked {
                    Span::styled("🔒 Transfer Locked  ", Style::default().fg(theme.info))
                } else {
                    Span::styled("🔓 Transfer Allowed  ", Style::default().fg(theme.warning))
                },
                if domain.privacy {
                    Span::styled("🛡 WHOIS Redacted", Style::default().fg(theme.success))
                } else {
                    Span::styled("○ WHOIS Public", Style::default().fg(theme.text_dim))
                },
            ]));

            if let Some(feedback) = &self.action_feedback {
                right_lines.push(Line::from(vec![Span::styled(
                    format!(" ▶ {feedback}"),
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                )]));
            }

            let ns_card = Paragraph::new(right_lines).block(
                Block::default()
                    .title(" Security & Nameservers ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.secondary)),
            );
            f.render_widget(ns_card, chunks[1]);
        } else {
            let empty_p = Paragraph::new(Line::from(vec![Span::styled(
                "No domain selected. Press [n] to onboard a new domain or register via Cloudflare.",
                Style::default().fg(theme.text_dim),
            )]))
            .block(
                Block::default()
                    .title(" Domain Details ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.secondary)),
            );
            f.render_widget(empty_p, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domains_view_navigation() {
        let db = CacheDb::open_in_memory().unwrap();
        let domains = vec![
            CachedRegistrarDomain {
                id: "1".into(),
                account_id: "acc".into(),
                name: "alpha.com".into(),
                status: "active".into(),
                auto_renew: true,
                expires_at: "2027-01-01".into(),
                created_at: "2024-01-01".into(),
                locked: true,
                privacy: true,
                name_servers: vec!["ns1.cf.com".into()],
            },
            CachedRegistrarDomain {
                id: "2".into(),
                account_id: "acc".into(),
                name: "beta.org".into(),
                status: "active".into(),
                auto_renew: false,
                expires_at: "2028-01-01".into(),
                created_at: "2025-01-01".into(),
                locked: false,
                privacy: true,
                name_servers: vec!["ns2.cf.com".into()],
            },
        ];
        db.replace_registrar_domains("acc", &domains).unwrap();

        let mut view = DomainsView::new(db);
        view.reload("acc");
        assert_eq!(view.domains.len(), 2);
        assert_eq!(view.selected_idx, 0);
        assert_eq!(view.selected_domain().unwrap().name, "alpha.com");

        view.nav_down();
        assert_eq!(view.selected_idx, 1);
        assert_eq!(view.selected_domain().unwrap().name, "beta.org");

        view.nav_down();
        assert_eq!(view.selected_idx, 1); // at bottom

        view.nav_up();
        assert_eq!(view.selected_idx, 0);
    }

    #[test]
    fn test_domains_view_filtering() {
        let db = CacheDb::open_in_memory().unwrap();
        let domains = vec![
            CachedRegistrarDomain {
                id: "1".into(),
                account_id: "acc".into(),
                name: "daedalus.com".into(),
                status: "active".into(),
                auto_renew: true,
                expires_at: "2027-01-01".into(),
                created_at: "2024-01-01".into(),
                locked: true,
                privacy: true,
                name_servers: vec![],
            },
            CachedRegistrarDomain {
                id: "2".into(),
                account_id: "acc".into(),
                name: "flare.org".into(),
                status: "active".into(),
                auto_renew: true,
                expires_at: "2028-01-01".into(),
                created_at: "2025-01-01".into(),
                locked: true,
                privacy: true,
                name_servers: vec![],
            },
        ];
        db.replace_registrar_domains("acc", &domains).unwrap();

        let mut view = DomainsView::new(db);
        view.reload("acc");
        assert_eq!(view.filtered_domains().len(), 2);

        view.input_search_char('f');
        view.input_search_char('l');
        view.input_search_char('a');
        view.input_search_char('r');
        view.input_search_char('e');
        assert_eq!(view.filtered_domains().len(), 1);
        assert_eq!(view.filtered_domains()[0].name, "flare.org");

        view.clear_search();
        assert_eq!(view.filtered_domains().len(), 2);
    }

    #[test]
    fn test_domains_view_rendering() {
        let db = CacheDb::open_in_memory().unwrap();
        let domains = vec![CachedRegistrarDomain {
            id: "1".into(),
            account_id: "acc".into(),
            name: "daedalus-dev.com".into(),
            status: "active".into(),
            auto_renew: true,
            expires_at: "2027-08-15".into(),
            created_at: "2024-08-15".into(),
            locked: true,
            privacy: true,
            name_servers: vec![
                "eva.ns.cloudflare.com".into(),
                "kurt.ns.cloudflare.com".into(),
            ],
        }];
        db.replace_registrar_domains("acc", &domains).unwrap();

        let mut view = DomainsView::new(db);
        view.reload("acc");

        let backend = ratatui::backend::TestBackend::new(120, 40);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal.draw(|f| view.render(f, f.area(), &theme)).unwrap();
    }
}
