//! Menu metrics — every tunable dimension the layout depends on.
//!
//! Two categories live here, and it is worth keeping them apart:
//!
//! 1. **Layout geometry** — gaps, edge margins, auto-hide timeout, nesting
//!    limit. These are *behavioural*: they come from the Tauri build's
//!    `rcm-ui/utils/layout.ts` and `rcm-tauri/src/events.rs`, and every
//!    frontend must use the same values to place menus identically.
//! 2. **Row presentation** — row/separator/ribbon heights and paddings. These
//!    are *visual*: a WinUI row and an HTML row are not the same height, so a
//!    toolkit is expected to override them.
//!
//! [`MenuMetrics::default`] uses the Tauri behavioural values with the Reactor
//! presentation values, since Reactor is the only frontend that needs them in
//! Rust. Use [`MenuMetrics::for_reactor`] for the labelled preset, or struct
//! update syntax to override individual fields:
//!
//! ```ignore
//! let metrics = MenuMetrics { row_height: 32.0, ..MenuMetrics::default() };
//! ```

/// Number of submenu windows a label-based frontend should pre-create.
///
/// The Tauri build pre-creates a small pool and reuses it, because creating a
/// webview is expensive. Depth `d` (1-based) uses `submenu-{d-1}`, so this must
/// be at least [`MenuMetrics::max_submenu_depth`].
pub const PRE_CREATED_WINDOWS: usize = 3;

/// Upper bound on the number of interned `submenu-N` labels.
///
/// Label-addressed frontends need `&'static str` identities; this is the array
/// size they can index into.
pub const MAX_PRE_CREATED_SUBMENUS: usize = 8;

/// Tunable menu dimensions, in physical pixels unless noted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuMetrics {
    // ── Behavioural geometry (must match across frontends) ──────────────
    /// Horizontal gap between a parent menu and its submenu.
    ///
    /// Tauri: `SUBMENU_GAP` in `rcm-ui/utils/layout.ts`.
    pub submenu_gap: i32,

    /// Minimum gap kept between a menu and the monitor edges.
    ///
    /// Tauri: `EDGE_GAP` in `rcm-ui/utils/layout.ts`.
    pub edge_gap: i32,

    /// Auto-hide every menu after this many milliseconds of inactivity.
    ///
    /// Tauri: `AUTO_HIDE_MS` in `rcm-tauri/src/events.rs`.
    pub auto_hide_ms: u64,

    /// Maximum submenu nesting depth (0 = root, 1..=4 = submenus).
    ///
    /// Tauri: `MAX_SUBMENU_DEPTH` in `rcm-tauri/src/events.rs`.
    pub max_submenu_depth: usize,

    // ── Row presentation (toolkit-specific, overridable) ────────────────
    /// Width assumed before the renderer has measured the content.
    pub fallback_width: f64,

    /// Lower bound applied to the measured menu width.
    pub min_width: f64,

    /// Upper bound applied to the measured menu width.
    pub max_width: f64,

    /// Height of a single menu row.
    pub row_height: f64,

    /// Vertical space taken by a group separator.
    pub separator_height: f64,

    /// Height of the icon ribbon at the top of the root menu.
    pub ribbon_height: f64,

    /// Padding inside the menu popup, on all four edges.
    pub padding: f64,

    /// Left/right padding inside a single menu row.
    pub row_padding: (f64, f64),

    /// Width of the icon gutter reserved in a row that shows an icon.
    pub icon_width: f64,

    /// Corner radius of the menu popup.
    pub corner_radius: f64,

    /// Corner radius of a highlighted row.
    pub row_corner_radius: f64,
}

impl Default for MenuMetrics {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl MenuMetrics {
    /// Explicit alias for the defaults, for call sites that want to read
    /// symmetrically with [`MenuMetrics::for_tauri`].
    pub const fn for_reactor() -> Self {
        Self::DEFAULT
    }

    /// The Tauri behavioural values, with the Tauri build's CSS row metrics.
    ///
    /// The Tauri frontend does its own measurement in the WebView, so these
    /// presentation values are only used when Rust-side height estimation is
    /// needed (e.g. computing a submenu's row offset).
    pub const fn for_tauri() -> Self {
        Self {
            submenu_gap: 8,
            edge_gap: 8,
            auto_hide_ms: 30_000,
            max_submenu_depth: 4,
            fallback_width: 260.0,
            min_width: 160.0,
            max_width: 520.0,
            row_height: 30.0,
            separator_height: 8.0,
            ribbon_height: 40.0,
            padding: 6.0,
            row_padding: (10.0, 8.0),
            icon_width: 20.0,
            corner_radius: 8.0,
            row_corner_radius: 4.0,
        }
    }

    /// The default metrics, as a constant.
    pub const DEFAULT: Self = Self {
        // Behavioural — Tauri reference values.
        submenu_gap: 8,
        edge_gap: 8,
        auto_hide_ms: 30_000,
        max_submenu_depth: 4,

        // Presentation — Reactor values.
        fallback_width: 252.0,
        min_width: 168.0,
        max_width: 480.0,
        row_height: 28.0,
        separator_height: 7.0,
        ribbon_height: 36.0,
        padding: 4.0,
        row_padding: (4.0, 6.0),
        icon_width: 16.0,
        corner_radius: 6.0,
        row_corner_radius: 4.0,
    };

    /// Height contributed by a single row of the given kind.
    pub fn row_kind_height(&self, separator: bool) -> f64 {
        if separator {
            self.separator_height
        } else {
            self.row_height
        }
    }
}
