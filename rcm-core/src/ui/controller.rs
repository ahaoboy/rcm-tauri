//! [`MenuController`] — the shared menu state machine.
//!
//! This is the single owner of two questions:
//!
//! 1. **Which level is displayed?** — depth limits, the index path, and the
//!    flattened rows for each open level.
//! 2. **Where is it displayed?** — the ideal position, the monitor clamp, and
//!    the right-edge flip.
//!
//! Frontends only draw what they are handed and measure the result; they never
//! compute a position. That is the whole point of this module: the placement
//! algorithm lives here once, so the Tauri and Reactor builds cannot drift.
//!
//! ## Show sequence
//!
//! ```text
//! controller.show_root(menu, at)  -> MenuShowRequest (which level)
//! controller.open(&request)       -> host draws it, no geometry yet
//!     ... frontend measures ...
//! controller.place(depth, m)      -> host applies the final rectangle
//! ```
//!
//! Reactor cannot let the host create windows (its framework only opens windows
//! from inside a component publication), so it uses
//! [`MenuController::show_root`] + [`MenuController::take_request`] and reports
//! the window back through [`MenuController::adopt`]. Everything after that is
//! identical.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::log;
use crate::types::{CommandPayload, IndexPath, Menu};

use super::geometry::{Point, Rect};
use super::host::{Measurement, MenuHost, MenuWindowInput};
use super::level::{FlattenOptions, MenuLevel};
use super::metrics::MenuMetrics;
use super::position::{PositionInfo, compute_window_position};
use super::state::MenuState;

/// What a frontend should display for one menu level.
#[derive(Debug, Clone)]
pub struct MenuShowRequest {
    /// The complete menu tree.
    pub menu: Arc<Menu>,
    /// Nesting depth (0 = root).
    pub depth: usize,
    /// Index path of the level.
    pub path: IndexPath,
    /// Rows to render.
    pub level: MenuLevel,
    /// Ideal position of the menu *content*, in physical pixels.
    pub position: Point,
    /// Left edge of the parent's content, for submenu flipping.
    pub parent_left: Option<i32>,
}

impl PartialEq for MenuShowRequest {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.menu, &other.menu)
            && self.depth == other.depth
            && self.path == other.path
            && self.level == other.level
            && self.position == other.position
            && self.parent_left == other.parent_left
    }
}

impl MenuShowRequest {
    /// Convert into the host input for opening the window.
    pub fn window_input(&self) -> MenuWindowInput {
        MenuWindowInput {
            menu: Arc::clone(&self.menu),
            level: self.level.clone(),
        }
    }

    /// Build the geometry for [`compute_window_position`].
    pub fn position_info(&self, measurement: Measurement) -> PositionInfo {
        PositionInfo {
            ideal_x: self.position.x,
            ideal_y: self.position.y,
            parent_root_x: self.parent_left,
            win_w: measurement.window.width,
            win_h: measurement.window.height,
            root_offset_x: measurement.content_offset.x,
            root_offset_y: measurement.content_offset.y,
            root_w: measurement.content.width,
        }
    }
}

/// Hover report from a frontend.
///
/// Deliberately tiny: the frontend says *which row* the pointer is on, and how
/// far down it sits. Every positioning decision follows from data the controller
/// already owns (the level's rows, the window's placed rectangle, the metrics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverInfo {
    /// Depth of the window the pointer is in (0 = root).
    pub depth: usize,
    /// Index path of the hovered item.
    pub path: IndexPath,
    /// Row index within the level, used for the fallback offset estimate.
    pub index: usize,
    /// Measured offset of the hovered row from the top of the level's content,
    /// in physical pixels.
    ///
    /// Frontends should always send this — they are the only ones that know how
    /// tall their rows are. When it is `None` the controller asks the host for an
    /// estimate instead, and failing that aligns the submenu with the top of the
    /// parent's content.
    pub item_y: Option<i32>,
}

/// Outcome of a hover, so the caller knows why nothing was shown.
#[derive(Debug, Clone, PartialEq)]
pub enum HoverResult {
    /// Show this level.
    Show(Box<MenuShowRequest>),
    /// The item is a leaf or disabled — deeper menus were closed.
    Leaf,
    /// An unknown path, or the nesting limit was reached — nothing changed.
    Ignored,
}

/// What the controller remembers about one open level.
#[derive(Debug, Clone)]
struct OpenLevel {
    request: MenuShowRequest,
    /// Rectangle the host was last told to use, if placement has happened.
    rect: Option<Rect>,
}

/// Owns the menu tree, the shared metrics, the open levels and the window
/// bookkeeping, and turns user interaction into placement decisions.
#[derive(Debug)]
pub struct MenuController<H: MenuHost> {
    host: H,
    metrics: MenuMetrics,
    state: MenuState<H::Window>,
    /// `depth -> open level`, including levels the host created but has not
    /// measured yet.
    levels: BTreeMap<usize, OpenLevel>,
    menu: Option<Arc<Menu>>,
    options: FlattenOptions,
    dev_mode: bool,
}

impl<H: MenuHost> MenuController<H> {
    /// Create a controller driving `host` with `metrics`.
    pub fn new(host: H, metrics: MenuMetrics) -> Self {
        Self {
            host,
            metrics,
            state: MenuState::new(),
            levels: BTreeMap::new(),
            menu: None,
            options: FlattenOptions::new(false),
            dev_mode: false,
        }
    }

    // ── Accessors ───────────────────────────────────────────────────────

    pub fn host(&self) -> &H {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut H {
        &mut self.host
    }

    pub fn metrics(&self) -> &MenuMetrics {
        &self.metrics
    }

    pub fn state(&self) -> &MenuState<H::Window> {
        &self.state
    }

    pub fn menu(&self) -> Option<&Menu> {
        self.menu.as_deref()
    }

    /// The open level at `depth`, if any.
    pub fn level(&self, depth: usize) -> Option<&MenuLevel> {
        self.levels.get(&depth).map(|open| &open.request.level)
    }

    /// The level to render at `depth`, if it is open.
    pub fn open_level(&self, depth: usize) -> Option<&MenuShowRequest> {
        self.levels.get(&depth).map(|open| &open.request)
    }

    /// The rectangle a level was last placed at.
    pub fn placed_rect(&self, depth: usize) -> Option<Rect> {
        self.levels.get(&depth).and_then(|open| open.rect)
    }

    /// The user's `icons` preference, as read from `rcm.config.json`.
    pub fn icons_enabled(&self) -> bool {
        self.options.icons_enabled
    }

    pub fn set_icons_enabled(&mut self, enabled: bool) {
        self.options = FlattenOptions::new(enabled);
    }

    /// Dev mode keeps the menu open after a command runs.
    pub fn dev_mode(&self) -> bool {
        self.dev_mode
    }

    pub fn set_dev_mode(&mut self, dev: bool) {
        self.dev_mode = dev;
    }

    /// Depth of the deepest visible menu.
    pub fn deepest(&self) -> usize {
        self.state.deepest()
    }

    /// Whether any menu level is open.
    pub fn has_levels(&self) -> bool {
        self.state.has_windows()
    }

    /// Monitor work areas, in physical pixels.
    pub fn work_areas(&self) -> Vec<Rect> {
        self.host.work_areas()
    }

    /// Flatten a level from the current menu without changing any state.
    pub fn flatten(&self, path: &[i32]) -> Option<MenuLevel> {
        let menu = self.menu.as_ref()?;
        Some(MenuLevel::flatten(menu, path, self.options))
    }

    // ── Showing ─────────────────────────────────────────────────────────

    /// Show a fresh root menu at `at`.
    ///
    /// Replaces any previously shown menu: deeper levels are closed and the new
    /// tree becomes the active one. The caller must then either
    /// [`Self::open`] the request (host creates the window) or
    /// [`Self::take_request`] it (frontend creates the window).
    pub fn show_root(&mut self, menu: Menu, at: Point) -> MenuShowRequest {
        log::info(
            "Menu::show_root",
            &format!(
                "pos=({}, {}) groups={} icons={} max_depth={}",
                at.x,
                at.y,
                menu.groups.len(),
                menu.icon_items.len(),
                menu.max_depth()
            ),
        );

        // A new right-click always replaces what is on screen — including a
        // previous *root* window. Reactor gives each menu its own window, so
        // closing only the deeper levels would leave the old root visible and
        // leak one window per right-click.
        self.hide_all();

        let menu = Arc::new(menu);
        let level = MenuLevel::flatten(&menu, &[], self.options);
        self.menu = Some(Arc::clone(&menu));
        self.state.touch();

        let request = MenuShowRequest {
            menu,
            depth: 0,
            path: Vec::new(),
            level,
            position: at,
            parent_left: None,
        };
        self.levels.insert(
            0,
            OpenLevel {
                request: request.clone(),
                rect: None,
            },
        );
        request
    }

    /// Handle the pointer entering a menu item.
    ///
    /// Leaf and disabled items close any deeper level; items with children
    /// produce a request for the next level, positioned from the parent's
    /// *placed* rectangle — not from anything the frontend computed.
    pub fn hover(&mut self, info: &HoverInfo) -> HoverResult {
        self.state.touch();

        let child_depth = info.depth + 1;
        log::event(
            "RECV",
            "menu-hover",
            &format!(
                "depth={} path={:?} index={} itemY={:?}",
                info.depth, info.path, info.index, info.item_y
            ),
        );

        // Decide first, so the immutable borrow of the menu ends before we
        // mutate any state.
        let decision = {
            let Some(menu) = self.menu.as_ref() else {
                log::warn("Menu::hover", "no menu data");
                return HoverResult::Ignored;
            };
            match menu.get_item(&info.path) {
                None => {
                    log::warn("Menu::hover", "item not found");
                    return HoverResult::Ignored;
                }
                Some(item) if item.disable || !item.has_children() => None,
                Some(_) if child_depth > self.metrics.max_submenu_depth => {
                    return HoverResult::Ignored;
                }
                Some(_) => Some(child_depth),
            }
        };

        let Some(child_depth) = decision else {
            // Leaf or disabled: just close anything deeper than this level.
            self.hide_deeper_than(info.depth);
            return HoverResult::Leaf;
        };

        // Position the child from the parent's *actual* rectangle, which only
        // this controller knows — the frontend never reports positions.
        let Some((parent_rect, parent_item_y)) = self.parent_geometry(info) else {
            log::warn("Menu::hover", "parent level has no rectangle yet");
            return HoverResult::Ignored;
        };

        // Close anything deeper than the level we are about to open.
        self.hide_deeper_than(child_depth);

        let Some(level) = self.flatten(&info.path) else {
            return HoverResult::Ignored;
        };
        let Some(menu) = self.menu.as_ref().map(Arc::clone) else {
            return HoverResult::Ignored;
        };

        // Ideal position: to the right of the parent content, aligned with the
        // hovered row.
        let ideal = Point::new(
            parent_rect.x + parent_rect.width + self.metrics.submenu_gap,
            parent_rect.y + parent_item_y,
        );

        let request = MenuShowRequest {
            menu,
            depth: child_depth,
            path: info.path.clone(),
            level,
            position: ideal,
            parent_left: Some(parent_rect.x),
        };

        self.levels.insert(
            child_depth,
            OpenLevel {
                request: request.clone(),
                rect: None,
            },
        );

        HoverResult::Show(Box::new(request))
    }

    /// Parent content rectangle and the hovered row's offset from its top.
    ///
    /// The offset comes from the frontend's measurement when it has one; failing
    /// that the host is asked for an estimate (in DIPs, so scaled by the parent
    /// window's DPI to match `rect`'s physical pixels); failing that the child
    /// lines up with the top of the parent's content.
    fn parent_geometry(&self, info: &HoverInfo) -> Option<(Rect, i32)> {
        let open = self.levels.get(&info.depth)?;
        let rect = open.rect?;

        let item_y = match info.item_y {
            Some(y) => y,
            None => self
                .host
                .row_offset(&open.request.level, info.index)
                .map(|dip| {
                    let scale = self
                        .state
                        .window(info.depth)
                        .map(|window| self.host.scale_factor(window))
                        .unwrap_or(1.0);
                    (dip * scale).round() as i32
                })
                .unwrap_or(0),
        };
        Some((rect, item_y))
    }

    // ── Windows ─────────────────────────────────────────────────────────

    /// Record `window` as the live window for `depth`, closing whatever it
    /// replaces.
    ///
    /// Hosts that create a window per menu (Reactor) hand back a new handle each
    /// time a level is shown, so the previous window for that depth is orphaned.
    /// Closing it here is what keeps `hide_all` authoritative: a window left out
    /// of the registry stays visible and makes the host report that focus left
    /// the menu, which then dismisses the rest.
    fn claim_window(&mut self, depth: usize, window: H::Window) {
        if let Some(displaced) = self.state.register(depth, window) {
            log::warn(
                "Menu::claim_window",
                &format!("depth {depth} replaced a live window — closing the old one"),
            );
            self.host.close_window(displaced);
        }
    }

    /// Open the window for `request` and remember it.
    ///
    /// No geometry is applied — see the [`MenuHost`] two-phase contract.
    pub fn open(&mut self, request: &MenuShowRequest) -> Option<H::Window> {
        let window = self.host.open_window(&request.window_input())?;
        self.claim_window(request.depth, window);
        self.levels.insert(
            request.depth,
            OpenLevel {
                request: request.clone(),
                rect: None,
            },
        );
        Some(window)
    }

    /// Register a window the frontend created itself.
    ///
    /// The Reactor counterpart of [`Self::open`], for frameworks that only allow
    /// opening windows from inside their own render callback. The level must
    /// already be open (from [`Self::show_root`] or [`Self::hover`]); this only
    /// records the window so [`Self::place`] can reach it.
    ///
    /// Returns `false` if the level is no longer open, which happens when the
    /// menu was dismissed before the window finished being created.
    pub fn adopt(&mut self, depth: usize, window: H::Window) -> bool {
        if !self.levels.contains_key(&depth) {
            log::warn("Menu::adopt", &format!("level {depth} is not open"));
            return false;
        }
        self.claim_window(depth, window);
        true
    }

    /// Position a level's window from its `measurement`.
    ///
    /// This is where the shared clamp/flip algorithm runs. Pass the real
    /// frontend measurement when available, or [`Measurement::exact`] of the
    /// estimated size beforehand so the window is usable immediately.
    ///
    /// Returns the final rectangle, or `None` if the level is no longer open (it
    /// may have been dismissed while the measurement was in flight).
    pub fn place(&mut self, depth: usize, measurement: Measurement) -> Option<Rect> {
        if !measurement.is_valid() {
            log::warn("Menu::place", "ignoring invalid measurement");
            return None;
        }

        let open = self.levels.get(&depth)?.clone();
        let window = self.state.window(depth)?;

        let info = open.request.position_info(measurement);
        let at = compute_window_position(&info, &self.metrics, &self.host.work_areas());
        let rect = Rect::at(at, measurement.window);

        self.host
            .place_window(window, &open.request.window_input(), rect);

        if let Some(open) = self.levels.get_mut(&depth) {
            open.rect = Some(rect);
        }
        Some(rect)
    }

    /// Close every level deeper than `depth`.
    ///
    /// Focus is handed back to the level that remains. Closing the focused
    /// window makes the OS choose a new foreground window, which is not
    /// guaranteed to be one of ours — without this, the menu reports "focus left"
    /// and dismisses itself while the pointer is still on it.
    pub fn hide_deeper_than(&mut self, depth: usize) {
        let windows = self.state.take_deeper_than(depth);
        let closed_anything = !windows.is_empty();
        for window in windows {
            self.host.close_window(window);
        }
        let deeper: Vec<usize> = self.levels.keys().copied().filter(|d| *d > depth).collect();
        for d in deeper {
            self.levels.remove(&d);
        }

        if closed_anything && let Some(window) = self.state.window(depth) {
            log::event(
                "Menu::hide_deeper_than",
                "focus",
                &format!("depth {depth} — restored after closing deeper levels"),
            );
            self.host.focus_window(window);
        }
    }

    /// Close every menu level.
    pub fn hide_all(&mut self) {
        let windows = self.state.take_all();
        for window in windows {
            self.host.close_window(window);
        }
        self.levels.clear();
    }

    // ── Focus / blur / idle ─────────────────────────────────────────────

    /// Handle a blur reported by one of our windows.
    ///
    /// Dismissal is decided from **who holds focus**, never from which window
    /// reported the blur. Focus legitimately moves between our own windows (a
    /// parent hands it to the submenu it just opened, and closing a level makes
    /// the OS reassign it), and those blur events arrive *after* our
    /// bookkeeping has already changed. Comparing depths against a blur event
    /// therefore cannot tell "focus moved within the menu" from "focus left the
    /// menu" — it produced spurious dismissals.
    ///
    /// The reliable signal is [`MenuHost::is_window_focused`], so this is just an
    /// immediate form of [`Self::handle_idle`]: the caller queries every open
    /// window and passes the answer.
    ///
    /// Returns `true` when the menu was closed.
    pub fn blur(&mut self, any_window_focused: bool) -> bool {
        self.handle_idle(any_window_focused)
    }

    /// Drive the click-away dismiss and the auto-hide timeout.
    ///
    /// This is the **only** decision point for dismissing the menu.
    /// `foreground_is_ours` must be a fresh query — "is any open menu window
    /// focused right now" — normally from [`MenuHost::is_window_focused`].
    ///
    /// Returns `true` when the menu was closed.
    pub fn handle_idle(&mut self, foreground_is_ours: bool) -> bool {
        if !self.state.has_windows() || self.dev_mode {
            return false;
        }

        if foreground_is_ours {
            self.state.note_foreground();
            return false;
        }

        // Click-away: only once a window has actually held focus, and only when
        // the loss survives two consecutive checks with no interaction in
        // between (`touch` clears the counter).
        if self.state.was_foreground() && self.state.note_foreground_miss() {
            log::info("Menu::idle", "focus left the menu — hiding all menus");
            self.hide_all();
            return true;
        }

        if self.state.is_idle(self.metrics.auto_hide_ms) {
            log::info("Menu::idle", "auto-hide timeout — hiding all menus");
            self.hide_all();
            return true;
        }

        false
    }

    /// Reset the idle timer without showing anything.
    pub fn touch(&mut self) {
        self.state.touch();
    }

    /// Whether the menu should close after running a command.
    pub fn close_after_execute(&self, command: &CommandPayload) -> bool {
        log::event(
            "RECV",
            "menu-execute",
            &format!("cmd='{}' admin={}", command.cmd, command.admin),
        );
        !self.dev_mode
    }

    /// Close the menu after a command, if policy says so. Returns whether it did.
    pub fn finish_execute(&mut self, command: &CommandPayload) -> bool {
        if self.close_after_execute(command) {
            self.hide_all();
            true
        } else {
            false
        }
    }
}
