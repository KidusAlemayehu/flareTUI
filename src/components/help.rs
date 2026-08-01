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
        let popup_area = crate::components::centered_rect(area, 75, 64, 105, 85, 20, 32);

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
            Row::new(vec!["i / Enter (DNS)", "Open Zone Inspector (SSL, Dev Mode, Cache Purge)"]),
            Row::new(vec!["i (Workers)", "Inspect Worker Script (Routes, Bindings, Compat)"]),
            Row::new(vec!["t / Enter (Workers)", "Attach Live Tail Log Console via WebSocket"]),
            Row::new(vec!["Enter / d (Pages)", "Open Pages Deployments History Drawer"]),
            Row::new(vec!["r (Deployments)", "Instant 1-keystroke Production Rollback"]),
            Row::new(vec!["l (Deployments)", "Inspect Historical Build Execution Logs"]),
            Row::new(vec!["Space (Live Tail)", "Pause / Resume Live Log Stream"]),
            Row::new(vec!["f (Live Tail)", "Toggle Auto-Follow / Manual History Scroll"]),
            Row::new(vec!["c (Live Tail)", "Clear Live Tail Log Buffer"]),
            Row::new(vec!["Enter (Live Tail)", "Inspect Full Log Payload & Error Traces"]),
            Row::new(vec!["n", "Add new DNS record to active zone"]),
            Row::new(vec!["e", "Edit selected DNS record content/TTL"]),
            Row::new(vec!["d (DNS)", "Delete selected DNS record with confirmation guard"]),
            Row::new(vec!["Shift+E", "Export active zone to RFC 1035 BIND zone file"]),
            Row::new(vec!["p", "Toggle Cloudflare Proxy (🟠 Proxied / ⚪ DNS Only)"]),
            Row::new(vec!["/", "Open search / log filter on active view"]),
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
