// ═══════════════════════════════════════════════════════════════════════════
// Events module — shared types, constants, payloads, and window helpers.
// Everything that passes between Rust ↔ Frontend lives here.
//
// Contract:
//   Rust → FE : menu-show (what to render), menu-hide-all, dev-mode,
//               icons-changed, theme-changed, style-changed
//   FE   → Rust: menu-hover (which row), menu-measured (how big it drew),
//               menu-execute, menu-close-all, log-event
//
// Dismissal is *not* event-driven: Rust polls which window holds focus, so no
// blur event is sent. A blur cannot distinguish "focus moved to another menu
// window" from "focus left the menu", and it arrives after our bookkeeping has
// changed.
//
// Rust never tells the frontend a *position*, and the frontend never tells Rust
// one except as a measurement. See `rcm_core::ui` for the placement algorithm.
// ═══════════════════════════════════════════════════════════════════════════

use rcm_core::Menu;
use rcm_core::ui::{MenuWindowInput, Point, Size};
use serde::{Deserialize, Serialize};
use tauri::PhysicalPosition;

// ═══════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════
//
// Behavioural layout constants live in `rcm_core::ui::MenuMetrics` so the Tauri
// and Reactor builds cannot drift apart. Only Tauri-specific values (the window
// labels and the off-screen park position) live here.

/// Label of the root menu window.
pub const ROOT_LABEL: &str = "main";

/// Off-screen position for hidden windows.
pub const OFF_SCREEN: PhysicalPosition<f64> = PhysicalPosition {
    x: -9999.0,
    y: -9999.0,
};

// ═══════════════════════════════════════════════════════════════════════════
// Config payload (for frontend)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct ConfigPayload {
    pub dev: bool,
    pub icons: bool,
    pub theme: String,
    pub js_url: Option<String>,
    pub css_url: Option<String>,
    pub config_url: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Event payloads — Rust → Frontend
// ═══════════════════════════════════════════════════════════════════════════

/// Render this menu level.
///
/// Carries **no geometry**: the frontend draws the level, measures it, and
/// reports the size back. Rust then positions the window.
#[derive(Debug, Clone, Serialize)]
pub struct MenuShowPayload {
    /// Full menu data — every window gets the complete tree.
    pub menu: Menu,
    /// Index path to render. Empty `[]` = root.
    pub path: Vec<i32>,
}

/// Build the `menu-show` payload for a level.
///
/// The level's advisory position is dropped: the window is positioned by Rust
/// after the frontend reports its measurement.
pub fn menu_show_payload(input: &MenuWindowInput) -> MenuShowPayload {
    MenuShowPayload {
        menu: (*input.menu).clone(),
        path: input.path().to_vec(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Event payloads — Frontend → Rust
// ═══════════════════════════════════════════════════════════════════════════

/// Frontend reports which row the pointer entered.
///
/// This is the *only* input the placement decisions need: the controller
/// already knows each level's rectangle and row metrics.
#[derive(Debug, Clone, Deserialize)]
pub struct MenuHoverPayload {
    /// Depth of the emitting window (0 = root).
    pub depth: usize,
    /// Index path to the hovered item.
    pub path: Vec<i32>,
    /// Row index within the level.
    #[serde(default)]
    pub index: usize,
    /// Measured offset of the row from the content top, in physical pixels.
    #[serde(default, rename = "itemY")]
    pub item_y: Option<i32>,
}

impl MenuHoverPayload {
    /// Convert to the framework-agnostic hover report.
    pub fn to_info(&self) -> rcm_core::ui::HoverInfo {
        rcm_core::ui::HoverInfo {
            depth: self.depth,
            path: self.path.clone(),
            index: self.index,
            item_y: self.item_y,
        }
    }
}

/// Frontend reports how large it drew a level, in **physical pixels**.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct MenuMeasuredPayload {
    /// Depth of the emitting window (0 = root).
    pub depth: usize,
    /// Full window size, including any CSS padding around `.rcm-root`.
    #[serde(rename = "winW")]
    pub win_w: i32,
    #[serde(rename = "winH")]
    pub win_h: i32,
    /// Size of the `.rcm-root` content itself (used for edge flipping).
    #[serde(rename = "rootW")]
    pub root_w: i32,
    #[serde(rename = "rootH")]
    pub root_h: i32,
    /// Offset of `.rcm-root` inside the window (the container's CSS padding).
    #[serde(rename = "rootOffsetX", default)]
    pub root_offset_x: i32,
    #[serde(rename = "rootOffsetY", default)]
    pub root_offset_y: i32,
}

impl MenuMeasuredPayload {
    /// Convert to the framework-agnostic measurement.
    pub fn to_measurement(&self) -> rcm_core::ui::Measurement {
        rcm_core::ui::Measurement {
            window: Size::new(self.win_w, self.win_h),
            content: Size::new(self.root_w, self.root_h),
            content_offset: Point::new(self.root_offset_x, self.root_offset_y),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuExecutePayload {
    /// Index path to the clicked item.
    pub path: Vec<i32>,
    /// Command to execute (sent directly from frontend).
    pub command: rcm_core::CommandPayload,
}

// ═══════════════════════════════════════════════════════════════════════════
// Window label helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Label for a submenu pool slot (`0 -> "submenu-0"`).
pub fn submenu_label(index: usize) -> String {
    format!("submenu-{index}")
}

/// The window label a given menu depth lives in.
///
/// Depth 0 is the root window; deeper levels are `submenu-0`, `submenu-1`, …
pub fn label_for_depth(depth: usize) -> &'static str {
    if depth == 0 {
        ROOT_LABEL
    } else {
        crate::menu_host::intern_label(&submenu_label(depth - 1)).unwrap_or(ROOT_LABEL)
    }
}
