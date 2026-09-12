//! The [`MenuHost`] trait — the toolkit-specific half of the menu.
//!
//! Everything in [`super`] is toolkit-independent; this is the narrow surface a
//! UI framework must implement so [`MenuController`](super::MenuController) can
//! drive it.
//!
//! # Two-phase contract
//!
//! A menu level is shown in two steps, which keeps *all* layout decisions in
//! Rust while letting the frontend do what only it can (draw, and measure what
//! it drew):
//!
//! 1. [`MenuHost::open_window`] — create or reuse the window for a level and ask
//!    the frontend to render it. **No geometry is applied yet.** A frontend that
//!    renders from Rust-provided data emits its "render this level" event here.
//! 2. [`MenuHost::place_window`] — apply the final rectangle computed by
//!    [`compute_window_position`](super::compute_window_position), reveal the
//!    window and focus it.
//!
//! Between the two, the frontend measures its own content and reports a
//! [`Measurement`] back. That is the *only* geometry the frontend produces.
//!
//! | Concern | Owner |
//! | --- | --- |
//! | Which level is displayed, and where | `rcm_core::ui` |
//! | Monitor clamping / edge flipping | `rcm_core::ui` |
//! | Hover, blur, auto-hide policy | `rcm_core::ui` |
//! | Drawing the level | frontend |
//! | Measuring the drawn content | frontend |
//! | Moving/resizing/focusing a native window | frontend |
//!
//! No method may block on the UI thread.

use std::sync::Arc;

use super::geometry::{Point, Rect, Size};
use super::level::MenuLevel;
use crate::types::Menu;

/// What a frontend needs in order to build one menu window.
#[derive(Debug, Clone)]
pub struct MenuWindowInput {
    /// The complete menu tree.
    ///
    /// Frontends that re-derive their own rows from the tree (the Tauri WebView
    /// does) read it from here; frontends that render [`Self::level`] directly
    /// (Reactor) can ignore it.
    pub menu: Arc<Menu>,
    /// The level to render (rows, ribbon and alignment flags).
    pub level: MenuLevel,
    /// Ideal screen position of the menu *content*, in physical pixels.
    ///
    /// Advisory only — [`MenuHost::place_window`] receives the final, clamped
    /// rectangle. Frontends that position natively can ignore this.
    pub position: Point,
    /// Left edge of the parent menu's content, for submenu flipping.
    /// `None` for the root menu.
    pub parent_left: Option<i32>,
}

impl PartialEq for MenuWindowInput {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.menu, &other.menu)
            && self.level == other.level
            && self.position == other.position
            && self.parent_left == other.parent_left
    }
}

impl MenuWindowInput {
    /// Nesting depth of the level being shown.
    pub fn depth(&self) -> usize {
        self.level.depth
    }

    /// Index path of the level being shown.
    pub fn path(&self) -> &[i32] {
        &self.level.path
    }
}

/// A frontend's report of how large it actually drew a menu level.
///
/// Produced by the frontend (only it can measure its own render), consumed by
/// [`MenuController`](super::MenuController) to compute the final rectangle.
/// All values are **physical pixels**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measurement {
    /// Full window size, including any padding around the content.
    pub window: Size,
    /// Size of the menu content itself (the `.rcm-root` equivalent).
    pub content: Size,
    /// Offset from the window's top-left to the content's top-left.
    ///
    /// `(0, 0)` when the window is sized exactly to its content, which is what
    /// Reactor does; a CSS-padded container reports its padding here.
    pub content_offset: Point,
}

impl Measurement {
    /// A measurement whose window is exactly its content (no padding).
    pub const fn exact(content: Size) -> Self {
        Self {
            window: content,
            content,
            content_offset: Point::new(0, 0),
        }
    }

    /// Whether both sizes are usable.
    pub const fn is_valid(&self) -> bool {
        self.window.is_positive() && self.content.is_positive()
    }
}

/// The UI operations a menu needs from its host framework.
///
/// Every method takes `&self` or `&mut self` on the host, never on a shared
/// global, so an implementation may be a component, an app handle wrapper, or a
/// plain test double.
pub trait MenuHost {
    /// Cheap, copyable identity for one menu window.
    ///
    /// `Ord` is required so [`MenuState`](super::MenuState) can key windows in a
    /// deterministic map. Reactor uses raw `HWND`s; Tauri uses its window
    /// labels.
    type Window: Copy + Ord;

    /// Ensure the window for `input` exists and ask the frontend to render it.
    ///
    /// Must not apply geometry, show, or focus the window — that is
    /// [`Self::place_window`]'s job. Returns `None` if the host cannot service
    /// the request at all.
    fn open_window(&mut self, input: &MenuWindowInput) -> Option<Self::Window>;

    /// Apply `rect`, reveal and focus `window`.
    ///
    /// `rect` is the final, clamped screen rectangle in physical pixels.
    ///
    /// A host that owns its own window *size* (Reactor publishes
    /// `WindowVisuals::client_size` in DIPs and lets the framework convert with
    /// the real window DPI) should apply the position only — `SetWindowPos` sizes
    /// the outer window, not the client area, so applying `rect`'s size would
    /// fight that conversion by the border width. Such a host still needs `rect`
    /// for placement, which is why the size is available but not mandatory.
    ///
    /// Hosts that render from `input` and need the level data again on re-place
    /// may use it; Reactor ignores it.
    fn place_window(&mut self, window: Self::Window, input: &MenuWindowInput, rect: Rect);

    /// Hide `window`, keeping it alive for reuse.
    fn hide_window(&mut self, window: Self::Window);

    /// Destroy `window`.
    fn close_window(&mut self, window: Self::Window);

    /// Whether `window` currently holds focus.
    ///
    /// Used for the click-away dismiss. Defaults to `true` so a host that cannot
    /// answer never triggers a spurious dismissal.
    fn is_window_focused(&self, _window: Self::Window) -> bool {
        true
    }

    /// Monitor work areas in physical pixels, for clamping and flipping.
    fn work_areas(&self) -> Vec<Rect>;

    /// Physical pixels per DIP for `window`.
    fn scale_factor(&self, _window: Self::Window) -> f64 {
        1.0
    }
}
