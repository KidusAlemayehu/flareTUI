use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::cloudflare::client::KvKeyResult;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvFocusPane {
    KeysList,
    ValueViewer,
}

pub struct KvModal {
    pub namespace_id: String,
    pub namespace_title: String,
    pub keys: Vec<KvKeyResult>,
    pub filtered_keys: Vec<KvKeyResult>,
    pub selected_key_idx: usize,
    pub filter_input: String,
    pub is_filtering: bool,
    pub active_pane: KvFocusPane,
    pub current_value: Option<String>,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub value_scroll_y: usize,
}

impl KvModal {
    pub fn new(namespace_id: String, namespace_title: String, keys: Vec<KvKeyResult>) -> Self {
        let filtered = keys.clone();
        Self {
            namespace_id,
            namespace_title,
            keys,
            filtered_keys: filtered,
            selected_key_idx: 0,
            filter_input: String::new(),
            is_filtering: false,
            active_pane: KvFocusPane::KeysList,
            current_value: None,
            is_loading: false,
            error_message: None,
            value_scroll_y: 0,
        }
    }

    pub fn set_keys(&mut self, keys: Vec<KvKeyResult>) {
        self.keys = keys;
        self.apply_filter();
    }

    pub fn apply_filter(&mut self) {
        if self.filter_input.trim().is_empty() {
            self.filtered_keys = self.keys.clone();
        } else {
            let filter_lower = self.filter_input.trim().to_lowercase();
            self.filtered_keys = self
                .keys
                .iter()
                .filter(|k| k.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect();
        }
        if self.selected_key_idx >= self.filtered_keys.len() {
            self.selected_key_idx = self.filtered_keys.len().saturating_sub(1);
        }
    }

    pub fn filter_input_char(&mut self, c: char) {
        self.filter_input.push(c);
        self.apply_filter();
    }

    pub fn filter_input_backspace(&mut self) {
        self.filter_input.pop();
        self.apply_filter();
    }

    pub fn clear_filter(&mut self) {
        self.filter_input.clear();
        self.apply_filter();
    }

    pub fn selected_key(&self) -> Option<&KvKeyResult> {
        self.filtered_keys.get(self.selected_key_idx)
    }

    pub fn switch_pane(&mut self) {
        self.active_pane = match self.active_pane {
            KvFocusPane::KeysList => KvFocusPane::ValueViewer,
            KvFocusPane::ValueViewer => KvFocusPane::KeysList,
        };
    }

    pub fn nav_up(&mut self) {
        match self.active_pane {
            KvFocusPane::KeysList => {
                if self.selected_key_idx > 0 {
                    self.selected_key_idx -= 1;
                }
            }
            KvFocusPane::ValueViewer => {
                self.scroll_value_up();
            }
        }
    }

    pub fn nav_down(&mut self) {
        match self.active_pane {
            KvFocusPane::KeysList => {
                if !self.filtered_keys.is_empty()
                    && self.selected_key_idx + 1 < self.filtered_keys.len()
                {
                    self.selected_key_idx += 1;
                }
            }
            KvFocusPane::ValueViewer => {
                self.scroll_value_down();
            }
        }
    }

    pub fn scroll_value_up(&mut self) {
        if self.value_scroll_y > 0 {
            self.value_scroll_y -= 1;
        }
    }

    pub fn scroll_value_down(&mut self) {
        self.value_scroll_y += 1;
    }

    pub fn set_value(&mut self, val: String) {
        self.is_loading = false;
        self.error_message = None;
        self.value_scroll_y = 0;
        self.current_value = Some(val);
    }

    pub fn set_error(&mut self, err: String) {
        self.is_loading = false;
        self.error_message = Some(err);
    }

    pub fn format_value_lines(raw: &str, theme: &Theme) -> Vec<Line<'static>> {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw) {
            if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                let mut lines = Vec::new();
                for (idx, line) in pretty.lines().enumerate() {
                    let line_num = format!("{:3} │ ", idx + 1);
                    let line_style = if line.trim_start().starts_with('"') && line.contains(':') {
                        // Key-value pair
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        Line::from(vec![
                            Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                parts[0].to_string(),
                                Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(":"),
                            Span::styled(
                                parts.get(1).unwrap_or(&"").to_string(),
                                Style::default().fg(theme.success),
                            ),
                        ])
                    } else if line.trim().starts_with('{') || line.trim().starts_with('}') || line.trim().starts_with('[') || line.trim().starts_with(']') {
                        Line::from(vec![
                            Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                line.to_string(),
                                Style::default().fg(theme.secondary),
                            ),
                        ])
                    } else {
                        Line::from(vec![
                            Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                line.to_string(),
                                Style::default().fg(theme.text),
                            ),
                        ])
                    };
                    lines.push(line_style);
                }
                return lines;
            }
        }

        // Fallback plain text with line numbers
        raw.lines()
            .enumerate()
            .map(|(idx, line)| {
                Line::from(vec![
                    Span::styled(format!("{:3} │ ", idx + 1), Style::default().fg(Color::DarkGray)),
                    Span::styled(line.to_string(), Style::default().fg(theme.text)),
                ])
            })
            .collect()
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 105.min(area.width.saturating_sub(2));
        let popup_height = 30.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let modal_block = Block::default()
            .title(format!(
                " 🗄 Workers KV Inspector: {} [{}] ",
                self.namespace_title, self.namespace_id
            ))
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(modal_block, popup_area);

        let inner_area = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(20),   // Split between keys list and value inspector
                Constraint::Length(3), // Keybindings & feedback bar
            ])
            .split(inner_area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Left: Key Browser & Filter
                Constraint::Percentage(60), // Right: Value Inspector
            ])
            .split(chunks[0]);

        self.render_key_browser(f, horizontal_chunks[0], theme);
        self.render_value_inspector(f, horizontal_chunks[1], theme);
        self.render_bottom_bar(f, chunks[1], theme);
    }

    fn render_key_browser(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let is_keys_focused = self.active_pane == KvFocusPane::KeysList;

        let browser_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Filter box
                Constraint::Min(10),   // Keys table
            ])
            .split(area);

        // 1. Filter input box
        let filter_display = if self.is_filtering {
            format!("{}_", self.filter_input)
        } else if self.filter_input.is_empty() {
            "Type '/' to filter keys...".to_string()
        } else {
            self.filter_input.clone()
        };

        let filter_block = Block::default()
            .title(" Prefix / Key Filter ")
            .title_style(if self.is_filtering {
                theme.title()
            } else {
                Style::default().fg(theme.text_dim)
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.is_filtering {
                theme.block_border(true)
            } else {
                Style::default().fg(theme.border)
            });

        let filter_para = Paragraph::new(filter_display)
            .style(if self.is_filtering || !self.filter_input.is_empty() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(theme.text_dim)
            })
            .block(filter_block);
        f.render_widget(filter_para, browser_chunks[0]);

        // 2. Keys list table
        let mut key_rows = Vec::new();
        if self.filtered_keys.is_empty() {
            let empty_msg = if self.keys.is_empty() {
                "  No keys in namespace"
            } else {
                "  No keys match filter"
            };
            key_rows.push(
                Row::new(vec![Cell::from(empty_msg)])
                    .style(Style::default().fg(theme.text_dim)),
            );
        } else {
            for (idx, key) in self.filtered_keys.iter().enumerate() {
                let is_selected = is_keys_focused && idx == self.selected_key_idx;
                let marker = if is_selected { "▶ " } else { "  " };
                let display = format!("{}{}", marker, key.name);

                let exp_display = if let Some(exp) = key.expiration {
                    format!("exp:{}", exp)
                } else {
                    "never".to_string()
                };

                let row_style = if is_selected {
                    theme.selected_row()
                } else {
                    Style::default().fg(theme.text)
                };

                key_rows.push(
                    Row::new(vec![Cell::from(display), Cell::from(exp_display)]).style(row_style),
                );
            }
        }

        let keys_title = format!(
            " Keys ({}/{}) ",
            self.filtered_keys.len(),
            self.keys.len()
        );

        let keys_block = Block::default()
            .title(keys_title)
            .title_style(if is_keys_focused {
                theme.title()
            } else {
                Style::default().fg(theme.text_dim)
            })
            .borders(Borders::ALL)
            .border_type(if is_keys_focused {
                BorderType::Thick
            } else {
                BorderType::Rounded
            })
            .border_style(theme.block_border(is_keys_focused));

        let keys_table = Table::new(
            key_rows,
            [Constraint::Percentage(70), Constraint::Percentage(30)],
        )
        .header(
            Row::new(vec!["Key Name", "Expires"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(keys_block);

        f.render_widget(keys_table, browser_chunks[1]);
    }

    fn render_value_inspector(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let is_viewer_focused = self.active_pane == KvFocusPane::ValueViewer;

        let selected_key_name = self
            .selected_key()
            .map(|k| k.name.as_str())
            .unwrap_or("No key selected");

        let title = format!(" Value: {} ", selected_key_name);

        let viewer_block = Block::default()
            .title(title)
            .title_style(if is_viewer_focused {
                theme.title()
            } else {
                Style::default().fg(theme.text_dim)
            })
            .borders(Borders::ALL)
            .border_type(if is_viewer_focused {
                BorderType::Thick
            } else {
                BorderType::Rounded
            })
            .border_style(theme.block_border(is_viewer_focused));

        if self.is_loading {
            let p = Paragraph::new("⚡ Fetching value from Cloudflare KV edge storage...")
                .style(Style::default().fg(theme.warning))
                .block(viewer_block);
            f.render_widget(p, area);
            return;
        }

        if let Some(err) = &self.error_message {
            let p = Paragraph::new(format!("Error fetching key:\n\n{}", err))
                .style(Style::default().fg(theme.error))
                .block(viewer_block);
            f.render_widget(p, area);
            return;
        }

        if let Some(val) = &self.current_value {
            let lines = Self::format_value_lines(val, theme);
            let p = Paragraph::new(lines)
                .scroll((self.value_scroll_y as u16, 0))
                .block(viewer_block);
            f.render_widget(p, area);
        } else {
            let p = Paragraph::new("Press [Enter] or [Space] on a key to load its value.")
                .style(Style::default().fg(theme.text_dim))
                .block(viewer_block);
            f.render_widget(p, area);
        }
    }

    fn render_bottom_bar(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let mut spans = Vec::new();

        if self.is_loading {
            spans.push(Span::styled(
                " [ ⚡ LOADING ] ",
                Style::default().bg(theme.warning).fg(Color::Black).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" Reading key from Cloudflare KV..."));
        } else if let Some(val) = &self.current_value {
            spans.push(Span::styled(
                " [ ✔ LOADED ] ",
                Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(format!(" Value size: {} bytes", val.len())));
        } else {
            spans.push(Span::styled(
                " [ READY ] ",
                Style::default().bg(Color::Rgb(60, 60, 80)).fg(Color::White),
            ));
            spans.push(Span::raw(" Select key to inspect"));
        }

        spans.push(Span::styled(
            " | [Tab] Switch Pane  [/] Filter  [Enter] Load Value  [↑/↓] Scroll  [Esc] Close ",
            Style::default().fg(theme.text_dim),
        ));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border));

        let para = Paragraph::new(Line::from(spans)).block(block);
        f.render_widget(para, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_modal_filter_and_navigation() {
        let keys = vec![
            KvKeyResult {
                name: "config:app".to_string(),
                expiration: None,
                metadata: None,
            },
            KvKeyResult {
                name: "config:db".to_string(),
                expiration: None,
                metadata: None,
            },
            KvKeyResult {
                name: "session:user_123".to_string(),
                expiration: Some(1735689600),
                metadata: None,
            },
        ];

        let mut modal = KvModal::new("ns-1".to_string(), "app-kv".to_string(), keys);
        assert_eq!(modal.filtered_keys.len(), 3);
        assert_eq!(modal.selected_key().unwrap().name, "config:app");

        // Filter for "session"
        modal.filter_input_char('s');
        modal.filter_input_char('e');
        modal.filter_input_char('s');
        modal.filter_input_char('s');
        assert_eq!(modal.filtered_keys.len(), 1);
        assert_eq!(modal.selected_key().unwrap().name, "session:user_123");

        modal.clear_filter();
        assert_eq!(modal.filtered_keys.len(), 3);

        modal.nav_down();
        assert_eq!(modal.selected_key_idx, 1);
        assert_eq!(modal.selected_key().unwrap().name, "config:db");
    }

    #[test]
    fn test_kv_modal_json_pretty_formatting() {
        let theme = Theme::default();
        let raw_json = r#"{"name":"flareTUI","version":"0.1.0","active":true}"#;
        let lines = KvModal::format_value_lines(raw_json, &theme);
        assert!(lines.len() >= 4);

        let raw_text = "line 1\nline 2\nline 3";
        let text_lines = KvModal::format_value_lines(raw_text, &theme);
        assert_eq!(text_lines.len(), 3);
    }
}
