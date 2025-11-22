use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
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
    pub cursor_pos: usize,
    pub query_result: Option<D1QueryResult>,
    pub is_executing: bool,
    pub error_message: Option<String>,
    pub selected_result_row: usize,
    pub result_scroll_x: usize,
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
            query_input: initial_query,
            cursor_pos: len,
            query_result: None,
            is_executing: false,
            error_message: None,
            selected_result_row: 0,
            result_scroll_x: 0,
            selected_preset_idx: 0,
        }
    }

    pub fn selected_table(&self) -> Option<&CachedD1Table> {
        self.tables.get(self.selected_table_idx)
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
        if self.focus_pane != D1FocusPane::QueryEditor {
            return;
        }
        if self.cursor_pos >= self.query_input.len() {
            self.query_input.push(c);
            self.cursor_pos = self.query_input.len();
        } else {
            self.query_input.insert(self.cursor_pos, c);
            self.cursor_pos += 1;
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
            D1FocusPane::QueryEditor => {}
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
            D1FocusPane::QueryEditor => {}
        }
    }

    pub fn scroll_results_left(&mut self) {
        if self.result_scroll_x > 0 {
            self.result_scroll_x -= 1;
        }
    }

    pub fn scroll_results_right(&mut self) {
        self.result_scroll_x += 1;
    }

    pub fn set_result(&mut self, result: D1QueryResult) {
        self.is_executing = false;
        self.error_message = None;
        self.selected_result_row = 0;
        self.result_scroll_x = 0;
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
        let popup_width = 110.min(area.width.saturating_sub(2));
        let popup_height = 32.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

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
        let mut table_rows = Vec::new();
        if self.tables.is_empty() {
            table_rows.push(
                Row::new(vec![Cell::from("  No tables cached")])
                    .style(Style::default().fg(theme.text_dim)),
            );
        } else {
            for (idx, tbl) in self.tables.iter().enumerate() {
                let is_selected = is_sidebar_focused && idx == self.selected_table_idx;
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

        let tables_block = Block::default()
            .title(" Schema Tables ")
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
                " ⚡ Executing query on Cloudflare D1 edge...",
                Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
            ));
        } else if let Some(err) = &self.error_message {
            metrics_spans.push(Span::styled(
                format!(" ✖ Error: {} ", err),
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
            ));
        } else if let Some(res) = &self.query_result {
            if let Some(meta) = &res.meta {
                let dur = meta.duration.unwrap_or(0.0);
                let read = meta.rows_read.unwrap_or(0);
                let written = meta.rows_written.unwrap_or(0);
                metrics_spans.push(Span::styled(
                    format!(" ⚡ {:.2}ms ", dur),
                    Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
                ));
                metrics_spans.push(Span::raw(format!(
                    "| Rows Read: {} | Written: {} ",
                    read, written
                )));
            } else {
                metrics_spans.push(Span::styled(
                    " ✔ Query succeeded ",
                    Style::default().fg(theme.success),
                ));
            }
        } else {
            metrics_spans.push(Span::styled(
                " Ready to execute SQL ",
                Style::default().fg(theme.text_dim),
            ));
        }

        metrics_spans.push(Span::styled(
            " | [Tab] Switch Pane  [1-4] Template  [Ctrl+Enter] Run  [Esc] Close ",
            Style::default().fg(theme.text_dim),
        ));

        let metrics_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border));

        let metrics_para = Paragraph::new(Line::from(metrics_spans)).block(metrics_block);
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

        let table_block = Block::default()
            .title(format!(
                " Tabular Results Grid ({} cols) ",
                columns.len()
            ))
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
        for col in &columns {
            header_cells.push(Cell::from(col.clone()));
        }
        let header_row = Row::new(header_cells)
            .style(theme.header_style())
            .bottom_margin(1);

        let mut data_rows = Vec::new();
        if let Some(res) = &self.query_result {
            if let Some(rows) = &res.results {
                for (row_idx, row_val) in rows.iter().enumerate() {
                    let mut cells = Vec::new();
                    let is_selected = is_focused && row_idx == self.selected_result_row;

                    if let serde_json::Value::Object(map) = row_val {
                        for col in &columns {
                            let (val_str, style) = Self::format_cell_value(map.get(col));
                            let cell_style = if is_selected {
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

        let widths: Vec<Constraint> = columns
            .iter()
            .map(|_| Constraint::Length(18))
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
}
