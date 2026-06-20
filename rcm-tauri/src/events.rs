// ═══════════════════════════════════════════════════════════════════════════
// Events module — shared types, constants, payloads, and window helpers.
// Everything that passes between Rust ↔ Frontend lives here.
// ═══════════════════════════════════════════════════════════════════════════

use rcm_core::{CommandPayload, Menu};
use serde::{Deserialize, Serialize};
use std::ops::{Range, RangeInclusive};
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::{Arc, Mutex};
use tauri::PhysicalPosition;

// ═══════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════

/// Maximum submenu depth (0 = root, 1-3 = submenus).
pub const MAX_SUBMENU_DEPTH: usize = 4;

/// Off-screen position for hidden windows.
pub const OFF_SCREEN: PhysicalPosition<f64> = PhysicalPosition {
    x: -9999.0,
    y: -9999.0,
};

/// Tracks the deepest menu depth currently visible.
/// Used to decide whether a blur event should hide all menus
/// (only if the deepest window lost focus).
pub static DEEPEST_DEPTH: AtomicUsize = AtomicUsize::new(0);

/// Submenu horizontal gap from parent window edge (physical px).
pub const SUBMENU_GAP: f64 = 8.0;

/// Auto-hide all menu windows after this many milliseconds of inactivity.
pub const AUTO_HIDE_MS: u64 = 30_000;

// ═══════════════════════════════════════════════════════════════════════════
// Shared state type aliases
// ═══════════════════════════════════════════════════════════════════════════

/// Holds the last built menu so hover/click handlers can navigate it.
pub type MenuArc = Arc<Mutex<Option<Menu>>>;

/// Epoch counter for the global auto-hide timer.
/// Incremented on every user interaction; the timer task checks this
/// before hiding all windows.
pub type AutoHideEpoch = Arc<AtomicU64>;

// ═══════════════════════════════════════════════════════════════════════════
// Config payload (for frontend)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct ConfigPayload {
    pub dev: bool,
    pub icons: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Event payloads — Rust → Frontend
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct MenuShowPayload {
    /// Full menu data — every window gets the complete tree.
    pub menu: Menu,
    /// Index path to render. Empty `[]` = root.
    pub path: Vec<i32>,
    /// Ideal screen position (frontend will clamp after measuring DOM).
    pub x: f64,
    pub y: f64,
    /// Parent window info for submenu flip logic (None for root).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_w: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Event payloads — Frontend → Rust
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MenuHoverPayload {
    /// Depth of the emitting window (0 = root).
    pub depth: usize,
    /// Index path to the hovered item.
    pub path: Vec<i32>,
    /// Parent window's absolute screen position.
    #[serde(rename = "parentX")]
    pub parent_x: f64,
    #[serde(rename = "parentY")]
    pub parent_y: f64,
    /// Parent window's size.
    #[serde(rename = "parentW")]
    pub parent_w: f64,
    #[serde(rename = "parentH")]
    pub parent_h: f64,
    /// Hovered item's position relative to the parent window.
    #[serde(rename = "itemX")]
    pub item_x: f64,
    #[serde(rename = "itemY")]
    pub item_y: f64,
    /// Hovered item's size.
    #[serde(rename = "itemW")]
    pub item_w: f64,
    #[serde(rename = "itemH")]
    pub item_h: f64,
    /// Absolute screen X of the parent window's content right edge (no shadow).
    #[serde(default)]
    pub content_right: f64,
    /// Height of the parent's .rcm-root content element (for boundary clamping).
    #[serde(rename = "parentContentHeight", default)]
    pub parent_content_h: f64,
    /// Width of the parent's .rcm-root content element (for precise X alignment).
    #[serde(rename = "parentContentWidth", default)]
    pub parent_content_w: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MenuHoverOutPayload {
    /// Depth of the emitting window.
    pub depth: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MenuExecutePayload {
    /// Index path to the clicked item.
    pub path: Vec<i32>,
    /// Command to execute (sent directly from frontend).
    pub command: CommandPayload,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuBlurPayload {
    /// Depth of the window that lost focus.
    pub depth: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// Window label helpers
// ═══════════════════════════════════════════════════════════════════════════

pub fn root_label() -> &'static str {
    "main"
}

pub fn submenu_label(depth: usize) -> String {
    format!("submenu-{}", depth)
}

/// depth 0 → "main", depth 1 → "submenu-0", depth 2 → "submenu-1", …
/// Indices used by submenu window labels: 0 -> "submenu-0", etc.
pub fn submenu_indices() -> Range<usize> {
    0..MAX_SUBMENU_DEPTH
}

/// Visible menu depths owned by submenu windows: 1 -> "submenu-0", etc.
pub fn submenu_window_depths() -> RangeInclusive<usize> {
    1..=MAX_SUBMENU_DEPTH
}

pub fn window_label(depth: usize) -> String {
    if depth == 0 {
        root_label().to_string()
    } else {
        submenu_label(depth - 1)
    }
}
