pub mod auth_modal;
pub mod confirm_modal;
pub mod dns_modal;
pub mod header;
pub mod help;
pub mod pages_deployments;
pub mod pages_logs_modal;
pub mod pages_rollback_modal;
pub mod status_bar;
pub mod views;
pub mod zone_inspector;
pub mod worker_inspector;
pub mod tail_log_modal;
pub mod d1_console_modal;
pub mod kv_modal;
pub mod security_inspector;
pub mod command_palette_modal;
pub mod profile_modal;
pub mod domain_create_modal;

use ratatui::layout::Rect;

/// Calculates a centered popup `Rect` responsive to window `area`.
/// Scales dynamically using percentages of window dimensions, clamped within min/max bounds,
/// and guarantees the popup never exceeds terminal window boundaries.
pub fn centered_rect(
    area: Rect,
    percent_x: u16,
    min_w: u16,
    max_w: u16,
    percent_y: u16,
    min_h: u16,
    max_h: u16,
) -> Rect {
    let target_w = (area.width * percent_x / 100).clamp(min_w, max_w);
    let popup_w = target_w.min(area.width.saturating_sub(2)).max(20.min(area.width));

    let target_h = (area.height * percent_y / 100).clamp(min_h, max_h);
    let popup_h = target_h.min(area.height.saturating_sub(2)).max(10.min(area.height));

    Rect {
        x: (area.width.saturating_sub(popup_w)) / 2,
        y: (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect_large_window() {
        let area = Rect::new(0, 0, 160, 50);
        let popup = centered_rect(area, 75, 64, 100, 75, 16, 26);
        assert_eq!(popup.width, 100);
        assert_eq!(popup.height, 26);
        assert_eq!(popup.x, 30);
        assert_eq!(popup.y, 12);
    }

    #[test]
    fn test_centered_rect_standard_window() {
        let area = Rect::new(0, 0, 80, 24);
        let popup = centered_rect(area, 75, 64, 100, 75, 16, 26);
        assert_eq!(popup.width, 64);
        assert_eq!(popup.height, 18);
        assert_eq!(popup.x, 8);
        assert_eq!(popup.y, 3);
    }

    #[test]
    fn test_centered_rect_small_window_clamping() {
        let area = Rect::new(0, 0, 60, 18);
        let popup = centered_rect(area, 75, 64, 100, 75, 16, 26);
        assert!(popup.width <= area.width);
        assert!(popup.height <= area.height);
        assert_eq!(popup.width, 58);
        assert_eq!(popup.height, 16);
    }

    #[test]
    fn test_centered_rect_ultrawide_window() {
        let area = Rect::new(0, 0, 240, 80);
        let popup = centered_rect(area, 92, 76, 145, 88, 20, 48);
        assert_eq!(popup.width, 145);
        assert_eq!(popup.height, 48);
        assert_eq!(popup.x, (240 - 145) / 2);
        assert_eq!(popup.y, (80 - 48) / 2);
    }

    #[test]
    fn test_centered_rect_compact_pane_bounds() {
        let area = Rect::new(0, 0, 35, 14);
        let popup = centered_rect(area, 80, 60, 100, 80, 18, 30);
        assert!(popup.width <= area.width);
        assert!(popup.height <= area.height);
        assert_eq!(popup.width, 33);
        assert_eq!(popup.height, 12);
        assert_eq!(popup.x, 1);
        assert_eq!(popup.y, 1);
    }
}
