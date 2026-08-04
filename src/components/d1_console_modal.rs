use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::cloudflare::client::D1QueryResult;
use crate::db::CachedD1Table;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum D1FocusPane {
    Sidebar,
    QueryEditor,
    ResultsTable,
}

#[derive(Debug, Clone)]
pub struct QueryPreset {
    pub label: &'static str,
    pub template: &'static str,
}

pub const DEFAULT_PRESETS: &[QueryPreset] = &[
    QueryPreset {
        label: "Scan table (limit 25)",
        template: "SELECT * FROM {table} LIMIT 25;",
    },
    QueryPreset {
        label: "Row count summary",
        template: "SELECT COUNT(*) AS total_rows FROM {table};",
    },
    QueryPreset {
        label: "Inspect table schema",
        template: "PRAGMA table_info({table});",
    },
    QueryPreset {
        label: "List all SQLite tables",
        template: "SELECT name, type, sql FROM sqlite_schema WHERE type='table' ORDER BY name;",
    },
];

pub struct D1ConsoleModal {
    pub database_id: String,
    pub database_name: String,
    pub tables: Vec<CachedD1Table>,
    pub selected_table_idx: usize,
    pub focus_pane: D1FocusPane,
    pub query_input: String,
    pub query_draft: String,
    pub cursor_pos: usize,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,
    pub query_result: Option<D1QueryResult>,
    pub is_executing: bool,
    pub error_message: Option<String>,
    pub selected_result_row: usize,
    pub selected_col_idx: usize,
    pub result_scroll_x: std::cell::Cell<usize>,
    pub result_scroll_y: std::cell::Cell<usize>,
    pub sidebar_scroll_y: std::cell::Cell<usize>,
    pub selected_preset_idx: usize,
}

impl D1ConsoleModal {
    pub fn new(database_id: String, database_name: String, tables: Vec<CachedD1Table>) -> Self {
        let initial_query = if let Some(first) = tables.first() {
            format!("SELECT * FROM {} LIMIT 25;", first.table_name)
        } else {
            "SELECT name, type FROM sqlite_schema WHERE type='table';".to_string()
        };
        let len = initial_query.len();

        Self {
            database_id,
            database_name,
            tables,
            selected_table_idx: 0,
            focus_pane: D1FocusPane::QueryEditor,
            query_input: initial_query.clone(),
            query_draft: initial_query,
            cursor_pos: len,
            history: Vec::new(),
            history_idx: None,
            query_result: None,
            is_executing: false,
            error_message: None,
            selected_result_row: 0,
            selected_col_idx: 0,
            result_scroll_x: std::cell::Cell::new(0),
            result_scroll_y: std::cell::Cell::new(0),
            sidebar_scroll_y: std::cell::Cell::new(0),
            selected_preset_idx: 0,
        }
    }

    pub fn selected_table(&self) -> Option<&CachedD1Table> {
        self.tables.get(self.selected_table_idx)
    }

    pub fn record_query_history(&mut self, query: &str) {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.history.last().map(|s| s.as_str()) != Some(trimmed) {
            self.history.push(trimmed.to_string());
        }
        self.history_idx = None;
    }

    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_idx.is_none() {
            self.query_draft = self.query_input.clone();
            let last_idx = self.history.len() - 1;
            self.history_idx = Some(last_idx);
            self.set_query(&self.history[last_idx].clone());
        } else if let Some(idx) = self.history_idx {
            if idx > 0 {
                let prev_idx = idx - 1;
                self.history_idx = Some(prev_idx);
                self.set_query(&self.history[prev_idx].clone());
            }
        }
    }

    pub fn history_next(&mut self) {
        if let Some(idx) = self.history_idx {
            if idx + 1 < self.history.len() {
                let next_idx = idx + 1;
                self.history_idx = Some(next_idx);
                self.set_query(&self.history[next_idx].clone());
            } else {
                self.history_idx = None;
                let draft = self.query_draft.clone();
                self.set_query(&draft);
            }
        }
    }

    pub fn jump_word_left(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let chars: Vec<char> = self.query_input.chars().collect();
        let mut idx = self.cursor_pos.min(chars.len());
        while idx > 0 && chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        while idx > 0 && !chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        self.cursor_pos = idx;
    }

    pub fn jump_word_right(&mut self) {
        let chars: Vec<char> = self.query_input.chars().collect();
        let len = chars.len();
        let mut idx = self.cursor_pos;
        while idx < len && !chars[idx].is_whitespace() {
            idx += 1;
        }
        while idx < len && chars[idx].is_whitespace() {
            idx += 1;
        }
        self.cursor_pos = idx;
    }

    pub fn clear_line(&mut self) {
        self.query_input.clear();
        self.cursor_pos = 0;
    }

    pub fn switch_pane(&mut self) {
        self.focus_pane = match self.focus_pane {
            D1FocusPane::Sidebar => D1FocusPane::QueryEditor,
            D1FocusPane::QueryEditor => D1FocusPane::ResultsTable,
            D1FocusPane::ResultsTable => D1FocusPane::Sidebar,
        };
    }

    pub fn set_focus(&mut self, pane: D1FocusPane) {
        self.focus_pane = pane;
    }

    pub fn input_char(&mut self, c: char) {
        if self.focus_pane == D1FocusPane::QueryEditor {
            if self.cursor_pos >= self.query_input.len() {
                self.query_input.push(c);
                self.cursor_pos = self.query_input.len();
            } else {
                self.query_input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            return;
        }

        // Quick navigation shortcuts for ResultsTable & Sidebar
        match c {
            'j' => self.nav_down(),
            'k' => self.nav_up(),
            'h' => self.nav_left(),
            'l' => self.nav_right(),
            'g' => self.nav_home(),
            'G' => self.nav_end(),
            _ => {}
        }
    }

    pub fn input_backspace(&mut self) {
        if self.focus_pane != D1FocusPane::QueryEditor {
            return;
        }
        if self.cursor_pos > 0 && !self.query_input.is_empty() {
            self.cursor_pos -= 1;
            if self.cursor_pos < self.query_input.len() {
                self.query_input.remove(self.cursor_pos);
            }
        }
    }

    pub fn input_delete(&mut self) {
        if self.focus_pane != D1FocusPane::QueryEditor {
            return;
        }
        if self.cursor_pos < self.query_input.len() {
            self.query_input.remove(self.cursor_pos);
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_pos < self.query_input.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor_pos = self.query_input.len();
    }

    pub fn clear_query(&mut self) {
        self.query_input.clear();
        self.cursor_pos = 0;
    }

    pub fn set_query(&mut self, query: &str) {
        self.query_input = query.to_string();
        self.cursor_pos = self.query_input.len();
    }

    pub fn apply_preset(&mut self, preset_idx: usize) {
        if preset_idx >= DEFAULT_PRESETS.len() {
            return;
        }
        let preset = &DEFAULT_PRESETS[preset_idx];
        let table_name = self
            .selected_table()
            .map(|t| t.table_name.as_str())
            .unwrap_or("users");

        let query = preset.template.replace("{table}", table_name);
        self.set_query(&query);
        self.focus_pane = D1FocusPane::QueryEditor;
    }

    pub fn nav_up(&mut self) {
        match self.focus_pane {
            D1FocusPane::Sidebar => {
                if self.selected_table_idx > 0 {
                    self.selected_table_idx -= 1;
                }
            }
            D1FocusPane::ResultsTable => {
                if self.selected_result_row > 0 {
                    self.selected_result_row -= 1;
                }
            }
            D1FocusPane::QueryEditor => {
                self.history_prev();
            }
        }
    }

    pub fn nav_down(&mut self) {
        match self.focus_pane {
            D1FocusPane::Sidebar => {
                if !self.tables.is_empty() && self.selected_table_idx + 1 < self.tables.len() {
                    self.selected_table_idx += 1;
                }
            }
            D1FocusPane::ResultsTable => {
                if let Some(res) = &self.query_result {
                    if let Some(rows) = &res.results {
                        if !rows.is_empty() && self.selected_result_row + 1 < rows.len() {
                            self.selected_result_row += 1;
                        }
                    }
                }
            }
            D1FocusPane::QueryEditor => {
                self.history_next();
            }
        }
    }

    pub fn nav_left(&mut self) {
        match self.focus_pane {
            D1FocusPane::QueryEditor => self.cursor_left(),
            D1FocusPane::ResultsTable => {
                if self.selected_col_idx > 0 {
                    self.selected_col_idx -= 1;
                } else {
                    self.scroll_results_left();
                }
            }
            D1FocusPane::Sidebar => {}
        }
    }

    pub fn nav_right(&mut self) {
        match self.focus_pane {
            D1FocusPane::QueryEditor => self.cursor_right(),
            D1FocusPane::ResultsTable => {
                let col_count = self.get_columns().len();
                if col_count > 0 && self.selected_col_idx + 1 < col_count {
                    self.selected_col_idx += 1;
                } else {
                    self.scroll_results_right();
                }
            }
            D1FocusPane::Sidebar => {}
        }
    }

    pub fn page_up(&mut self) {
        match self.focus_pane {
            D1FocusPane::Sidebar => {
                self.selected_table_idx = self.selected_table_idx.saturating_sub(10);
            }
            D1FocusPane::ResultsTable => {
                self.selected_result_row = self.selected_result_row.saturating_sub(10);
            }
            D1FocusPane::QueryEditor => {
                self.history_prev();
            }
        }
    }

    pub fn page_down(&mut self) {
        match self.focus_pane {
            D1FocusPane::Sidebar => {
                if !self.tables.is_empty() {
                    self.selected_table_idx = (self.selected_table_idx + 10).min(self.tables.len().saturating_sub(1));
                }
            }
            D1FocusPane::ResultsTable => {
                if let Some(res) = &self.query_result {
                    if let Some(rows) = &res.results {
                        if !rows.is_empty() {
                            self.selected_result_row =
                                (self.selected_result_row + 10).min(rows.len().saturating_sub(1));
                        }
                    }
                }
            }
            D1FocusPane::QueryEditor => {
                self.history_next();
            }
        }
    }

    pub fn nav_home(&mut self) {
        match self.focus_pane {
            D1FocusPane::Sidebar => {
                self.selected_table_idx = 0;
            }
            D1FocusPane::ResultsTable => {
                self.selected_result_row = 0;
                self.selected_col_idx = 0;
            }
            D1FocusPane::QueryEditor => {
                self.cursor_home();
            }
        }
    }

    pub fn nav_end(&mut self) {
        match self.focus_pane {
            D1FocusPane::Sidebar => {
                if !self.tables.is_empty() {
                    self.selected_table_idx = self.tables.len().saturating_sub(1);
                }
            }
            D1FocusPane::ResultsTable => {
                if let Some(res) = &self.query_result {
                    if let Some(rows) = &res.results {
                        if !rows.is_empty() {
                            self.selected_result_row = rows.len().saturating_sub(1);
                        }
                    }
                }
            }
            D1FocusPane::QueryEditor => {
                self.cursor_end();
            }
        }
    }

    pub fn scroll_results_left(&mut self) {
        let cur = self.result_scroll_x.get();
        if cur > 0 {
            self.result_scroll_x.set(cur - 1);
        }
    }

    pub fn scroll_results_right(&mut self) {
        let cur = self.result_scroll_x.get();
        self.result_scroll_x.set(cur + 1);
    }

    pub fn set_result(&mut self, result: D1QueryResult) {
        self.is_executing = false;
        self.error_message = None;
        self.selected_result_row = 0;
        self.selected_col_idx = 0;
        self.result_scroll_x.set(0);
        self.result_scroll_y.set(0);
        self.query_result = Some(result);
    }

    pub fn set_error(&mut self, error: String) {
        self.is_executing = false;
        self.error_message = Some(error);
    }

    /// Extract column names from query result rows
    pub fn get_columns(&self) -> Vec<String> {
        let mut cols = Vec::new();
        if let Some(res) = &self.query_result {
            if let Some(rows) = &res.results {
                if let Some(serde_json::Value::Object(map)) = rows.first() {
                    for key in map.keys() {
                        cols.push(key.clone());
                    }
                }
            }
        }
        cols
    }

    /// Convert row value to displayable string
    pub fn format_cell_value(val: Option<&serde_json::Value>) -> (String, Style) {
        match val {
            None => ("NULL".to_string(), Style::default().fg(Color::DarkGray)),
            Some(serde_json::Value::Null) => {
                ("NULL".to_string(), Style::default().fg(Color::DarkGray))
            }
            Some(serde_json::Value::Bool(b)) => {
                (b.to_string(), Style::default().fg(Color::Rgb(250, 200, 50)))
            }
            Some(serde_json::Value::Number(n)) => {
                (n.to_string(), Style::default().fg(Color::Rgb(80, 170, 240)))
            }
            Some(serde_json::Value::String(s)) => {
                (s.clone(), Style::default().fg(Color::White))
            }
            Some(serde_json::Value::Array(a)) => (
                format!("[{} items]", a.len()),
                Style::default().fg(Color::Rgb(200, 150, 255)),
            ),
            Some(serde_json::Value::Object(o)) => (
                format!("{{{} keys}}", o.len()),
                Style::default().fg(Color::Rgb(200, 150, 255)),
            ),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = crate::components::centered_rect(area, 92, 76, 145, 88, 20, 48);

        f.render_widget(Clear, popup_area);

        let modal_block = Block::default()
            .title(format!(
                " 🗄 D1 SQL Console: {} [{}] ",
                self.database_name, self.database_id
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

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30), // Left sidebar: Tables & Presets
                Constraint::Min(40),    // Right main pane: SQL + Results + Metrics
            ])
            .split(inner_area);

        self.render_sidebar(f, horizontal_chunks[0], theme);
        self.render_main_pane(f, horizontal_chunks[1], theme);
    }

    fn render_sidebar(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Tables schema list
                Constraint::Length(7), // Quick Presets
            ])
            .split(area);

        let is_sidebar_focused = self.focus_pane == D1FocusPane::Sidebar;

        // 1. Tables list
        let tables_area = sidebar_chunks[0];
        let visible_sidebar_rows = (tables_area.height.saturating_sub(2) as usize).max(1);
        let total_tables = self.tables.len();

        let mut sidebar_scroll = self.sidebar_scroll_y.get();
        if self.selected_table_idx < sidebar_scroll {
            sidebar_scroll = self.selected_table_idx;
        } else if self.selected_table_idx >= sidebar_scroll + visible_sidebar_rows {
            sidebar_scroll = self.selected_table_idx + 1 - visible_sidebar_rows;
        }
        let max_sidebar_scroll = total_tables.saturating_sub(visible_sidebar_rows);
        if sidebar_scroll > max_sidebar_scroll {
            sidebar_scroll = max_sidebar_scroll;
        }
        self.sidebar_scroll_y.set(sidebar_scroll);

        let start_tbl = sidebar_scroll;
        let end_tbl = (start_tbl + visible_sidebar_rows).min(total_tables);

        let mut table_rows = Vec::new();
        if self.tables.is_empty() {
            table_rows.push(
                Row::new(vec![Cell::from("  No tables cached")])
                    .style(Style::default().fg(theme.text_dim)),
            );
        } else {
            for (offset, tbl) in self.tables[start_tbl..end_tbl].iter().enumerate() {
                let actual_tbl_idx = start_tbl + offset;
                let is_selected = is_sidebar_focused && actual_tbl_idx == self.selected_table_idx;
                let marker = if is_selected { "▶ " } else { "  " };
                let display = format!("{}{}", marker, tbl.table_name);
                let meta = format!("~{} rows", tbl.row_count);

                let row_style = if is_selected {
                    theme.selected_row()
                } else {
                    Style::default().fg(theme.text)
                };

                table_rows.push(Row::new(vec![Cell::from(display), Cell::from(meta)]).style(row_style));
            }
        }

        let title_str = if total_tables > visible_sidebar_rows {
            format!(
                " Schema Tables ({}) [Row {}/{}] ",
                total_tables,
                self.selected_table_idx + 1,
                total_tables
            )
        } else if total_tables > 0 {
            format!(" Schema Tables ({}) ", total_tables)
        } else {
            " Schema Tables ".to_string()
        };

        let tables_block = Block::default()
            .title(title_str)
            .title_style(if is_sidebar_focused {
                theme.title()
            } else {
                Style::default().fg(theme.text_dim)
            })
            .borders(Borders::ALL)
            .border_type(if is_sidebar_focused {
                BorderType::Thick
            } else {
                BorderType::Rounded
            })
            .border_style(theme.block_border(is_sidebar_focused));

        let tables_table = Table::new(
            table_rows,
            [Constraint::Percentage(65), Constraint::Percentage(35)],
        )
        .block(tables_block);

        f.render_widget(tables_table, sidebar_chunks[0]);

        // 2. Query Presets
        let preset_lines = vec![
            Line::from(vec![
                Span::styled("[1] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                Span::raw("Scan LIMIT 25"),
            ]),
            Line::from(vec![
                Span::styled("[2] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                Span::raw("Count Rows"),
            ]),
            Line::from(vec![
                Span::styled("[3] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                Span::raw("Table Schema"),
            ]),
            Line::from(vec![
                Span::styled("[4] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                Span::raw("sqlite_schema"),
            ]),
        ];

        let presets_block = Block::default()
            .title(" Templates ")
            .title_style(Style::default().fg(theme.text_dim))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border));

        let presets_para = Paragraph::new(preset_lines).block(presets_block);
        f.render_widget(presets_para, sidebar_chunks[1]);
    }

    fn render_main_pane(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // SQL Query Input editor
                Constraint::Min(12),   // Results Grid Table
                Constraint::Length(3), // Metrics & Keybindings bar
            ])
            .split(area);

        let is_editor_focused = self.focus_pane == D1FocusPane::QueryEditor;
        let is_results_focused = self.focus_pane == D1FocusPane::ResultsTable;

        // 1. SQL Query Editor
        let mut display_query = self.query_input.clone();
        if is_editor_focused {
            if self.cursor_pos >= display_query.len() {
                display_query.push('█');
            } else {
                display_query.insert(self.cursor_pos, '█');
            }
        }

        let editor_block = Block::default()
            .title(" SQL Query [Ctrl+Enter to Execute] ")
            .title_style(if is_editor_focused {
                theme.title()
            } else {
                Style::default().fg(theme.text_dim)
            })
            .borders(Borders::ALL)
            .border_type(if is_editor_focused {
                BorderType::Thick
            } else {
                BorderType::Rounded
            })
            .border_style(theme.block_border(is_editor_focused));

        let query_para = Paragraph::new(display_query)
            .style(Style::default().fg(Color::Rgb(240, 240, 255)))
            .block(editor_block);
        f.render_widget(query_para, main_chunks[0]);

        // 2. Results Grid Table
        self.render_results_table(f, main_chunks[1], is_results_focused, theme);

        // 3. Execution Metrics & Status Bar
        let mut metrics_spans = Vec::new();
        if self.is_executing {
            metrics_spans.push(Span::styled(
                " [ ⚡ EXECUTING ] ",
                Style::default().bg(theme.warning).fg(Color::Black).add_modifier(Modifier::BOLD),
            ));
            metrics_spans.push(Span::raw(" Running SQL query against Cloudflare D1 edge replica..."));
        } else if let Some(err) = &self.error_message {
            metrics_spans.push(Span::styled(
                " [ ✖ FAILED ] ",
                Style::default().bg(theme.error).fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            metrics_spans.push(Span::styled(
                format!(" {} ", err),
                Style::default().fg(theme.error),
            ));
        } else if let Some(res) = &self.query_result {
            if let Some(meta) = &res.meta {
                let dur = meta.duration.unwrap_or(0.0);
                let read = meta.rows_read.unwrap_or(0);
                let written = meta.rows_written.unwrap_or(0);
                metrics_spans.push(Span::styled(
                    format!(" [ ✔ OK {:.1}ms ] ", dur),
                    Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD),
                ));
                metrics_spans.push(Span::raw(format!(
                    " Read: {} | Written: {}",
                    read, written
                )));
                if let Some(changes) = meta.changes {
                    if changes > 0 {
                        metrics_spans.push(Span::raw(format!(" | Changes: {}", changes)));
                    }
                }
                if let Some(last_id) = meta.last_row_id {
                    metrics_spans.push(Span::raw(format!(" | Last ID: {}", last_id)));
                }
            } else {
                metrics_spans.push(Span::styled(
                    " [ ✔ OK ] ",
                    Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD),
                ));
                metrics_spans.push(Span::raw(" Query completed"));
            }
        } else {
            metrics_spans.push(Span::styled(
                " [ READY ] ",
                Style::default().bg(Color::Rgb(60, 60, 80)).fg(Color::White),
            ));
            metrics_spans.push(Span::raw(" Enter query or select template"));
        }

        let key_hints = match self.focus_pane {
            D1FocusPane::QueryEditor => {
                " | [Tab] Results  [↑/↓ History]  [1-4] Presets  [Ctrl+Enter] Run  [Esc] Close "
            }
            D1FocusPane::ResultsTable => {
                " | [Tab] Sidebar  [↑/↓/PgUp/PgDn] Scroll  [←/→] Cols  [Home/End] Extremes  [Esc] Close "
            }
            D1FocusPane::Sidebar => {
                " | [Tab] Query  [↑/↓] Tables  [1-4] Presets  [Ctrl+Enter] Run  [Esc] Close "
            }
        };

        metrics_spans.push(Span::styled(
            key_hints,
            Style::default().fg(theme.text_dim),
        ));

        let metrics_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border));

        let metrics_para = Paragraph::new(Line::from(metrics_spans))
            .wrap(Wrap { trim: true })
            .block(metrics_block);
        f.render_widget(metrics_para, main_chunks[2]);
    }

    fn render_results_table(
        &self,
        f: &mut Frame,
        area: Rect,
        is_focused: bool,
        theme: &Theme,
    ) {
        let columns = self.get_columns();
        let row_count = self
            .query_result
            .as_ref()
            .and_then(|r| r.results.as_ref())
            .map(|rows| rows.len())
            .unwrap_or(0);

        // 1. Calculate vertical viewport windowing
        // Non-data rows: top border (1) + bottom border (1) + header row (1) + header bottom margin (1) = 4
        let visible_height = (area.height.saturating_sub(4) as usize).max(1);
        let mut scroll_y = self.result_scroll_y.get();
        if self.selected_result_row < scroll_y {
            scroll_y = self.selected_result_row;
        } else if self.selected_result_row >= scroll_y + visible_height {
            scroll_y = self.selected_result_row + 1 - visible_height;
        }
        let max_scroll_y = row_count.saturating_sub(visible_height);
        if scroll_y > max_scroll_y {
            scroll_y = max_scroll_y;
        }
        self.result_scroll_y.set(scroll_y);

        let start_row = scroll_y;
        let end_row = (start_row + visible_height).min(row_count);

        // 2. Calculate horizontal column windowing
        let col_count = columns.len();
        let available_width = area.width.saturating_sub(2) as usize;
        let col_width: usize = 18;
        let visible_cols = (available_width / col_width).max(1);

        let mut scroll_x = self.result_scroll_x.get();
        if self.selected_col_idx < scroll_x {
            scroll_x = self.selected_col_idx;
        } else if self.selected_col_idx >= scroll_x + visible_cols {
            scroll_x = self.selected_col_idx + 1 - visible_cols;
        }
        let max_scroll_x = col_count.saturating_sub(visible_cols);
        if scroll_x > max_scroll_x {
            scroll_x = max_scroll_x;
        }
        self.result_scroll_x.set(scroll_x);

        let start_col = scroll_x;
        let end_col = (start_col + visible_cols).min(col_count);

        let title_info = if row_count > 0 {
            let row_range_str = if row_count > visible_height {
                format!(" (Showing {}-{} of {})", start_row + 1, end_row, row_count)
            } else {
                String::new()
            };
            let col_range_str = if col_count > visible_cols {
                format!(" (Cols {}-{} of {})", start_col + 1, end_col, col_count)
            } else {
                String::new()
            };
            format!(
                " Tabular Results Grid ({} rows, {} cols) [Row {}/{}, Col {}/{}] {}{} ",
                row_count,
                col_count,
                self.selected_result_row + 1,
                row_count,
                self.selected_col_idx + 1,
                col_count,
                row_range_str,
                col_range_str,
            )
        } else {
            format!(" Tabular Results Grid ({} cols) ", col_count)
        };

        let table_block = Block::default()
            .title(title_info)
            .title_style(if is_focused {
                theme.title()
            } else {
                Style::default().fg(theme.text_dim)
            })
            .borders(Borders::ALL)
            .border_type(if is_focused {
                BorderType::Thick
            } else {
                BorderType::Rounded
            })
            .border_style(theme.block_border(is_focused));

        if self.is_executing {
            let p = Paragraph::new("⚡ Executing SQL query against edge database...")
                .style(Style::default().fg(theme.warning))
                .block(table_block);
            f.render_widget(p, area);
            return;
        }

        if let Some(err) = &self.error_message {
            let p = Paragraph::new(format!("SQL Execution Error:\n\n{}", err))
                .style(Style::default().fg(theme.error))
                .block(table_block);
            f.render_widget(p, area);
            return;
        }

        if columns.is_empty() {
            let msg = if self.query_result.is_some() {
                "Query executed successfully. (0 rows returned or empty result set)"
            } else {
                "No results yet. Enter SQL query above and press [Ctrl+Enter]."
            };
            let p = Paragraph::new(msg)
                .style(Style::default().fg(theme.text_dim))
                .block(table_block);
            f.render_widget(p, area);
            return;
        }

        let mut header_cells = Vec::new();
        for col in &columns[start_col..end_col] {
            header_cells.push(Cell::from(col.clone()));
        }
        let header_row = Row::new(header_cells)
            .style(theme.header_style())
            .bottom_margin(1);

        let mut data_rows = Vec::new();
        if let Some(res) = &self.query_result {
            if let Some(rows) = &res.results {
                if start_row < rows.len() {
                    for (offset, row_val) in rows[start_row..end_row].iter().enumerate() {
                        let actual_row_idx = start_row + offset;
                        let mut cells = Vec::new();
                        let is_selected = is_focused && actual_row_idx == self.selected_result_row;

                        if let serde_json::Value::Object(map) = row_val {
                            for (col_offset, col) in columns[start_col..end_col].iter().enumerate() {
                                let actual_col_idx = start_col + col_offset;
                                let (val_str, style) = Self::format_cell_value(map.get(col));
                                let is_cell_focused = is_selected && actual_col_idx == self.selected_col_idx;
                                let cell_style = if is_cell_focused {
                                    Style::default()
                                        .bg(Color::Rgb(70, 70, 100))
                                        .fg(Color::Rgb(255, 220, 100))
                                        .add_modifier(Modifier::BOLD)
                                } else if is_selected {
                                    theme.selected_row()
                                } else {
                                    style
                                };
                                cells.push(Cell::from(val_str).style(cell_style));
                            }
                        }

                        let row_style = if is_selected {
                            theme.selected_row()
                        } else {
                            Style::default()
                        };
                        data_rows.push(Row::new(cells).style(row_style));
                    }
                }
            }
        }

        let widths: Vec<Constraint> = (start_col..end_col)
            .map(|_| Constraint::Min(18))
            .collect();

        let table = Table::new(data_rows, widths)
            .header(header_row)
            .block(table_block);

        f.render_widget(table, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_d1_console_modal_initialization() {
        let tables = vec![
            CachedD1Table {
                database_id: "db-123".to_string(),
                table_name: "users".to_string(),
                schema_sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);".to_string(),
                row_count: 100,
            },
            CachedD1Table {
                database_id: "db-123".to_string(),
                table_name: "orders".to_string(),
                schema_sql: "CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER);".to_string(),
                row_count: 450,
            },
        ];

        let modal = D1ConsoleModal::new("db-123".to_string(), "prod_db".to_string(), tables);
        assert_eq!(modal.focus_pane, D1FocusPane::QueryEditor);
        assert_eq!(modal.query_input, "SELECT * FROM users LIMIT 25;");
        assert_eq!(modal.selected_table_idx, 0);
        assert_eq!(modal.selected_table().unwrap().table_name, "users");
    }

    #[test]
    fn test_d1_console_input_and_presets() {
        let mut modal = D1ConsoleModal::new("db-123".to_string(), "prod_db".to_string(), vec![]);
        modal.clear_query();
        assert_eq!(modal.query_input, "");

        modal.input_char('S');
        modal.input_char('E');
        modal.input_char('L');
        modal.input_char('1');
        assert_eq!(modal.query_input, "SEL1");

        modal.input_backspace();
        assert_eq!(modal.query_input, "SEL");

        modal.apply_preset(1); // count(*)
        assert!(modal.query_input.contains("COUNT(*)"));

        modal.switch_pane();
        assert_eq!(modal.focus_pane, D1FocusPane::ResultsTable);
        modal.switch_pane();
        assert_eq!(modal.focus_pane, D1FocusPane::Sidebar);
        modal.switch_pane();
        assert_eq!(modal.focus_pane, D1FocusPane::QueryEditor);
    }

    #[test]
    fn test_d1_console_results_parsing() {
        let mut modal = D1ConsoleModal::new("db-123".to_string(), "prod_db".to_string(), vec![]);
        let result = D1QueryResult {
            results: Some(vec![
                json!({ "id": 1, "username": "alice", "active": true, "extra": serde_json::Value::Null }),
                json!({ "id": 2, "username": "bob", "active": false, "extra": "not null" }),
            ]),
            success: Some(true),
            meta: Some(crate::cloudflare::client::D1QueryMeta {
                duration: Some(12.4),
                rows_read: Some(2),
                rows_written: Some(0),
                changes: Some(0),
                last_row_id: None,
            }),
        };

        modal.set_result(result);
        assert!(!modal.is_executing);
        assert!(modal.error_message.is_none());

        let cols = modal.get_columns();
        assert!(cols.contains(&"id".to_string()));
        assert!(cols.contains(&"username".to_string()));
        assert!(cols.contains(&"active".to_string()));
        assert!(cols.contains(&"extra".to_string()));

        let (null_str, _) = D1ConsoleModal::format_cell_value(Some(&serde_json::Value::Null));
        assert_eq!(null_str, "NULL");

        let (bool_str, _) = D1ConsoleModal::format_cell_value(Some(&serde_json::Value::Bool(true)));
        assert_eq!(bool_str, "true");
    }

    #[test]
    fn test_d1_console_history_and_navigation() {
        let mut modal = D1ConsoleModal::new("db-123".to_string(), "prod_db".to_string(), vec![]);
        modal.record_query_history("SELECT 1;");
        modal.record_query_history("SELECT 2;");
        modal.record_query_history("SELECT 3;");

        modal.set_query("SELECT current_draft;");
        modal.history_prev();
        assert_eq!(modal.query_input, "SELECT 3;");
        modal.history_prev();
        assert_eq!(modal.query_input, "SELECT 2;");
        modal.history_next();
        assert_eq!(modal.query_input, "SELECT 3;");
        modal.history_next();
        assert_eq!(modal.query_input, "SELECT current_draft;");

        // Word jumping
        modal.set_query("SELECT id FROM users;");
        modal.cursor_home();
        assert_eq!(modal.cursor_pos, 0);
        modal.jump_word_right();
        assert_eq!(modal.cursor_pos, 7); // after "SELECT "
        modal.jump_word_left();
        assert_eq!(modal.cursor_pos, 0);

        modal.clear_line();
        assert_eq!(modal.query_input, "");
        assert_eq!(modal.cursor_pos, 0);
    }

    #[test]
    fn test_d1_console_viewport_scrolling_and_large_result_sets() {
        let mut modal = D1ConsoleModal::new("db-123".to_string(), "prod_db".to_string(), vec![]);

        // Construct 25 rows (user had 22 records clipped at 18)
        let mut rows = Vec::new();
        for i in 1..=25 {
            rows.push(json!({
                "id": i,
                "name": format!("user_{}", i),
                "role": "member",
            }));
        }

        let result = D1QueryResult {
            results: Some(rows),
            success: Some(true),
            meta: None,
        };

        modal.set_result(result);
        modal.set_focus(D1FocusPane::ResultsTable);
        assert_eq!(modal.selected_result_row, 0);

        // Navigate down past row 18 (e.g. to row 21 which is the 22nd record)
        for _ in 0..21 {
            modal.nav_down();
        }
        assert_eq!(modal.selected_result_row, 21);

        // Navigate to the 25th row (index 24)
        modal.nav_down();
        modal.nav_down();
        modal.nav_down();
        assert_eq!(modal.selected_result_row, 24);

        // Clamp at bottom
        modal.nav_down();
        assert_eq!(modal.selected_result_row, 24);

        // Test nav_home and nav_end
        modal.nav_home();
        assert_eq!(modal.selected_result_row, 0);

        modal.nav_end();
        assert_eq!(modal.selected_result_row, 24);

        // Test page_up and page_down
        modal.page_up();
        assert_eq!(modal.selected_result_row, 14);
        modal.page_down();
        assert_eq!(modal.selected_result_row, 24);

        // Test vim navigation keys in ResultsTable
        modal.input_char('k'); // up
        assert_eq!(modal.selected_result_row, 23);
        modal.input_char('j'); // down
        assert_eq!(modal.selected_result_row, 24);
        modal.input_char('g'); // top
        assert_eq!(modal.selected_result_row, 0);
        modal.input_char('G'); // bottom
        assert_eq!(modal.selected_result_row, 24);

        // Render with Ratatui TestBackend to verify viewport windowing
        let backend = ratatui::backend::TestBackend::new(100, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                let area = f.area();
                modal.render(f, area, &theme);
            })
            .unwrap();

        // At row 24, result_scroll_y should have scrolled down so row 24 is visible
        assert!(modal.result_scroll_y.get() > 0);
    }
}

