use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::db::{CacheDb, CachedTunnel, CachedTunnelIngressRule};
use crate::system::{CloudflaredStatus, WarpStatus};
use crate::theme::Theme;

pub struct TunnelsView {
    pub db: CacheDb,
    pub tunnels: Vec<CachedTunnel>,
    pub selected_idx: usize,
    pub selected_ingress_rules: Vec<CachedTunnelIngressRule>,
    pub warp_status: WarpStatus,
    pub cloudflared_status: CloudflaredStatus,
    pub search_query: String,
    pub search_active: bool,
    pub action_feedback: Option<String>,
}

impl TunnelsView {
    pub fn new(db: CacheDb) -> Self {
        let tunnels = db.get_tunnels("").unwrap_or_default();
        let selected_ingress_rules = if let Some(t) = tunnels.first() {
            db.get_tunnel_ingress_rules(&t.id).unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            db,
            tunnels,
            selected_idx: 0,
            selected_ingress_rules,
            warp_status: WarpStatus::default(),
            cloudflared_status: CloudflaredStatus::default(),
            search_query: String::new(),
            search_active: false,
            action_feedback: None,
        }
    }

    pub fn reload(&mut self, account_id: &str) {
        self.tunnels = self.db.get_tunnels(account_id).unwrap_or_default();
        let total = self.filtered_tunnels().len();
        if self.selected_idx >= total && total > 0 {
            self.selected_idx = total - 1;
        } else if total == 0 {
            self.selected_idx = 0;
        }
        self.reload_ingress();
    }

    pub fn reload_ingress(&mut self) {
        if let Some(t) = self.selected_tunnel().cloned() {
            self.selected_ingress_rules = self.db.get_tunnel_ingress_rules(&t.id).unwrap_or_default();
        } else {
            self.selected_ingress_rules.clear();
        }
    }

    pub fn selected_tunnel(&self) -> Option<&CachedTunnel> {
        let filtered = self.filtered_tunnels();
        filtered.get(self.selected_idx).copied()
    }

    pub fn filtered_tunnels(&self) -> Vec<&CachedTunnel> {
        let q = self.search_query.trim().to_lowercase();
        if q.is_empty() {
            self.tunnels.iter().collect()
        } else {
            self.tunnels
                .iter()
                .filter(|t| {
                    t.name.to_lowercase().contains(&q)
                        || t.status.to_lowercase().contains(&q)
                        || t.id.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn nav_up(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
            self.reload_ingress();
        }
    }

    pub fn nav_down(&mut self) {
        let total = self.filtered_tunnels().len();
        if total > 0 && self.selected_idx + 1 < total {
            self.selected_idx += 1;
            self.reload_ingress();
        }
    }

    pub fn toggle_search(&mut self) {
        self.search_active = !self.search_active;
        if !self.search_active {
            self.search_query.clear();
        }
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_active = false;
        self.selected_idx = 0;
        self.reload_ingress();
    }

    pub fn input_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.selected_idx = 0;
        self.reload_ingress();
    }

    pub fn input_search_backspace(&mut self) {
        self.search_query.pop();
        self.selected_idx = 0;
        self.reload_ingress();
    }

    pub fn set_feedback<S: Into<String>>(&mut self, msg: S) {
        self.action_feedback = Some(msg.into());
    }

    pub fn set_warp_status(&mut self, status: WarpStatus) {
        self.warp_status = status;
    }

    pub fn set_cloudflared_status(&mut self, status: CloudflaredStatus) {
        self.cloudflared_status = status;
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let search_height = if self.search_active || !self.search_query.is_empty() { 3 } else { 0 };

        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(search_height),
                Constraint::Min(5),
            ])
            .split(area);

        // 1. Optional Search Filter
        if search_height > 0 {
            let search_border_style = if self.search_active {
                Style::default().fg(theme.primary)
            } else {
                Style::default().fg(theme.border)
            };

            let search_input = Paragraph::new(Line::from(vec![
                Span::styled(" Filter Tunnels: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(&self.search_query, Style::default().fg(Color::White)),
                if self.search_active {
                    Span::styled("█", Style::default().fg(theme.primary))
                } else {
                    Span::raw("")
                },
                Span::raw("   "),
                Span::styled("[Esc: Clear / Close]", Style::default().fg(theme.text_dim)),
            ]))
            .block(
                Block::default()
                    .title(" Search Filter ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(search_border_style),
            );
            f.render_widget(search_input, outer_chunks[0]);
        }

        // 2. Main Split: Left (Tunnels Table), Right (Ingress + Local WARP/cloudflared sidecar)
        let content_area = outer_chunks[1];
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(55),
                Constraint::Percentage(45),
            ])
            .split(content_area);

        self.render_tunnels_table(f, horizontal_chunks[0], theme);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(55),
                Constraint::Percentage(45),
            ])
            .split(horizontal_chunks[1]);

        self.render_ingress_card(f, right_chunks[0], theme);
        self.render_sidecar_card(f, right_chunks[1], theme);
    }

    fn render_tunnels_table(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let filtered = self.filtered_tunnels();

        let header_cells = [
            "Status",
            "Tunnel Name",
            "Tunnel ID",
            "Replicas",
            "Config",
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

        let rows = filtered.iter().enumerate().map(|(idx, t)| {
            let is_selected = idx == self.selected_idx;

            let (status_icon, status_color) = match t.status.to_lowercase().as_str() {
                "healthy" => ("● Healthy", theme.success),
                "degraded" => ("▲ Degraded", theme.warning),
                "down" => ("✖ Down", theme.error),
                _ => ("○ Inactive", theme.text_dim),
            };

            let config_span = if t.remote_config {
                Span::styled("✔ Cloudflare", Style::default().fg(theme.info))
            } else {
                Span::styled("Local File", Style::default().fg(theme.text_dim))
            };

            let style = if is_selected {
                theme.selected_row()
            } else {
                Style::default()
            };

            let short_id = if t.id.len() > 14 {
                format!("{}...", &t.id[..14])
            } else {
                t.id.clone()
            };

            Row::new(vec![
                Cell::from(Span::styled(status_icon, Style::default().fg(status_color))),
                Cell::from(Span::styled(&t.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(short_id, Style::default().fg(theme.text_dim))),
                Cell::from(Span::styled(format!("{} conns", t.conns_count), Style::default().fg(Color::White))),
                Cell::from(config_span),
            ])
            .style(style)
        });

        let title = format!(
            " Cloudflare Zero Trust Tunnels ({}) - [w] Toggle WARP | [m] WARP Mode | [r] Sync ",
            filtered.len()
        );

        let table = Table::new(
            rows,
            [
                Constraint::Length(12),
                Constraint::Percentage(32),
                Constraint::Percentage(28),
                Constraint::Length(12),
                Constraint::Length(14),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(title)
                .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        );

        f.render_widget(table, area);
    }

    fn render_ingress_card(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let title = if let Some(t) = self.selected_tunnel() {
            format!(" Ingress Routing Rules: {} ", t.name)
        } else {
            " Ingress Routing Rules ".to_string()
        };

        let mut lines = Vec::new();

        if let Some(feedback) = &self.action_feedback {
            lines.push(Line::from(vec![
                Span::styled(" Notice: ", Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
                Span::styled(feedback, Style::default().fg(Color::White)),
            ]));
            lines.push(Line::raw(""));
        }

        if self.selected_ingress_rules.is_empty() {
            if let Some(t) = self.selected_tunnel() {
                lines.push(Line::from(vec![
                    Span::styled("  Tunnel Name: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(&t.name, Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  Tunnel ID:   ", Style::default().fg(theme.text_dim)),
                    Span::styled(&t.id, Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  Status:      ", Style::default().fg(theme.text_dim)),
                    Span::styled(&t.status, Style::default().fg(if t.status == "healthy" { theme.success } else { theme.warning })),
                ]));
                lines.push(Line::raw(""));
                lines.push(Line::from(vec![
                    Span::styled("  Default Catch-all: ", Style::default().fg(theme.text_dim)),
                    Span::styled("http_status:404", Style::default().fg(theme.info)),
                ]));
                lines.push(Line::raw(""));
                lines.push(Line::from(vec![
                    Span::styled("  Press 'r' to fetch live ingress hostname routes from Cloudflare API.", Style::default().fg(theme.text_dim)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("  No tunnels found. Press 'r' to sync with Cloudflare.", Style::default().fg(theme.text_dim)),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("  #  ", Style::default().fg(theme.text_dim)),
                Span::styled("Public Hostname & Path", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::raw(" -> "),
                Span::styled("Local Origin Service", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::raw("  -------------------------------------------------------------"));

            for (idx, rule) in self.selected_ingress_rules.iter().enumerate() {
                let host = if rule.hostname.is_empty() {
                    "*(Catch-All)".to_string()
                } else {
                    format!("{}{}", rule.hostname, rule.path)
                };

                let svc_color = if rule.service.starts_with("http://") || rule.service.starts_with("https://") {
                    theme.success
                } else if rule.service.starts_with("http_status:404") {
                    theme.text_dim
                } else {
                    theme.info
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("  {:2}. ", idx + 1), Style::default().fg(theme.text_dim)),
                    Span::styled(format!("{:<26}", host), Style::default().fg(Color::White)),
                    Span::raw(" -> "),
                    Span::styled(&rule.service, Style::default().fg(svc_color)),
                ]));
            }
        }

        let p = Paragraph::new(lines).block(
            Block::default()
                .title(title)
                .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        );

        f.render_widget(p, area);
    }

    fn render_sidecar_card(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let (warp_icon, warp_color) = if self.warp_status.connected {
            ("● Active (Protected)", theme.success)
        } else if self.warp_status.installed {
            ("○ Disconnected", theme.warning)
        } else {
            ("✖ Not Installed", theme.text_dim)
        };

        let cloudflared_text = if let Some(ver) = &self.cloudflared_status.version {
            ver.clone()
        } else if self.cloudflared_status.installed {
            "Installed (binary found)".to_string()
        } else {
            "Not Installed (install via: apt install cloudflared)".to_string()
        };

        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled(" Cloudflare WARP Client: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(warp_icon, Style::default().fg(warp_color).add_modifier(Modifier::BOLD)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("   Mode:          ", Style::default().fg(theme.text_dim)),
            Span::styled(&self.warp_status.mode, Style::default().fg(Color::White)),
            Span::raw("   "),
            Span::styled("[m: Cycle Mode]", Style::default().fg(theme.text_dim)),
        ]));

        if let Some(org) = &self.warp_status.organization {
            lines.push(Line::from(vec![
                Span::styled("   Organization:  ", Style::default().fg(theme.text_dim)),
                Span::styled(org, Style::default().fg(theme.info)),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("   Controls:      ", Style::default().fg(theme.text_dim)),
            Span::styled("[w] Connect / Disconnect WARP", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]));

        lines.push(Line::raw(""));

        lines.push(Line::from(vec![
            Span::styled(" Local cloudflared Sidecar: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(if self.cloudflared_status.installed { "● Ready" } else { "○ Missing" }, Style::default().fg(if self.cloudflared_status.installed { theme.success } else { theme.text_dim })),
        ]));

        lines.push(Line::from(vec![
            Span::styled("   Version:       ", Style::default().fg(theme.text_dim)),
            Span::styled(cloudflared_text, Style::default().fg(Color::White)),
        ]));

        if let Some(path) = &self.cloudflared_status.binary_path {
            lines.push(Line::from(vec![
                Span::styled("   Binary Path:   ", Style::default().fg(theme.text_dim)),
                Span::styled(path, Style::default().fg(theme.text_dim)),
            ]));
        }

        let p = Paragraph::new(lines).block(
            Block::default()
                .title(" Local WARP CLI & cloudflared Status ")
                .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        );

        f.render_widget(p, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnels_view_navigation_and_filtering() {
        let db = CacheDb::open_in_memory().expect("open db");
        let tunnels = vec![
            CachedTunnel {
                id: "tun-prod-1".to_string(),
                account_id: "acc-1".to_string(),
                name: "prod-cluster".to_string(),
                status: "healthy".to_string(),
                created_at: "2026-01-01".to_string(),
                conns_count: 4,
                remote_config: true,
            },
            CachedTunnel {
                id: "tun-dev-2".to_string(),
                account_id: "acc-1".to_string(),
                name: "dev-local".to_string(),
                status: "down".to_string(),
                created_at: "2026-02-01".to_string(),
                conns_count: 0,
                remote_config: false,
            },
        ];
        db.replace_tunnels("acc-1", &tunnels).expect("save tunnels");

        let mut view = TunnelsView::new(db);
        view.reload("acc-1");
        assert_eq!(view.filtered_tunnels().len(), 2);

        view.nav_down();
        assert_eq!(view.selected_idx, 1);
        view.nav_up();
        assert_eq!(view.selected_idx, 0);

        view.search_query = "prod".to_string();
        assert_eq!(view.filtered_tunnels().len(), 1);
        assert_eq!(view.filtered_tunnels()[0].name, "prod-cluster");
    }
}
