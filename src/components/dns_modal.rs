use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::cloudflare::client::NewDnsRecord;
use crate::theme::Theme;

#[derive(PartialEq, Eq)]
pub enum DnsModalField {
    Type,
    Name,
    Content,
    Ttl,
    Proxied,
}

pub struct DnsModal {
    pub is_edit: bool,
    pub record_id: Option<String>,
    pub zone_id: String,
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
    pub active_field: DnsModalField,
    pub error_msg: Option<String>,
}

impl DnsModal {
    pub fn new_create(zone_id: String) -> Self {
        Self {
            is_edit: false,
            record_id: None,
            zone_id,
            record_type: "A".to_string(),
            name: String::new(),
            content: String::new(),
            ttl: 1, // Auto
            proxied: true,
            active_field: DnsModalField::Name,
            error_msg: None,
        }
    }

    pub fn new_edit(
        zone_id: String,
        record_id: String,
        record_type: String,
        name: String,
        content: String,
        ttl: u32,
        proxied: bool,
    ) -> Self {
        Self {
            is_edit: true,
            record_id: Some(record_id),
            zone_id,
            record_type,
            name,
            content,
            ttl,
            proxied,
            active_field: DnsModalField::Content,
            error_msg: None,
        }
    }

    pub fn next_field(&mut self) {
        self.active_field = match self.active_field {
            DnsModalField::Type => DnsModalField::Name,
            DnsModalField::Name => DnsModalField::Content,
            DnsModalField::Content => DnsModalField::Ttl,
            DnsModalField::Ttl => DnsModalField::Proxied,
            DnsModalField::Proxied => DnsModalField::Type,
        };
    }

    pub fn prev_field(&mut self) {
        self.active_field = match self.active_field {
            DnsModalField::Type => DnsModalField::Proxied,
            DnsModalField::Name => DnsModalField::Type,
            DnsModalField::Content => DnsModalField::Name,
            DnsModalField::Ttl => DnsModalField::Content,
            DnsModalField::Proxied => DnsModalField::Ttl,
        };
    }

    pub fn cycle_type(&mut self, next: bool) {
        let types = ["A", "AAAA", "CNAME", "TXT", "MX"];
        if let Some(pos) = types.iter().position(|&t| t == self.record_type) {
            let next_pos = if next {
                (pos + 1) % types.len()
            } else if pos == 0 {
                types.len() - 1
            } else {
                pos - 1
            };
            self.record_type = types[next_pos].to_string();
        }
    }

    pub fn handle_char(&mut self, c: char) {
        match self.active_field {
            DnsModalField::Name => self.name.push(c),
            DnsModalField::Content => self.content.push(c),
            DnsModalField::Proxied => {
                if c == ' ' {
                    self.proxied = !self.proxied;
                }
            }
            DnsModalField::Type => {
                if c == ' ' {
                    self.cycle_type(true);
                }
            }
            DnsModalField::Ttl => {
                if c == ' ' {
                    self.ttl = match self.ttl {
                        1 => 60,
                        60 => 300,
                        300 => 1800,
                        1800 => 3600,
                        3600 => 86400,
                        _ => 1,
                    };
                }
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.active_field {
            DnsModalField::Name => {
                self.name.pop();
            }
            DnsModalField::Content => {
                self.content.pop();
            }
            _ => {}
        }
    }

    pub fn to_new_record(&self) -> NewDnsRecord {
        NewDnsRecord {
            record_type: self.record_type.clone(),
            name: self.name.clone(),
            content: self.content.clone(),
            ttl: self.ttl,
            proxied: self.proxied,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = crate::components::centered_rect(area, 72, 54, 96, 65, 14, 22);
        let popup_width = popup_area.width;

        f.render_widget(Clear, popup_area);

        let title = if self.is_edit {
            " ✏️ Edit DNS Record "
        } else {
            " ➕ Add New DNS Record "
        };

        let block = Block::default()
            .title(title)
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Type & Proxy
                Constraint::Length(3), // Name
                Constraint::Length(3), // Content
                Constraint::Length(3), // TTL & Proxy info
                Constraint::Length(2), // Hints & Error
            ])
            .margin(1)
            .split(popup_area);

        // 1. Type & Proxy row
        let is_type_foc = self.active_field == DnsModalField::Type;
        let is_prox_foc = self.active_field == DnsModalField::Proxied;

        let type_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        let type_text = if type_chunks[0].width >= 34 {
            vec![
                Span::styled("Type: [◄ ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &self.record_type,
                    Style::default()
                        .fg(theme.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ►] (Space to cycle)", Style::default().fg(theme.text_dim)),
            ]
        } else {
            vec![
                Span::styled("Type: ◄ ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    &self.record_type,
                    Style::default()
                        .fg(theme.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ►", Style::default().fg(theme.text_dim)),
            ]
        };

        let type_para = Paragraph::new(Line::from(type_text)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.block_border(is_type_foc)),
        );
        f.render_widget(type_para, type_chunks[0]);

        let proxy_label = if self.proxied {
            if type_chunks[1].width >= 32 {
                Span::styled(
                    "🟠 Proxied (Orange Cloud)",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    "🟠 Proxied",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                )
            }
        } else if type_chunks[1].width >= 32 {
            Span::styled(
                "⚪ DNS Only (Grey Cloud)",
                Style::default().fg(theme.text_dim),
            )
        } else {
            Span::styled("⚪ DNS Only", Style::default().fg(theme.text_dim))
        };

        let proxy_para = Paragraph::new(Line::from(vec![
            Span::raw(" "),
            proxy_label,
            Span::styled(" (Space)", Style::default().fg(theme.text_dim)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.block_border(is_prox_foc)),
        );
        f.render_widget(proxy_para, type_chunks[1]);

        // 2. Name field
        let is_name_foc = self.active_field == DnsModalField::Name;
        let name_para = Paragraph::new(Line::from(vec![
            Span::styled("Name: ", Style::default().fg(theme.text_dim)),
            Span::styled(&self.name, Style::default().fg(Color::White)),
            if is_name_foc {
                Span::styled("█", Style::default().fg(theme.primary))
            } else {
                Span::raw("")
            },
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.block_border(is_name_foc)),
        );
        f.render_widget(name_para, chunks[1]);

        // 3. Content field
        let is_content_foc = self.active_field == DnsModalField::Content;
        let content_para = Paragraph::new(Line::from(vec![
            Span::styled("Content: ", Style::default().fg(theme.text_dim)),
            Span::styled(&self.content, Style::default().fg(Color::White)),
            if is_content_foc {
                Span::styled("█", Style::default().fg(theme.primary))
            } else {
                Span::raw("")
            },
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.block_border(is_content_foc)),
        );
        f.render_widget(content_para, chunks[2]);

        // 4. TTL Field
        let is_ttl_foc = self.active_field == DnsModalField::Ttl;
        let ttl_str = match self.ttl {
            1 => "Auto (Default)".to_string(),
            60 => "1 min (60s)".to_string(),
            300 => "5 mins (300s)".to_string(),
            1800 => "30 mins (1800s)".to_string(),
            3600 => "1 hour (3600s)".to_string(),
            86400 => "1 day (86400s)".to_string(),
            other => format!("{other}s"),
        };
        let ttl_para = Paragraph::new(Line::from(vec![
            Span::styled("TTL: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                ttl_str,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (Space to cycle)", Style::default().fg(theme.text_dim)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.block_border(is_ttl_foc)),
        );
        f.render_widget(ttl_para, chunks[3]);

        // 5. Footer & Hints
        let footer_text = if let Some(err) = &self.error_msg {
            Line::from(vec![
                Span::styled("❌ ", Style::default().fg(theme.error)),
                Span::styled(err, Style::default().fg(theme.error)),
            ])
        } else if popup_width >= 72 {
            Line::from(vec![Span::styled(
                "[Tab] Next Field   [Enter] Save to Cloudflare   [Esc] Cancel",
                Style::default().fg(theme.text_dim),
            )])
        } else {
            Line::from(vec![Span::styled(
                "[Tab] Next   [Enter] Save   [Esc] Cancel",
                Style::default().fg(theme.text_dim),
            )])
        };
        let footer = Paragraph::new(footer_text).wrap(Wrap { trim: true });
        f.render_widget(footer, chunks[4]);
    }
}
