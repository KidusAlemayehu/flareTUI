use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::cloudflare::client::PagesDeploymentResult;
use crate::theme::Theme;

pub struct PagesDeploymentsModal {
    pub project_name: String,
    pub deployments: Vec<PagesDeploymentResult>,
    pub selected_idx: usize,
    pub is_loading: bool,
    pub action_feedback: Option<String>,
}

impl PagesDeploymentsModal {
    pub fn new(project_name: String) -> Self {
        Self {
            project_name,
            deployments: Vec::new(),
            selected_idx: 0,
            is_loading: false,
            action_feedback: None,
        }
    }

    pub fn nav_up(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn nav_down(&mut self) {
        if !self.deployments.is_empty() && self.selected_idx + 1 < self.deployments.len() {
            self.selected_idx += 1;
        }
    }

    pub fn selected_deployment(&self) -> Option<&PagesDeploymentResult> {
        self.deployments.get(self.selected_idx)
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = crate::components::centered_rect(area, 85, 70, 125, 80, 18, 36);

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(format!(" 🚀 Pages Deployments: {} ", self.project_name))
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),    // Deployments table
                Constraint::Length(5), // Selected deployment detail
                Constraint::Length(3), // Footer hints & feedback
            ])
            .margin(1)
            .split(popup_area);

        // 1. Deployments Table
        let rows: Vec<Row> = if self.deployments.is_empty() {
            if self.is_loading {
                vec![Row::new(vec![
                    Cell::from("  ⏳ Loading deployments from Cloudflare...")
                        .style(Style::default().fg(theme.warning)),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])]
            } else {
                vec![Row::new(vec![
                    Cell::from("  No deployments recorded for this Pages project")
                        .style(Style::default().fg(theme.text_dim)),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])]
            }
        } else {
            self.deployments
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let is_sel = i == self.selected_idx;
                    let prefix = if is_sel { "▶ " } else { "  " };

                    let env_str = d.environment.as_deref().unwrap_or("preview");
                    let env_cell = if env_str.eq_ignore_ascii_case("production") {
                        Cell::from("[PROD]").style(
                            Style::default()
                                .fg(theme.success)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Cell::from("[PREV]").style(Style::default().fg(theme.info))
                    };

                    let short_id = d.short_id.clone().unwrap_or_else(|| {
                        if d.id.len() >= 8 {
                            d.id[..8].to_string()
                        } else {
                            d.id.clone()
                        }
                    });

                    let status_str = d
                        .latest_stage
                        .as_ref()
                        .map(|s| s.status.as_str())
                        .unwrap_or("active");

                    let (status_icon, status_style) = match status_str.to_lowercase().as_str() {
                        "success" => (
                            "● Success",
                            Style::default()
                                .fg(theme.success)
                                .add_modifier(Modifier::BOLD),
                        ),
                        "failure" | "failed" => (
                            "✖ Failed",
                            Style::default()
                                .fg(theme.error)
                                .add_modifier(Modifier::BOLD),
                        ),
                        "building" | "active" => (
                            "▲ Building",
                            Style::default()
                                .fg(theme.warning)
                                .add_modifier(Modifier::BOLD),
                        ),
                        _ => ("⚪ Idle", Style::default().fg(theme.text_dim)),
                    };

                    let branch = d
                        .deployment_trigger
                        .as_ref()
                        .and_then(|t| t.metadata.as_ref())
                        .and_then(|m| m.branch.as_deref())
                        .unwrap_or("main");

                    let msg = d
                        .deployment_trigger
                        .as_ref()
                        .and_then(|t| t.metadata.as_ref())
                        .and_then(|m| m.commit_message.as_deref())
                        .unwrap_or("Direct upload / automated build");
                    let clean_msg = msg.lines().next().unwrap_or(msg);

                    let created = d.created_on.as_deref().unwrap_or("-");
                    let display_date = if created.len() >= 19 {
                        &created[..19]
                    } else {
                        created
                    };

                    let row_style = if is_sel {
                        theme.selected_row()
                    } else {
                        Style::default().fg(theme.text)
                    };

                    Row::new(vec![
                        Cell::from(format!("{prefix}{short_id}")).style(
                            Style::default()
                                .fg(theme.secondary)
                                .add_modifier(Modifier::BOLD),
                        ),
                        env_cell,
                        Cell::from(status_icon).style(status_style),
                        Cell::from(branch).style(Style::default().fg(Color::White)),
                        Cell::from(clean_msg).style(Style::default().fg(theme.text)),
                        Cell::from(display_date).style(Style::default().fg(theme.text_dim)),
                    ])
                    .style(row_style)
                })
                .collect()
        };

        let table = Table::new(
            rows,
            [
                Constraint::Length(12), // ID
                Constraint::Length(8),  // Env
                Constraint::Length(12), // Status
                Constraint::Length(14), // Branch
                Constraint::Min(24),    // Commit Message
                Constraint::Length(20), // Created
            ],
        )
        .header(
            Row::new(vec![
                "Deploy ID",
                "Env",
                "Status",
                "Branch",
                "Commit / Trigger",
                "Deployed At",
            ])
            .style(theme.header_style())
            .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
        f.render_widget(table, chunks[0]);

        // 2. Selected Deployment Detail Card
        if let Some(sel) = self.selected_deployment() {
            let full_url = sel.url.as_deref().unwrap_or("N/A");
            let commit_hash = sel
                .deployment_trigger
                .as_ref()
                .and_then(|t| t.metadata.as_ref())
                .and_then(|m| m.commit_hash.as_deref())
                .unwrap_or("N/A");

            let detail_lines = vec![
                Line::from(vec![
                    Span::styled(
                        "🔗 URL: ",
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(full_url, Style::default().fg(theme.info)),
                    Span::raw("    "),
                    Span::styled("Commit: ", Style::default().fg(theme.text_dim)),
                    Span::styled(commit_hash, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Deployment ID: ", Style::default().fg(theme.text_dim)),
                    Span::styled(&sel.id, Style::default().fg(theme.secondary)),
                    Span::raw("    "),
                    Span::styled("Modified: ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        sel.modified_on.as_deref().unwrap_or("Never"),
                        Style::default().fg(theme.text_dim),
                    ),
                ]),
            ];

            let detail_widget = Paragraph::new(detail_lines)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title(" Selected Deployment Details ")
                        .title_style(theme.title())
                        .borders(Borders::BOTTOM)
                        .border_style(Style::default().fg(theme.border)),
                );
            f.render_widget(detail_widget, chunks[1]);
        } else {
            let empty_widget = Paragraph::new("Select a deployment to view commit and URL details")
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title(" Selected Deployment Details ")
                        .borders(Borders::BOTTOM)
                        .border_style(Style::default().fg(theme.border)),
                );
            f.render_widget(empty_widget, chunks[1]);
        }

        // 3. Operational Footer & Feedback
        let feedback_line = if let Some(fb) = &self.action_feedback {
            Line::from(vec![
                Span::styled("● ", Style::default().fg(theme.success)),
                Span::styled(
                    fb,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else if self.is_loading {
            Line::from(vec![Span::styled(
                "⏳ Communicating with Cloudflare Pages API...",
                Style::default().fg(theme.warning),
            )])
        } else {
            Line::from(vec![
                Span::styled(
                    " [r] ",
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Rollback Production to this Build   ",
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    " [l] ",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("View Build Logs   ", Style::default().fg(Color::White)),
                Span::styled(" [j/k] ", Style::default().fg(theme.secondary)),
                Span::styled("Select   ", Style::default().fg(theme.text_dim)),
                Span::styled(" [Esc] ", Style::default().fg(theme.text_dim)),
                Span::styled("Close Drawer", Style::default().fg(theme.text_dim)),
            ])
        };

        let footer = Paragraph::new(feedback_line).wrap(Wrap { trim: true });
        f.render_widget(footer, chunks[2]);
    }
}
