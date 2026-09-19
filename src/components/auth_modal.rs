use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::cloudflare::auth::AuthManager;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    OAuth,
    Token,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthStatus {
    Idle,
    Waiting {
        port: u16,
        auth_url: String,
        state: String,
    },
    Success(String),
    Error(String),
}

pub struct AuthModal {
    pub mode: AuthMode,
    pub input_token: String,
    pub is_validating: bool,
    pub error_msg: Option<String>,
    pub oauth_status: OAuthStatus,
    pub action_feedback: Option<String>,
}

impl Default for AuthModal {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthModal {
    pub fn new() -> Self {
        Self {
            mode: AuthMode::OAuth,
            input_token: String::new(),
            is_validating: false,
            error_msg: None,
            oauth_status: OAuthStatus::Idle,
            action_feedback: None,
        }
    }

    pub fn switch_mode(&mut self, mode: AuthMode) {
        self.mode = mode;
        self.error_msg = None;
        self.action_feedback = None;
    }

    pub fn cycle_mode(&mut self) {
        match self.mode {
            AuthMode::OAuth => self.switch_mode(AuthMode::Token),
            AuthMode::Token => self.switch_mode(AuthMode::OAuth),
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if !c.is_control() {
            self.input_token.push(c);
            self.error_msg = None;
            self.action_feedback = None;
        }
    }

    pub fn handle_backspace(&mut self) {
        self.input_token.pop();
        self.error_msg = None;
        self.action_feedback = None;
    }

    pub fn clear(&mut self) {
        self.input_token.clear();
        self.is_validating = false;
        self.error_msg = None;
        self.action_feedback = None;
    }

    pub fn copy_oauth_url(&mut self) {
        if let OAuthStatus::Waiting { auth_url, .. } = &self.oauth_status {
            self.action_feedback = Some(format!(
                "Copied authorization URL to clipboard buffer ({})",
                &auth_url[..auth_url.len().min(40)]
            ));
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = crate::components::centered_rect(area, 75, 60, 100, 70, 16, 26);

        f.render_widget(Clear, popup_area);

        let config = AuthManager::load_config();
        let active_profile = config.active_profile();
        let is_authenticated = config.active_token().is_some();

        if is_authenticated {
            let block = Block::default()
                .title(" Cloudflare Account Profile ")
                .title_style(theme.title())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.primary));

            f.render_widget(block, popup_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Profile Header Banner
                    Constraint::Length(1), // Divider
                    Constraint::Min(8),    // Profile Details & Logout Option
                    Constraint::Length(2), // Footer Keybindings
                ])
                .margin(1)
                .split(popup_area);

            self.render_authenticated_header(f, chunks[0], theme, &config, active_profile.as_ref());

            let div_text = Paragraph::new("─".repeat(chunks[1].width as usize))
                .style(Style::default().fg(theme.border));
            f.render_widget(div_text, chunks[1]);

            self.render_authenticated_content(
                f,
                chunks[2],
                theme,
                &config,
                active_profile.as_ref(),
            );

            self.render_authenticated_footer(f, chunks[3], theme);
            return;
        }

        let block = Block::default()
            .title(" Cloudflare Account Authentication ")
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Mode Tabs & Account Status
                Constraint::Length(1), // Divider
                Constraint::Min(8),    // Body Content
                Constraint::Length(2), // Footer Keybindings
            ])
            .margin(1)
            .split(popup_area);

        // 1. Top Section: Mode tabs + Session indicator
        self.render_header_tabs(
            f,
            chunks[0],
            theme,
            is_authenticated,
            active_profile.as_ref(),
        );

        // 2. Horizontal divider
        let div_text = Paragraph::new("─".repeat(chunks[1].width as usize))
            .style(Style::default().fg(theme.border));
        f.render_widget(div_text, chunks[1]);

        // 3. Body Content
        match self.mode {
            AuthMode::OAuth => self.render_oauth_content(f, chunks[2], theme),
            AuthMode::Token => self.render_token_content(f, chunks[2], theme),
        }

        // 4. Footer Keybindings
        self.render_footer(f, chunks[3], theme, is_authenticated);
    }

    fn render_header_tabs(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        is_authenticated: bool,
        active_profile: Option<&crate::cloudflare::auth::AccountProfile>,
    ) {
        let sub = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(30)])
            .split(area);

        // Tab buttons: [1] Browser Login (OAuth)   [2] API Token (Manual)
        let tab1_style = if self.mode == AuthMode::OAuth {
            Style::default()
                .fg(Color::Black)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_dim)
        };

        let tab2_style = if self.mode == AuthMode::Token {
            Style::default()
                .fg(Color::Black)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_dim)
        };

        let tabs = Paragraph::new(Line::from(vec![
            Span::styled(" [1] Browser Login (OAuth) ", tab1_style),
            Span::raw("  "),
            Span::styled(" [2] API Token (Manual) ", tab2_style),
        ]));
        f.render_widget(tabs, sub[0]);

        // Session indicator
        let session_line = if is_authenticated {
            let name = active_profile.map(|p| p.name.as_str()).unwrap_or("default");
            Line::from(vec![
                Span::styled("● ", Style::default().fg(theme.success)),
                Span::styled(
                    format!("Session: {name}"),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("○ ", Style::default().fg(theme.warning)),
                Span::styled("Unauthenticated", Style::default().fg(theme.text_dim)),
            ])
        };

        let session_widget = Paragraph::new(session_line).alignment(Alignment::Right);
        f.render_widget(session_widget, sub[1]);
    }

    fn render_oauth_content(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Info banner
                Constraint::Length(4), // Action / URL box
                Constraint::Length(3), // Fallback input or status
            ])
            .split(area);

        match &self.oauth_status {
            OAuthStatus::Idle => {
                let intro = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("Browser-based OAuth sign-in (redirect flow):", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("flareTUI spawns a local loopback callback server on 127.0.0.1 to securely capture your token.", Style::default().fg(theme.text_dim)),
                    ]),
                ]);
                f.render_widget(intro, content_chunks[0]);

                let action_box = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "  ▶ Press [Enter] or [o] to Launch Browser & Authorize  ",
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    )]),
                ])
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(theme.secondary)),
                );
                f.render_widget(action_box, content_chunks[1]);

                let hint = if let Some(err) = &self.error_msg {
                    Line::from(vec![
                        Span::styled(
                            "Error: ",
                            Style::default()
                                .fg(theme.error)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(err, Style::default().fg(theme.error)),
                    ])
                } else {
                    Line::from(vec![Span::styled(
                        "No browser? Press [Tab] to switch to manual API Token input.",
                        Style::default().fg(theme.text_dim),
                    )])
                };
                f.render_widget(Paragraph::new(hint), content_chunks[2]);
            }
            OAuthStatus::Waiting { port, auth_url, .. } => {
                let status_header = Paragraph::new(vec![Line::from(vec![
                    Span::styled(
                        "Waiting for browser authorization...",
                        Style::default()
                            .fg(theme.warning)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" (Listening on http://127.0.0.1:{port}/callback)"),
                        Style::default().fg(theme.text_dim),
                    ),
                ])]);
                f.render_widget(status_header, content_chunks[0]);

                let max_url_len = (content_chunks[1].width as usize)
                    .saturating_sub(12)
                    .max(20);
                let display_url = if auth_url.len() > max_url_len {
                    format!("{}...", &auth_url[..max_url_len.saturating_sub(3)])
                } else {
                    auth_url.clone()
                };

                let url_box = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled(
                            " URL: ",
                            Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            display_url,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                    ]),
                    Line::from(vec![Span::styled(
                        " [c] Copy URL to clipboard buffer",
                        Style::default().fg(theme.text_dim),
                    )]),
                ])
                .block(
                    Block::default()
                        .title(" Authorization Link ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(theme.info)),
                );
                f.render_widget(url_box, content_chunks[1]);

                let masked_input = if self.input_token.is_empty() {
                    Span::styled(
                        "Or paste callback token/code here...",
                        Style::default().fg(theme.text_dim),
                    )
                } else {
                    Span::styled(&self.input_token, Style::default().fg(Color::White))
                };

                let fallback_box = Paragraph::new(Line::from(vec![
                    Span::styled(" Token Fallback: ", Style::default().fg(theme.text_dim)),
                    masked_input,
                    Span::styled("█", Style::default().fg(theme.primary)),
                ]));
                f.render_widget(fallback_box, content_chunks[2]);
            }
            OAuthStatus::Success(msg) => {
                let success = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            "✓ ",
                            Style::default()
                                .fg(theme.success)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            msg,
                            Style::default()
                                .fg(theme.success)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                ])
                .alignment(Alignment::Center);
                f.render_widget(success, content_chunks[1]);
            }
            OAuthStatus::Error(err) => {
                let err_widget = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled(
                            "Authorization Failed: ",
                            Style::default()
                                .fg(theme.error)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(err, Style::default().fg(theme.error)),
                    ]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Press [Enter] or [o] to retry, or [Tab] for manual token.",
                        Style::default().fg(theme.text_dim),
                    )]),
                ]);
                f.render_widget(err_widget, content_chunks[1]);
            }
        }
    }

    fn render_token_content(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Instructions
                Constraint::Length(3), // Input box
                Constraint::Length(3), // Status / error
            ])
            .split(area);

        let instructions = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "Create an API Token with Zone & Worker read/edit permissions at:",
                Style::default().fg(theme.text_dim),
            )]),
            Line::from(vec![Span::styled(
                "https://dash.cloudflare.com/profile/api-tokens",
                Style::default()
                    .fg(theme.info)
                    .add_modifier(Modifier::UNDERLINED),
            )]),
        ]);
        f.render_widget(instructions, chunks[0]);

        let masked_input = if self.input_token.is_empty() {
            Span::styled(
                "Paste or type your Cloudflare API Token here...",
                Style::default().fg(theme.text_dim),
            )
        } else {
            let len = self.input_token.len();
            if len <= 8 {
                Span::styled("•".repeat(len), Style::default().fg(Color::White))
            } else {
                let masked = format!("{}...{}", "•".repeat(12), &self.input_token[len - 4..]);
                Span::styled(masked, Style::default().fg(Color::White))
            }
        };

        let input_box = Paragraph::new(Line::from(vec![
            Span::styled(
                " Token: ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
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

        let status_line = if let Some(err) = &self.error_msg {
            Line::from(vec![
                Span::styled(
                    "Error: ",
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(err, Style::default().fg(theme.error)),
            ])
        } else if self.is_validating {
            Line::from(vec![Span::styled(
                "Validating token against Cloudflare API v4...",
                Style::default().fg(theme.warning),
            )])
        } else {
            Line::from(vec![Span::styled(
                "[Enter] Connect & Sync   [Esc] Cancel",
                Style::default().fg(theme.text_dim),
            )])
        };

        let footer = Paragraph::new(status_line);
        f.render_widget(footer, chunks[2]);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect, theme: &Theme, is_authenticated: bool) {
        let mut spans = vec![Span::styled(
            "[Tab] Switch Mode  ",
            Style::default().fg(theme.text_dim),
        )];

        if is_authenticated {
            spans.push(Span::styled(
                "[Ctrl+L] Log Out Active Profile  ",
                Style::default().fg(theme.warning),
            ));
        }

        if let Some(fb) = &self.action_feedback {
            spans.push(Span::styled(fb, Style::default().fg(theme.success)));
        } else {
            spans.push(Span::styled(
                "[Esc] Close",
                Style::default().fg(theme.text_dim),
            ));
        }

        let p = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        f.render_widget(p, area);
    }

    fn render_authenticated_header(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        config: &crate::cloudflare::auth::Config,
        active_profile: Option<&crate::cloudflare::auth::AccountProfile>,
    ) {
        let sub = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(30)])
            .split(area);

        let email = active_profile
            .and_then(|p| p.account_name.as_deref())
            .or(config.account_name.as_deref())
            .unwrap_or("kidusalemayehu705@gmail.com");

        let header_left = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "● ",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Active Account: ",
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    email,
                    Style::default()
                        .fg(theme.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(
                "Connected to Cloudflare Edge API v4",
                Style::default().fg(theme.text_dim),
            )]),
        ]);
        f.render_widget(header_left, sub[0]);

        let prof_name = active_profile.map(|p| p.name.as_str()).unwrap_or("default");
        let badge_style = Style::default()
            .fg(Color::Black)
            .bg(theme.primary)
            .add_modifier(Modifier::BOLD);
        let header_right = Paragraph::new(Line::from(vec![Span::styled(
            format!(" Profile: {prof_name} "),
            badge_style,
        )]))
        .alignment(Alignment::Right);
        f.render_widget(header_right, sub[1]);
    }

    fn render_authenticated_content(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        config: &crate::cloudflare::auth::Config,
        active_profile: Option<&crate::cloudflare::auth::AccountProfile>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Profile info card
                Constraint::Min(5),    // Logout option box
            ])
            .split(area);

        let email = active_profile
            .and_then(|p| p.account_name.as_deref())
            .or(config.account_name.as_deref())
            .unwrap_or("kidusalemayehu705@gmail.com");

        let prof_name = active_profile.map(|p| p.name.as_str()).unwrap_or("default");
        let account_id = active_profile
            .and_then(|p| p.account_id.as_deref())
            .or(config.account_id.as_deref())
            .unwrap_or("e50e525d4a021564b2a920f05515e6a6");

        let raw_token = config.active_token().unwrap_or_default();
        let masked_token = if raw_token.len() > 16 {
            format!("{}...{}", "•".repeat(12), &raw_token[raw_token.len() - 6..])
        } else if !raw_token.is_empty() {
            "•".repeat(raw_token.len())
        } else {
            "None".to_string()
        };

        let profile_lines = if chunks[0].width >= 78 {
            vec![
                Line::from(vec![
                    Span::styled("  Account Email : ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        email,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Profile Name  : ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        prof_name,
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("    Account ID : ", Style::default().fg(theme.text_dim)),
                    Span::styled(account_id, Style::default().fg(theme.info)),
                ]),
                Line::from(vec![
                    Span::styled("  Session Status: ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        "● Active & Authenticated",
                        Style::default().fg(theme.success),
                    ),
                    Span::styled("    Token      : ", Style::default().fg(theme.text_dim)),
                    Span::styled(masked_token, Style::default().fg(theme.text_dim)),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::styled("  Email : ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        email,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Profile: ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        prof_name,
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  Status: ", Style::default().fg(theme.text_dim)),
                    Span::styled("● Active", Style::default().fg(theme.success)),
                ]),
                Line::from(vec![
                    Span::styled("  Acc ID: ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        if account_id.len() > 18 {
                            format!("{}...", &account_id[..15])
                        } else {
                            account_id.to_string()
                        },
                        Style::default().fg(theme.info),
                    ),
                    Span::styled("  Token: ", Style::default().fg(theme.text_dim)),
                    Span::styled(masked_token, Style::default().fg(theme.text_dim)),
                ]),
            ]
        };

        let profile_card = Paragraph::new(profile_lines)
            .block(
                Block::default()
                    .title(" Profile Information ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.secondary)),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(profile_card, chunks[0]);

        let logout_box = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  ▶ Press [Enter] or [l] to Log Out of This Account  ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Safely terminates active Cloudflare session and switches to offline cached mode",
                Style::default().fg(theme.text_dim),
            )]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Session Action ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.warning)),
        );
        f.render_widget(logout_box, chunks[1]);
    }

    fn render_authenticated_footer(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let p = Paragraph::new(Line::from(vec![
            Span::styled(
                "[Enter] / [l] Log Out  ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[p] Switch Profile  ", Style::default().fg(theme.primary)),
            Span::styled("[Esc] Close", Style::default().fg(theme.text_dim)),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(p, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_modal_mode_switching() {
        let mut modal = AuthModal::new();
        assert_eq!(modal.mode, AuthMode::OAuth);

        modal.cycle_mode();
        assert_eq!(modal.mode, AuthMode::Token);

        modal.cycle_mode();
        assert_eq!(modal.mode, AuthMode::OAuth);

        modal.switch_mode(AuthMode::Token);
        assert_eq!(modal.mode, AuthMode::Token);
    }

    #[test]
    fn test_auth_modal_input_and_backspace() {
        let mut modal = AuthModal::new();
        modal.handle_char('t');
        modal.handle_char('e');
        modal.handle_char('s');
        modal.handle_char('t');
        assert_eq!(modal.input_token, "test");

        modal.handle_backspace();
        assert_eq!(modal.input_token, "tes");

        modal.clear();
        assert!(modal.input_token.is_empty());
    }

    #[test]
    fn test_auth_modal_copy_oauth_url() {
        let mut modal = AuthModal::new();
        modal.oauth_status = OAuthStatus::Waiting {
            port: 8976,
            auth_url: "https://dash.cloudflare.com/oauth2/auth?client_id=123".to_string(),
            state: "test_state".to_string(),
        };

        modal.copy_oauth_url();
        assert!(modal.action_feedback.is_some());
        assert!(modal
            .action_feedback
            .unwrap()
            .contains("Copied authorization URL"));
    }

    #[test]
    fn test_auth_modal_rendering_unauthenticated_and_authenticated() {
        let modal = AuthModal::new();
        let backend = ratatui::backend::TestBackend::new(100, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let theme = Theme::default();

        // 1. Full modal render
        terminal
            .draw(|f| modal.render(f, f.area(), &theme))
            .unwrap();

        // 2. Direct render of authenticated sections with mock profile
        let mut auth_config = crate::cloudflare::auth::Config::default();
        let profile = crate::cloudflare::auth::AccountProfile {
            name: "production".to_string(),
            api_token: "cfut_secure_token_123456789".to_string(),
            account_id: Some("acc_abcdef123456".to_string()),
            account_name: Some("kidusalemayehu705@gmail.com".to_string()),
            default_zone_id: None,
        };
        auth_config.profiles.push(profile);
        auth_config.current_profile = Some("production".to_string());
        let active = auth_config.active_profile();

        terminal
            .draw(|f| {
                let area = f.area();
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(3),
                        ratatui::layout::Constraint::Length(1),
                        ratatui::layout::Constraint::Min(8),
                        ratatui::layout::Constraint::Length(2),
                    ])
                    .split(area);

                modal.render_authenticated_header(
                    f,
                    chunks[0],
                    &theme,
                    &auth_config,
                    active.as_ref(),
                );
                modal.render_authenticated_content(
                    f,
                    chunks[2],
                    &theme,
                    &auth_config,
                    active.as_ref(),
                );
                modal.render_authenticated_footer(f, chunks[3], &theme);
            })
            .unwrap();
    }
}
