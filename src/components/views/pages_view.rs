use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::db::{CacheDb, PagesProject};
use crate::theme::Theme;

pub struct PagesView {
    pub db: CacheDb,
    pub pages: Vec<PagesProject>,
    pub selected_idx: usize,
}

impl PagesView {
    pub fn new(db: CacheDb) -> Self {
        let pages = db.get_pages().unwrap_or_default();
        Self {
            db,
            pages,
            selected_idx: 0,
        }
    }

    pub fn reload(&mut self) {
        self.pages = self.db.get_pages().unwrap_or_default();
        if self.selected_idx >= self.pages.len() && !self.pages.is_empty() {
            self.selected_idx = self.pages.len() - 1;
        }
    }

    pub fn nav_up(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn nav_down(&mut self) {
        if !self.pages.is_empty() && self.selected_idx + 1 < self.pages.len() {
            self.selected_idx += 1;
        }
    }

    pub fn selected_project(&self) -> Option<&PagesProject> {
        self.pages.get(self.selected_idx)
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(7)])
            .split(area);

        let rows: Vec<Row> = if self.pages.is_empty() {
            vec![Row::new(vec![
                Cell::from("  No Pages projects deployed"),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("-"),
            ])
            .style(Style::default().fg(theme.text_dim))]
        } else {
            self.pages
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let is_selected = i == self.selected_idx;
                    let prefix = if is_selected { "▶ " } else { "  " };
                    let style = if is_selected {
                        theme.selected_row()
                    } else {
                        Style::default().fg(theme.text)
                    };

                    let status_style = match p.latest_deployment_status.as_str() {
                        "Success" => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
                        "Building" => Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
                        _ => Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
                    };

                    Row::new(vec![
                        Cell::from(format!("{prefix}{}", p.name))
                            .style(Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                        Cell::from(p.subdomain.clone()).style(Style::default().fg(theme.info)),
                        Cell::from(p.production_branch.clone()),
                        Cell::from(p.latest_deployment_status.clone()).style(status_style),
                        Cell::from(p.updated_on.clone()).style(Style::default().fg(theme.text_dim)),
                    ])
                    .style(style)
                })
                .collect()
        };

        let table = Table::new(
            rows,
            [
                Constraint::Length(24),
                Constraint::Length(32),
                Constraint::Length(16),
                Constraint::Length(16),
                Constraint::Length(20),
            ],
        )
        .header(
            Row::new(vec!["Project Name", "Pages Domain", "Branch", "Deploy Status", "Last Updated"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(format!(" Cloudflare Pages (Total: {}) ", self.pages.len()))
                .title_style(theme.title())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_focus)),
        );
        f.render_widget(table, chunks[0]);

        if let Some(p) = self.pages.get(self.selected_idx) {
            let details = vec![
                Line::from(vec![
                    Span::styled("Live URL: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("https://{}", p.subdomain), Style::default().fg(theme.info)),
                    Span::raw("    "),
                    Span::styled("Branch: ", Style::default().fg(theme.primary)),
                    Span::styled(&p.production_branch, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Framework: ", Style::default().fg(theme.text_dim)),
                    Span::styled("Git Integration / Cloudflare Pages", Style::default().fg(Color::White)),
                ]),
            ];

            let details_widget = Paragraph::new(details).block(
                Block::default()
                    .title(format!(" Project Deployment Details: {} ", p.name))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border)),
            );
            f.render_widget(details_widget, chunks[1]);
        } else {
            let details_widget = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("No Cloudflare Pages projects found on this account.", Style::default().fg(theme.text_dim)),
                ]),
                Line::from(vec![
                    Span::styled("Deploy web apps and static sites with Cloudflare Pages.", Style::default().fg(theme.primary)),
                ]),
            ]).block(
                Block::default()
                    .title(" Project Details ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border)),
            );
            f.render_widget(details_widget, chunks[1]);
        }
    }
}
