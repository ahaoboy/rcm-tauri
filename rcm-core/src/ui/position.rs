//! Menu position computation — monitor selection, flip, and clamping.
//!
//! Direct port of `rcm-ui/utils/layout.ts` (`chooseMonitorForPoint` +
//! `computeWindowPosition`), which is what the Tauri build runs in its WebView.
//! Keeping it here means the Reactor build places menus *identically* instead
//! of re-deriving the edge cases.
//!
//! # The algorithm
//!
//! 1. Pick the monitor containing the ideal point (or the nearest one).
//! 2. Start at the ideal position, offset by how far the menu content sits
//!    inside its window (the window padding around `.rcm-root`).
//! 3. If the window would overflow the monitor's right edge, **flip**:
//!    - a submenu flips to the left of its parent,
//!    - a root menu flips so its right edge sits at the cursor.
//! 4. Clamp both axes into the monitor, keeping [`MenuMetrics::edge_gap`]
//!    away from every edge.
//!
//! All values are physical pixels.

use super::geometry::{Point, Rect};
use super::metrics::MenuMetrics;

/// Geometry needed to place one menu window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionInfo {
    /// Ideal left of the menu content on screen (the cursor, for the root).
    pub ideal_x: i32,
    /// Ideal top of the menu content on screen.
    pub ideal_y: i32,
    /// Left edge of the parent menu's content, for submenu flipping.
    /// `None` for the root menu.
    pub parent_root_x: Option<i32>,
    /// Window width, including padding around the content.
    pub win_w: i32,
    /// Window height, including padding around the content.
    pub win_h: i32,
    /// Distance from the window's left edge to the content's left edge.
    pub root_offset_x: i32,
    /// Distance from the window's top edge to the content's top edge.
    pub root_offset_y: i32,
    /// Width of the menu content itself.
    pub root_w: i32,
}

/// Find the monitor containing `point`, falling back to the nearest one.
///
/// Returns `None` only when `areas` is empty.
pub fn choose_monitor_for_point(areas: &[Rect], point: Point) -> Option<Rect> {
    if areas.is_empty() {
        return None;
    }

    for area in areas {
        if area.contains(point) {
            return Some(*area);
        }
    }

    areas
        .iter()
        .copied()
        .min_by_key(|area| area.distance_sq(point))
}

/// Compute the final window position for a menu, in physical pixels.
pub fn compute_window_position(
    info: &PositionInfo,
    metrics: &MenuMetrics,
    areas: &[Rect],
) -> Point {
    let monitor = choose_monitor_for_point(areas, Point::new(info.ideal_x, info.ideal_y));

    // Normal case: the window sits so its content lands on the ideal point.
    let mut win_x = info.ideal_x - info.root_offset_x;
    let mut win_y = info.ideal_y - info.root_offset_y;

    if let Some(area) = monitor {
        // Flip when the window would overrun the right edge of the monitor.
        if win_x + info.win_w > area.right() - metrics.edge_gap {
            win_x = match info.parent_root_x {
                // Submenu: put the content's right edge just left of the parent.
                Some(parent_root_x) => {
                    parent_root_x - metrics.submenu_gap - info.root_w - info.root_offset_x
                }
                // Root: put the content's right edge at the cursor.
                None => info.ideal_x - info.root_w - info.root_offset_x,
            };
        }
    }

    if let Some(area) = monitor {
        let bounds = area.inset(metrics.edge_gap);
        win_x = clamp_axis(win_x, bounds.left(), bounds.right(), info.win_w);
        win_y = clamp_axis(win_y, bounds.top(), bounds.bottom(), info.win_h);
    }

    Point::new(win_x, win_y)
}

/// Clamp `value` into `[min, max - extent]`, never producing a value below `min`.
///
/// Mirrors the TypeScript `Math.max(min, Math.min(value, Math.max(min, max - extent)))`,
/// which keeps the window visible even when it is larger than the monitor.
fn clamp_axis(value: i32, min: i32, max: i32, extent: i32) -> i32 {
    value.min((max - extent).max(min)).max(min)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1920x1080 monitor at the origin.
    fn monitor() -> Rect {
        Rect::new(0, 0, 1920, 1080)
    }

    fn metrics() -> MenuMetrics {
        MenuMetrics::default()
    }

    /// Root menu with no window padding, for readable arithmetic.
    fn root_info(ideal_x: i32, ideal_y: i32, w: i32, h: i32) -> PositionInfo {
        PositionInfo {
            ideal_x,
            ideal_y,
            parent_root_x: None,
            win_w: w,
            win_h: h,
            root_offset_x: 0,
            root_offset_y: 0,
            root_w: w,
        }
    }

    #[test]
    fn chooses_containing_monitor() {
        let areas = [Rect::new(0, 0, 1920, 1080), Rect::new(1920, 0, 1920, 1080)];
        assert_eq!(
            choose_monitor_for_point(&areas, Point::new(2000, 100)),
            Some(areas[1])
        );
    }

    #[test]
    fn chooses_nearest_monitor_when_outside_all() {
        let areas = [Rect::new(0, 0, 1920, 1080), Rect::new(3000, 0, 1920, 1080)];
        assert_eq!(
            choose_monitor_for_point(&areas, Point::new(2600, 100)),
            Some(areas[1])
        );
        assert_eq!(
            choose_monitor_for_point(&areas, Point::new(1900, 100)),
            Some(areas[0])
        );
    }

    #[test]
    fn no_monitors_returns_none() {
        assert_eq!(choose_monitor_for_point(&[], Point::new(0, 0)), None);
    }

    #[test]
    fn keeps_ideal_position_when_it_fits() {
        let pos = compute_window_position(&root_info(100, 100, 200, 300), &metrics(), &[monitor()]);
        assert_eq!(pos, Point::new(100, 100));
    }

    #[test]
    fn root_flips_left_at_right_edge() {
        let pos =
            compute_window_position(&root_info(1900, 100, 200, 300), &metrics(), &[monitor()]);
        // Content right edge lands on the cursor.
        assert_eq!(pos.x, 1900 - 200);
        assert_eq!(pos.y, 100);
    }

    #[test]
    fn submenu_flips_left_of_parent_at_right_edge() {
        let info = PositionInfo {
            parent_root_x: Some(1800),
            ..root_info(1870, 100, 200, 300)
        };
        let pos = compute_window_position(&info, &metrics(), &[monitor()]);
        let m = metrics();
        assert_eq!(pos.x, 1800 - m.submenu_gap - 200);
    }

    #[test]
    fn clamps_bottom_and_right_edges() {
        let pos =
            compute_window_position(&root_info(100, 1000, 200, 300), &metrics(), &[monitor()]);
        let m = metrics();
        assert_eq!(pos.y, 1080 - m.edge_gap - 300);
    }

    #[test]
    fn clamps_to_top_left_with_edge_gap() {
        // Ideal negative: the menu must stay `edge_gap` inside the monitor.
        let pos =
            compute_window_position(&root_info(-500, -500, 200, 300), &metrics(), &[monitor()]);
        let m = metrics();
        assert_eq!(pos.x, m.edge_gap);
        assert_eq!(pos.y, m.edge_gap);
    }

    #[test]
    fn oversized_menu_still_reaches_the_top_left() {
        // Taller than the monitor: clamping must not push it below the top.
        let pos =
            compute_window_position(&root_info(100, 500, 200, 2000), &metrics(), &[monitor()]);
        assert_eq!(pos.y, metrics().edge_gap);
    }

    #[test]
    fn honors_window_padding_offset() {
        // A 6px padding means the window starts 6px left of the content.
        let info = PositionInfo {
            root_offset_x: 6,
            root_offset_y: 4,
            ..root_info(100, 100, 200, 300)
        };
        let pos = compute_window_position(&info, &metrics(), &[monitor()]);
        assert_eq!(pos, Point::new(94, 96));
    }

    #[test]
    fn no_monitors_leaves_position_unclamped() {
        let pos = compute_window_position(&root_info(5000, 5000, 200, 300), &metrics(), &[]);
        assert_eq!(pos, Point::new(5000, 5000));
    }
}
