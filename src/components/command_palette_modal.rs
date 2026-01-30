use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::palette::{CommandCategory, PaletteAction, PaletteCommand, PaletteEngine, ScoredCommand};
use crate::theme::Theme;

pub struct CommandPaletteModal {
    pub engine: PaletteEngine,
    pub query: String,
    pub filtered_results: Vec<ScoredCommand>,
    pub selected_idx: usize,
    pub category_filter: Option<CommandCategory>,
}

impl CommandPaletteModal {
    pub fn new(engine: PaletteEngine) -> Self {
        let mut modal = Self {
            engine,
            query: String::new(),
            filtered_results: Vec::new(),
            selected_idx: 0,
            category_filter: None,
        };
        modal.refresh_results();
        modal
    }

    pub fn refresh_results(&mut self) {
        let all_scored = self.engine.search(&self.query);
        if let Some(cat) = self.category_filter {
            self.filtered_results = all_scored
                .into_iter()
                .filter(|sc| sc.command.category == cat)
                .collect();
        } else {
            self.filtered_results = all_scored;
        }

        if self.selected_idx >= self.filtered_results.len() && !self.filtered_results.is_empty() {
            self.selected_idx = self.filtered_results.len() - 1;
        }
    }

    pub fn input_char(&mut self, c: char) {
        self.query.push(c);
        self.selected_idx = 0;
        self.refresh_results();
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.selected_idx = 0;
        self.refresh_results();
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.selected_idx = 0;
        self.refresh_results();
    }

    pub fn nav_up(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn nav_down(&mut self) {
        if !self.filtered_results.is_empty() && self.selected_idx + 1 < self.filtered_results.len() {
            self.selected_idx += 1;
        }
    }

    pub fn nav_top(&mut self) {
        self.selected_idx = 0;
    }

    pub fn nav_bottom(&mut self) {
        if !self.filtered_results.is_empty() {
            self.selected_idx = self.filtered_results.len() - 1;
        }
    }

    pub fn next_category_filter(&mut self) {
        let categories = [
            None,
            Some(CommandCategory::Navigation),
            Some(CommandCategory::Cloudflare),
            Some(CommandCategory::Security),
            Some(CommandCategory::Storage),
            Some(CommandCategory::Workers),
            Some(CommandCategory::Profiles),
            Some(CommandCategory::System),
        ];

        let curr_pos = categories
            .iter()
            .position(|c| *c == self.category_filter)
            .unwrap_or(0);
        let next_pos = (curr_pos + 1) % categories.len();
        self.category_filter = categories[next_pos];
        self.selected_idx = 0;
        self.refresh_results();
    }

    pub fn prev_category_filter(&mut self) {
        let categories = [
            None,
            Some(CommandCategory::Navigation),
            Some(CommandCategory::Cloudflare),
            Some(CommandCategory::Security),
            Some(CommandCategory::Storage),
            Some(CommandCategory::Workers),
            Some(CommandCategory::Profiles),
            Some(CommandCategory::System),
        ];

        let curr_pos = categories
            .iter()
            .position(|c| *c == self.category_filter)
            .unwrap_or(0);
        let prev_pos = if curr_pos == 0 {
            categories.len() - 1
        } else {
            curr_pos - 1
        };
        self.category_filter = categories[prev_pos];
        self.selected_idx = 0;
        self.refresh_results();
    }

    pub fn selected_command(&self) -> Option<PaletteCommand> {
        self.filtered_results.get(self.selected_idx).map(|sc| sc.command.clone())
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_width = 86.min(area.width.saturating_sub(4));
        let popup_height = 24.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Command Palette (Ctrl+P) ")
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input box
                Constraint::Length(2), // Category tabs
                Constraint::Min(9),    // Results table
                Constraint::Length(3), // Preview / Description pane
                Constraint::Length(2), // Key hints footer
            ])
            .margin(1)
            .split(popup_area);

        self.render_search_box(f, chunks[0], theme);
        self.render_category_tabs(f, chunks[1], theme);
        self.render_results_table(f, chunks[2], theme);
        self.render_description_pane(f, chunks[3], theme);
        self.render_footer(f, chunks[4], theme);
    }

    fn render_search_box(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let display_text = if self.query.is_empty() {
            vec![
                Span::styled("> ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled("Type to fuzzy search commands, zones, tools, actions...", Style::default().fg(Color::DarkGray)),
            ]
        } else {
            vec![
                Span::styled("> ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(&self.query, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(theme.primary)),
            ]
        };

        let p = Paragraph::new(Line::from(display_text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border_focus)),
            );

        f.render_widget(p, area);
    }

    fn render_category_tabs(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let categories = [
            ("All", None),
            ("Nav", Some(CommandCategory::Navigation)),
            ("Cloudflare", Some(CommandCategory::Cloudflare)),
            ("Security", Some(CommandCategory::Security)),
            ("Storage", Some(CommandCategory::Storage)),
            ("Workers", Some(CommandCategory::Workers)),
            ("Profiles", Some(CommandCategory::Profiles)),
            ("System", Some(CommandCategory::System)),
        ];

        let selected_idx = categories
            .iter()
            .position(|(_, c)| *c == self.category_filter)
            .unwrap_or(0);

        let titles: Vec<Line> = categories
            .iter()
            .map(|(label, _)| Line::from(*label))
            .collect();

        let tabs = Tabs::new(titles)
            .select(selected_idx)
            .style(Style::default().fg(theme.text_dim))
            .highlight_style(
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            );

        f.render_widget(tabs, area);
    }

    fn render_results_table(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if self.filtered_results.is_empty() {
            let msg = if self.query.is_empty() {
                "No commands available for this category filter."
            } else {
                "No matching commands found. Try adjusting your query."
            };
            let p = Paragraph::new(msg)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(p, area);
            return;
        }

        // Window the results around selected_idx if results exceed viewport
        let visible_height = (area.height as usize).saturating_sub(1);
        let start_idx = if self.selected_idx >= visible_height {
            self.selected_idx.saturating_sub(visible_height - 1)
        } else {
            0
        };

        let rows = self
            .filtered_results
            .iter()
            .enumerate()
            .skip(start_idx)
            .take(visible_height)
            .map(|(idx, sc)| {
                let is_selected = idx == self.selected_idx;
                let badge_color = match sc.command.category {
                    CommandCategory::Navigation => theme.info,
                    CommandCategory::Cloudflare => theme.primary,
                    CommandCategory::Security => theme.error,
                    CommandCategory::Storage => theme.warning,
                    CommandCategory::Workers => Color::Rgb(0, 229, 255),
                    CommandCategory::Profiles => Color::Rgb(180, 100, 240),
                    CommandCategory::System => Color::DarkGray,
                };

                let category_cell = Cell::from(format!(" {:^10} ", sc.command.category.as_str()))
                    .style(Style::default().fg(Color::Black).bg(badge_color).add_modifier(Modifier::BOLD));

                // Title with fuzzy-matched highlights
                let mut title_spans = Vec::new();
                for (char_idx, ch) in sc.command.title.chars().enumerate() {
                    let is_matched = sc.matched_indices.contains(&(char_idx as u32));
                    let span_style = if is_matched {
                        Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                    } else if is_selected {
                        Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    };
                    title_spans.push(Span::styled(ch.to_string(), span_style));
                }
                let title_cell = Cell::from(Line::from(title_spans));

                // Shortcut pill
                let shortcut_text = sc
                    .command
                    .shortcut
                    .as_deref()
                    .map(|s| format!(" [{s}] "))
                    .unwrap_or_default();
                let shortcut_cell = Cell::from(shortcut_text)
                    .style(Style::default().fg(theme.text_dim));

                let row_style = if is_selected {
                    theme.selected_row()
                } else {
                    Style::default()
                };

                Row::new(vec![category_cell, title_cell, shortcut_cell]).style(row_style)
            });

        let table = Table::new(
            rows,
            [
                Constraint::Length(14), // Category badge
                Constraint::Min(40),    // Title
                Constraint::Length(12), // Shortcut
            ],
        );

        f.render_widget(table, area);
    }

    fn render_description_pane(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let selected_cmd = self.selected_command();
        let desc = if let Some(cmd) = &selected_cmd {
            vec![
                Span::styled("Description: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&cmd.description, Style::default().fg(theme.text)),
                Span::styled("  |  Keywords: ", Style::default().fg(Color::DarkGray)),
                Span::styled(cmd.keywords.join(", "), Style::default().fg(theme.text_dim)),
            ]
        } else {
            vec![Span::styled("Select a command to view details.", Style::default().fg(Color::DarkGray))]
        };

        let p = Paragraph::new(Line::from(desc))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border)),
            );

        f.render_widget(p, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect, _theme: &Theme) {
        let p = Paragraph::new(
            " [Enter] Execute  [Tab] Cycle Category  [↑/↓] Navigate  [Esc] Close ",
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
    fn test_modal_query_and_results() {
        let engine = PaletteEngine::new();
        let mut modal = CommandPaletteModal::new(engine);

        assert!(!modal.filtered_results.is_empty());

        modal.input_char('d');
        modal.input_char('n');
        modal.input_char('s');

        assert!(!modal.filtered_results.is_empty());
        assert_eq!(modal.selected_command().unwrap().id, "nav_dns");

        modal.backspace();
        assert_eq!(modal.query, "dn");

        modal.clear_query();
        assert_eq!(modal.query, "");
    }

    #[test]
    fn test_modal_navigation() {
        let engine = PaletteEngine::new();
        let mut modal = CommandPaletteModal::new(engine);

        assert_eq!(modal.selected_idx, 0);
        modal.nav_down();
        assert_eq!(modal.selected_idx, 1);
        modal.nav_up();
        assert_eq!(modal.selected_idx, 0);
        modal.nav_bottom();
        assert_eq!(modal.selected_idx, modal.filtered_results.len() - 1);
        modal.nav_top();
        assert_eq!(modal.selected_idx, 0);
    }

    #[test]
    fn test_modal_category_cycling() {
        let engine = PaletteEngine::new();
        let mut modal = CommandPaletteModal::new(engine);

        assert_eq!(modal.category_filter, None);
        modal.next_category_filter();
        assert_eq!(modal.category_filter, Some(CommandCategory::Navigation));

        for sc in &modal.filtered_results {
            assert_eq!(sc.command.category, CommandCategory::Navigation);
        }

        modal.prev_category_filter();
        assert_eq!(modal.category_filter, None);
    }

    #[test]
    fn test_modal_selection() {
        let engine = PaletteEngine::new();
        let mut modal = CommandPaletteModal::new(engine);

        modal.query = "security".to_string();
        modal.refresh_results();

        let cmd = modal.selected_command().expect("selected command exists");
        assert!(
            cmd.action == PaletteAction::OpenSecurityInspector
                || cmd.action == PaletteAction::SwitchView(5)
        );
    }
}
