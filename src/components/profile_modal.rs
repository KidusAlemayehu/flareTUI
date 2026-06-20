use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::cloudflare::auth::{AccountProfile, AuthManager, Config};
use crate::theme::Theme;

pub fn mask_token(token: &str) -> String {
    let trimmed = token.trim();
    if trimmed.len() <= 8 {
        "••••••••".to_string()
    } else {
        let prefix = &trimmed[..4];
        let suffix = &trimmed[trimmed.len() - 4..];
        format!("{prefix}••••••••{suffix}")
    }
}

pub struct ProfileModal {
    pub config: Config,
    pub profiles: Vec<AccountProfile>,
    pub selected_idx: usize,
    pub status_msg: Option<String>,
}

impl ProfileModal {
    pub fn new() -> Self {
        let config = AuthManager::load_config();
        let profiles = config.get_effective_profiles();
        let current_name = config.current_profile.as_deref().unwrap_or("default");
        let selected_idx = profiles.iter().position(|p| p.name == current_name).unwrap_or(0);

        Self {
            config,
            profiles,
            selected_idx,
            status_msg: None,
        }
    }

    pub fn reload(&mut self) {
        self.config = AuthManager::load_config();
        self.profiles = self.config.get_effective_profiles();
        if self.selected_idx >= self.profiles.len() && !self.profiles.is_empty() {
            self.selected_idx = self.profiles.len() - 1;
        }
    }

    pub fn nav_up(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn nav_down(&mut self) {
        if !self.profiles.is_empty() && self.selected_idx + 1 < self.profiles.len() {
            self.selected_idx += 1;
        }
    }

    pub fn selected_profile(&self) -> Option<&AccountProfile> {
        self.profiles.get(self.selected_idx)
    }

    pub fn switch_to_selected(&mut self) -> Option<AccountProfile> {
        if let Some(selected) = self.selected_profile().cloned() {
            let _ = AuthManager::switch_active_profile(&selected.name);
            self.status_msg = Some(format!("Switched to profile '{}'", selected.name));
            self.reload();
            Some(selected)
        } else {
            None
        }
    }

    pub fn delete_selected(&mut self) -> bool {
        if let Some(selected) = self.selected_profile().cloned() {
            if self.profiles.len() <= 1 {
                self.status_msg = Some("Cannot delete the only configured profile".to_string());
                return false;
            }
            self.config.delete_profile(&selected.name);
            let _ = AuthManager::save_config(&self.config);
            self.status_msg = Some(format!("Deleted profile '{}'", selected.name));
            self.reload();
            true
        } else {
            false
        }
    }

    pub fn logout_selected(&mut self) -> bool {
        let _ = AuthManager::logout();
        self.status_msg = Some("Logged out active session".to_string());
        self.reload();
        true
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 82.min(area.width.saturating_sub(4));
        let popup_height = 18.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let active_name = self.config.current_profile.as_deref().unwrap_or("default");

        let block = Block::default()
            .title(format!(" Account Profile Switcher (Active: {active_name}) "))
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),    // Profiles Table
                Constraint::Length(3), // Status feedback / Details
                Constraint::Length(2), // Key hints footer
            ])
            .margin(1)
            .split(popup_area);

        self.render_profiles_table(f, chunks[0], theme, active_name);
        self.render_status_pane(f, chunks[1], theme);
        self.render_footer(f, chunks[2], theme);
    }

    fn render_profiles_table(&self, f: &mut Frame, area: Rect, theme: &Theme, active_name: &str) {
        let header_cells = ["Profile Name", "Status", "Account Org", "Masked API Token"]
            .iter()
            .map(|h| Cell::from(*h).style(theme.header_style()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = self.profiles.iter().enumerate().map(|(idx, p)| {
            let is_selected = idx == self.selected_idx;
            let is_active = p.name == active_name;

            let status_cell = if is_active {
                Cell::from(" ACTIVE ")
                    .style(Style::default().fg(Color::Black).bg(theme.success).add_modifier(Modifier::BOLD))
            } else {
                Cell::from("-").style(Style::default().fg(Color::DarkGray))
            };

            let row_style = if is_selected {
                theme.selected_row()
            } else {
                Style::default()
            };

            let account_disp = if let Some(org) = &p.account_name {
                org.clone()
            } else if let Some(acc_id) = &p.account_id {
                acc_id.clone()
            } else {
                "Default Org".to_string()
            };

            Row::new(vec![
                Cell::from(p.name.clone()).style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
                status_cell,
                Cell::from(account_disp),
                Cell::from(mask_token(&p.api_token)).style(Style::default().fg(theme.secondary)),
            ])
            .style(row_style)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(20), // Profile Name
                Constraint::Length(10), // Status
                Constraint::Length(22), // Account Org
                Constraint::Min(20),    // Masked Token
            ],
        )
        .header(header);

        f.render_widget(table, area);
    }

    fn render_status_pane(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let status_span = if let Some(msg) = &self.status_msg {
            Span::styled(format!(" {msg} "), Style::default().fg(Color::White).bg(theme.primary))
        } else if let Some(p) = self.selected_profile() {
            Span::styled(
                format!("Selected: {} | Default Zone: {}", p.name, p.default_zone_id.as_deref().unwrap_or("All")),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::styled("No profiles configured.", Style::default().fg(Color::DarkGray))
        };

        let p = Paragraph::new(Line::from(vec![status_span]))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border)),
            );

        f.render_widget(p, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect, _theme: &Theme) {
        let p = Paragraph::new(
            " [Enter] Switch Profile  [d] Delete  [l] Logout  [↑/↓] Navigate  [Esc/q] Close ",
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

        f.render_widget(p, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_token() {
        assert_eq!(mask_token("short"), "••••••••");
        assert_eq!(mask_token("1234567890abcdef"), "1234••••••••cdef");
    }

    #[test]
    fn test_profile_modal_navigation() {
        let mut modal = ProfileModal {
            config: Config::default(),
            profiles: vec![
                AccountProfile {
                    name: "prof-1".to_string(),
                    api_token: "tok_1".to_string(),
                    account_id: None,
                    account_name: None,
                    default_zone_id: None,
                },
                AccountProfile {
                    name: "prof-2".to_string(),
                    api_token: "tok_2".to_string(),
                    account_id: None,
                    account_name: None,
                    default_zone_id: None,
                },
            ],
            selected_idx: 0,
            status_msg: None,
        };

        assert_eq!(modal.selected_idx, 0);
        modal.nav_down();
        assert_eq!(modal.selected_idx, 1);
        modal.nav_down();
        assert_eq!(modal.selected_idx, 1);
        modal.nav_up();
        assert_eq!(modal.selected_idx, 0);
        assert_eq!(modal.selected_profile().unwrap().name, "prof-1");
    }

    #[test]
    fn test_profile_modal_single_profile_delete_prevention() {
        let mut modal = ProfileModal {
            config: Config::default(),
            profiles: vec![AccountProfile {
                name: "single".to_string(),
                api_token: "tok_1".to_string(),
                account_id: None,
                account_name: None,
                default_zone_id: None,
            }],
            selected_idx: 0,
            status_msg: None,
        };

        assert!(!modal.delete_selected());
        assert!(modal.status_msg.unwrap().contains("Cannot delete"));
    }
}
