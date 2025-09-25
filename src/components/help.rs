use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Row, Table},
    Frame,
};

use crate::theme::Theme;

pub struct HelpModal;

impl HelpModal {
    pub fn render(f: &mut Frame, area: Rect, theme: &Theme) {
        // Center a 60x20 modal
        let popup_width = 72.min(area.width.saturating_sub(4));
        let popup_height = 22.min(area.height.saturating_sub(2));

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        // Clear beneath modal
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" flareTUI - Keyboard Shortcuts ")
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        let rows = vec![
            Row::new(vec!["Tab / Shift+Tab", "Cycle through top tabs"]),
            Row::new(vec!["1 .. 5", "Direct jump to View (DNS, Workers, Pages, D1/KV, Logs)"]),
            Row::new(vec!["j / k or ↓ / ↑", "Navigate items in active list/table"]),
            Row::new(vec!["s", "Switch pane focus between Zone List and DNS Table"]),
            Row::new(vec!["p", "Toggle Cloudflare Proxy (🟠 Proxied / ⚪ DNS Only)"]),
            Row::new(vec!["/", "Open fuzzy search filter on active view"]),
            Row::new(vec!["Esc", "Exit search filter or close modal"]),
            Row::new(vec!["r", "Force refresh data / sync from Cloudflare API"]),
            Row::new(vec!["a / Ctrl+A", "Connect Cloudflare Account via API Token"]),
            Row::new(vec!["?", "Toggle this Help overlay"]),
            Row::new(vec!["q / Ctrl+C", "Quit flareTUI"]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Length(22), Constraint::Min(40)],
        )
        .header(
            Row::new(vec!["Keybinding", "Action"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(block);

        f.render_widget(table, popup_area);
    }
}
