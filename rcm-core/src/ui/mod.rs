//! Framework-agnostic menu layout.
//!
//! This module holds everything about *where* and *how* an RCM context menu is
//! laid out, independent of the UI toolkit used to draw it:
//!
//! - [`geometry`] — screen-space points, sizes and rectangles in physical pixels.
//! - [`metrics`] — the few layout numbers every frontend must agree on.
//! - [`level`] — flattens an [`rcm_core::Menu`] subtree into renderable rows.
//! - [`position`] — the clamp/flip position algorithm shared by every frontend.
//! - [`state`] — open-window registry, deepest depth, auto-hide and blur debounce.
//! - [`host`] — the [`MenuHost`] trait a toolkit implements (create/move/resize/
//!   measure/show/hide/focus windows).
//! - [`controller`] — [`MenuController`], the shared show/hover/execute/hide state
//!   machine driven through a [`MenuHost`].
//! - [`blocking`] — the cached native context-menu blocking flag.
//!
//! The reference behaviour is the Tauri build: `rcm-tauri`'s `layout.rs` +
//! `rcm-ui/utils/layout.ts`. The algorithms here are direct ports of those, so
//! the Reactor build and the Tauri build place menus identically.
//!
//! Nothing here describes how a menu *looks*. A level's window size reaches the
//! controller as a [`Measurement`] from the host, and the rows reach it as a
//! [`MenuLevel`] — both without any notion of row heights or padding. Each
//! frontend owns its own presentation values.

pub mod blocking;
pub mod controller;
pub mod geometry;
pub mod host;
pub mod level;
pub mod metrics;
pub mod position;
pub mod state;

pub use blocking::{disable_blocking, enable_blocking, is_blocking_enabled};
pub use controller::{HoverInfo, HoverResult, MenuController, MenuShowRequest};
pub use geometry::{Point, Rect, Size};
pub use host::{Measurement, MenuHost, MenuWindowInput};
pub use level::{FlattenOptions, MenuLevel, MenuRow};
pub use metrics::MenuMetrics;
pub use state::MenuState;
