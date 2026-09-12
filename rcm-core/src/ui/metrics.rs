//! Layout metrics — the handful of numbers every frontend must agree on.
//!
//! Keep this module small. It exists so the Tauri and Reactor builds place
//! menus in *identically* the same spot, which only works if the values that
//! feed the position maths are shared. Everything else is somebody else's job:
//!
//! - **Here**: the gap between a parent menu and its submenu, the margin kept
//!   from the monitor edges, the auto-hide timeout, and the nesting limit.
//! - **The frontend**: how a row *looks*. Row heights, paddings, icon gutters,
//!   corner radii and window width clamps are rendering decisions, so each
//!   toolkit declares its own — Reactor in `rcm-reactor/src/metrics.rs`, the
//!   WebView build in `rcm-ui/style.css`.
//!
//! The controller never asks how tall a row is. It learns a level's window
//! rectangle from the host's [`Measurement`](super::host::Measurement) and works
//! from that, which is what keeps this crate free of any toolkit's look.

/// Tunable layout values, in physical pixels unless noted.
///
/// Every field here is *behavioural*: it decides where a menu goes, not what it
/// looks like. See the module docs for why that distinction is enforced.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuMetrics {
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
}

impl Default for MenuMetrics {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl MenuMetrics {
    /// The shared values.
    ///
    /// Frontends use these as-is; there is nothing to override, because nothing
    /// here is presentational.
    pub const DEFAULT: Self = Self {
        submenu_gap: 8,
        edge_gap: 8,
        auto_hide_ms: 30_000,
        max_submenu_depth: 4,
    };
}
