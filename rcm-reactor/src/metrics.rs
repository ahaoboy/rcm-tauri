//! How the Reactor build draws a menu level.
//!
//! `rcm_core::ui` owns the *behavioural* geometry — the submenu gap, the edge
//! margin, the auto-hide timeout, the nesting limit — because those decide where
//! a menu lands and must match the Tauri build exactly. It deliberately has no
//! opinion on how a row *looks*, and no idea how tall one is.
//!
//! That knowledge lives here. Reactor renders rows itself at fixed heights, so
//! these are the real dimensions rather than estimates: a WinUI row is a flat
//! 28px, not a CSS box the webview has to measure. Two things follow from that,
//! and they are why this module is more than a table of numbers:
//!
//! - [`level_height`] predicts a level's height before it is rendered, which is
//!   what gives a popup a sensible size on its very first frame;
//! - [`row_offset`] places a hover precisely, which is what the controller uses
//!   to line a submenu up with the row that opened it.

use rcm_core::ui::MenuLevel;

/// The dimensions Reactor draws its menus with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuStyle {
    /// Width assumed for a level before it has been measured.
    pub fallback_width: f64,
    /// Lower bound applied to the rendered menu width.
    pub min_width: f64,
    /// Upper bound applied to the rendered menu width.
    pub max_width: f64,
    /// Height of a single menu row.
    pub row_height: f64,
    /// Vertical space taken by a group separator.
    pub separator_height: f64,
    /// Height of the icon ribbon at the top of the root menu.
    pub ribbon_height: f64,
    /// Padding inside the popup border, on all four edges.
    pub padding: f64,
    /// Left/right padding inside a single row.
    pub row_padding: (f64, f64),
    /// Width of the icon gutter reserved in a level that shows icons.
    pub icon_width: f64,
    /// Width reserved for a row's submenu arrow and its surrounding padding.
    pub arrow_gutter: f64,
    /// Corner radius of the popup border.
    pub corner_radius: f64,
    /// Corner radius of a highlighted row.
    pub row_corner_radius: f64,
}

/// The style Reactor renders menus with.
pub const MENU_STYLE: MenuStyle = MenuStyle {
    fallback_width: 252.0,
    min_width: 168.0,
    max_width: 480.0,
    row_height: 28.0,
    separator_height: 7.0,
    ribbon_height: 36.0,
    padding: 4.0,
    row_padding: (4.0, 6.0),
    icon_width: 16.0,
    arrow_gutter: 40.0,
    corner_radius: 6.0,
    row_corner_radius: 4.0,
};

impl MenuStyle {
    /// Height contributed by one row of the given kind.
    fn row_kind_height(&self, separator: bool) -> f64 {
        if separator {
            self.separator_height
        } else {
            self.row_height
        }
    }

    /// Height taken by the ribbon plus its separator (0 when hidden).
    fn ribbon_offset(&self, level: &MenuLevel) -> f64 {
        if level.ribbon_visible {
            self.ribbon_height + self.separator_height
        } else {
            0.0
        }
    }
}

/// Total height of `level` when drawn with `style`, including popup padding.
///
/// Used to size a popup before its content exists, so the first frame is not a
/// zero-height window.
pub fn level_height(level: &MenuLevel, style: &MenuStyle) -> f64 {
    let rows: f64 = level
        .rows
        .iter()
        .map(|row| style.row_kind_height(row.is_separator()))
        .sum();
    style.padding * 2.0 + style.ribbon_offset(level) + rows
}

/// Y offset of row `index` from the top of the popup, in DIPs.
///
/// This is the fallback the controller uses when a hover arrives without a
/// measured offset, so it must agree with what [`crate::menu_window`] renders.
pub fn row_offset(level: &MenuLevel, index: usize, style: &MenuStyle) -> f64 {
    let mut offset = style.padding + style.ribbon_offset(level);
    for row in level.rows.iter().take(index) {
        offset += style.row_kind_height(row.is_separator());
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcm_core::ui::{FlattenOptions, MenuRow};
    use rcm_core::{Item, Menu};

    fn item(key: &str) -> Item {
        Item {
            key: key.to_string(),
            icon: String::new(),
            label: key.to_string(),
            disable: false,
            admin: false,
            window: Default::default(),
            items: Vec::new(),
            command: None,
        }
    }

    fn menu_with_groups(groups: Vec<Vec<Item>>) -> Menu {
        Menu {
            icon_items: Vec::new(),
            groups: groups
                .into_iter()
                .map(|items| Item {
                    items,
                    ..item("group")
                })
                .collect(),
        }
    }

    #[test]
    fn height_and_offsets_follow_the_style() {
        let menu = menu_with_groups(vec![vec![item("a"), item("b")], vec![item("c")]]);
        let level = MenuLevel::flatten(&menu, &[], FlattenOptions::new(false));
        let s = &MENU_STYLE;

        // Rows are: [a, b, SEP, c] — 3 items and 1 separator, plus padding.
        assert_eq!(level.rows.len(), 4);
        let expected = s.padding * 2.0 + s.row_height * 3.0 + s.separator_height;
        assert!((level_height(&level, s) - expected).abs() < f64::EPSILON);

        // First row sits just below the top padding.
        assert!((row_offset(&level, 0, s) - s.padding).abs() < f64::EPSILON);
        // Second row is one row lower.
        assert!((row_offset(&level, 1, s) - (s.padding + s.row_height)).abs() < f64::EPSILON);
        // Third row is the separator, right after two rows.
        let expected_sep = s.padding + s.row_height * 2.0;
        assert!((row_offset(&level, 2, s) - expected_sep).abs() < f64::EPSILON);
        // Fourth row is past the separator.
        let expected_last = expected_sep + s.separator_height;
        assert!((row_offset(&level, 3, s) - expected_last).abs() < f64::EPSILON);
    }

    #[test]
    fn ribbon_shifts_rows_down() {
        let mut menu = menu_with_groups(vec![vec![item("a")]]);
        menu.icon_items = vec![item("ribbon")];
        let level = MenuLevel::flatten(&menu, &[], FlattenOptions::new(true));
        let s = &MENU_STYLE;

        assert!(level.ribbon_visible);
        let expected = s.padding + s.ribbon_height + s.separator_height;
        assert!((row_offset(&level, 0, s) - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn separators_take_separator_height_not_row_height() {
        let menu = menu_with_groups(vec![vec![item("a")], vec![item("b")]]);
        let level = MenuLevel::flatten(&menu, &[], FlattenOptions::new(false));
        let s = &MENU_STYLE;

        assert_eq!(level.rows.len(), 3, "[a, SEP, b]");
        assert!(matches!(level.rows[1], MenuRow::Separator));
        let expected = s.padding + s.row_height + s.separator_height;
        assert!((row_offset(&level, 2, s) - expected).abs() < f64::EPSILON);
    }
}
