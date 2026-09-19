use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::theme::Theme;

pub struct StatusBar {
    pub message: String,
    pub is_error: bool,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            message: "Ready | Press '?' for keybindings cheat sheet".to_string(),
            is_error: false,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        self.is_error = false;
    }

    pub fn set_error(&mut self, err: impl Into<String>) {
        self.message = err.into();
        self.is_error = true;
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme, active_tab: usize) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(45), Constraint::Length(35)])
            .split(area);

        // Dynamic hotkeys based on tab
        let key_hints = match active_tab {
            0 => " [q] Quit  [i] Inspector  [n] Add Record  [e] Edit  [d] Del  [p] Proxy  [Ctrl+P] Palette ",
            1 => " [q] Quit  [i] Inspector  [t/Enter] Live Tail  [j/k] Nav  [r] Sync  [Ctrl+P] Palette ",
            2 => " [q] Quit  [Enter/d] Deployments  [j/k] Nav  [r] Sync  [Ctrl+P] Palette ",
            3 => " [q] Quit  [Tab] View  [j/k] Nav  [Enter] Query  [r] Sync  [Ctrl+P] Palette ",
            4 => " [Space] Pause  [/] Filter  [f] Follow  [c] Clear  [Enter] Log Details  [Ctrl+P] Palette ",
            5 => " [q] Quit  [Tab] Sec Tab  [Enter] Inspect  [f] Filter  [j/k] Nav  [Ctrl+P] Palette ",
            6 => " [q] Quit  [n] Add Domain  [a] Auto-Renew  [l] Lock  [p] Privacy  [/] Filter  [Ctrl+P] Palette ",
            _ => " [q] Quit  [Tab] View  [Space] Pause  [c] Clear  [Ctrl+P] Palette  [?] Help ",
        };

        let hints_widget = Paragraph::new(Line::from(vec![
            Span::styled(
                "KEYS:",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(key_hints, Style::default().fg(theme.text_dim)),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme.border)),
        );

        let msg_style = if self.is_error {
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.success)
        };

        let status_widget = Paragraph::new(Line::from(vec![
            Span::styled("● ", msg_style),
            Span::styled(&self.message, Style::default().fg(theme.text)),
        ]))
        .alignment(ratatui::layout::Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme.border)),
        );

        f.render_widget(hints_widget, chunks[0]);
        f.render_widget(status_widget, chunks[1]);
    }
}
