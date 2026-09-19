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
        let popup_area = crate::components::centered_rect(area, 80, 68, 120, 88, 20, 44);

        // Clear beneath modal
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" flareTUI - Keyboard Shortcuts ")
            .title_style(theme.title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary));

        let rows = vec![
            Row::new(vec![
                "Ctrl+P",
                "Open Universal Command Palette (Fuzzy Search & Actions)",
            ]),
            Row::new(vec!["Tab / Shift+Tab", "Cycle through top navigation tabs"]),
            Row::new(vec![
                "1 .. 7",
                "Direct jump to View (DNS, Workers, Pages, D1/KV, Logs, Security, Domains)",
            ]),
            Row::new(vec!["Ctrl+U", "Open Multi-Account Profile Switcher modal"]),
            Row::new(vec!["Ctrl+L", "Logout active Cloudflare account session"]),
            Row::new(vec![
                "j / k or ↓ / ↑",
                "Navigate items in active list, table, or drawer",
            ]),
            Row::new(vec![
                "s",
                "Switch pane focus between primary & secondary views",
            ]),
            Row::new(vec!["/", "Open live search / query filter on active view"]),
            Row::new(vec![
                "Esc",
                "Exit search filter or close active modal / drawer",
            ]),
            Row::new(vec![
                "r",
                "Force refresh data / revalidate cache from Cloudflare API",
            ]),
            Row::new(vec![
                "a / Ctrl+A",
                "Connect Cloudflare Account via API Token",
            ]),
            Row::new(vec!["?", "Toggle this Keyboard Shortcuts Help overlay"]),
            Row::new(vec!["q / Ctrl+C", "Quit flareTUI"]),
            Row::new(vec!["n (DNS)", "Add new DNS record to active zone"]),
            Row::new(vec!["e (DNS)", "Edit selected DNS record content / TTL"]),
            Row::new(vec![
                "d (DNS)",
                "Delete selected DNS record with confirmation guard",
            ]),
            Row::new(vec![
                "p (DNS)",
                "Toggle Cloudflare Proxy (🟠 Proxied / ⚪ DNS Only)",
            ]),
            Row::new(vec![
                "i / Enter (DNS)",
                "Open Zone Inspector (SSL, Dev Mode, Cache Purge)",
            ]),
            Row::new(vec![
                "Shift+E",
                "Export active zone to RFC 1035 BIND zone file",
            ]),
            Row::new(vec![
                "i (Workers)",
                "Inspect Worker Script (Routes, Bindings, Compat)",
            ]),
            Row::new(vec![
                "t / Enter (Workers)",
                "Attach Live Tail Log Console via WebSocket",
            ]),
            Row::new(vec![
                "Enter / d (Pages)",
                "Open Pages Deployments History Drawer",
            ]),
            Row::new(vec![
                "r (Deployments)",
                "Instant 1-keystroke Production Rollback",
            ]),
            Row::new(vec![
                "l (Deployments)",
                "Inspect Historical Build Execution Logs",
            ]),
            Row::new(vec![
                "Enter / i (Storage)",
                "Open D1 SQL Console or Workers KV Key-Value Browser",
            ]),
            Row::new(vec![
                "Space (Live Tail)",
                "Pause / Resume Live WebSocket Log Stream",
            ]),
            Row::new(vec![
                "f (Live Tail)",
                "Toggle Auto-Follow / Manual History Scroll",
            ]),
            Row::new(vec!["c (Live Tail)", "Clear Live Tail Log Buffer"]),
            Row::new(vec![
                "Enter (Live Tail)",
                "Inspect Full Log Payload & Error Traces",
            ]),
            Row::new(vec![
                "Enter / i (Security)",
                "Open Security Event Inspector (IP, Ray ID, Raw JSON)",
            ]),
            Row::new(vec![
                "t (Security)",
                "Cycle sub-tabs (Security Events, WAF Rules, Analytics)",
            ]),
            Row::new(vec![
                "f (Security)",
                "Cycle threat action filter (Block, Challenge, Log)",
            ]),
            Row::new(vec![
                "p (Security)",
                "Toggle selected WAF Rule active / paused",
            ]),
            Row::new(vec![
                "+ / - (Security)",
                "Shift selected WAF Rule execution priority",
            ]),
            Row::new(vec![
                "n / c (Domains)",
                "Open Domain Registration & Zone Onboarding modal",
            ]),
            Row::new(vec!["a (Domains)", "Toggle Domain Auto-Renew on / off"]),
            Row::new(vec![
                "l (Domains)",
                "Toggle Domain Transfer Lock protection",
            ]),
            Row::new(vec!["p (Domains)", "Toggle WHOIS Privacy Protection"]),
        ];

        let table = Table::new(rows, [Constraint::Length(25), Constraint::Min(45)])
            .header(
                Row::new(vec!["Keybinding", "Action"])
                    .style(theme.header_style())
                    .bottom_margin(1),
            )
            .block(block);

        f.render_widget(table, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_help_modal_rendering() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                let area = f.area();
                HelpModal::render(f, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_help_modal_compact_rendering() {
        let backend = TestBackend::new(70, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                let area = f.area();
                HelpModal::render(f, area, &theme);
            })
            .unwrap();
    }
}
