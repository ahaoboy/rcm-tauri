//! Reactor implementation of [`rcm_core::ui::MenuHost`] and the process-wide
//! [`MenuController`] behind it.
//!
//! Every layout decision lives in `rcm_core::ui` and is shared with the Tauri
//! build. This module only supplies the toolkit half:
//!
//! - which monitor work areas exist ([`win32::list_work_areas`]);
//! - how to move (and, once measured, resize) a native `HWND`;
//! - how to close and focus one.
//!
//! Reactor cannot create windows from here — its framework only opens windows
//! from inside a component publication — so `open_window` reports that it cannot
//! and the component adopts the level with [`MenuController::adopt`].

use std::sync::{Mutex, OnceLock};

use rcm_core::ui::{
    HoverInfo, HoverResult, Measurement, MenuController, MenuHost, MenuMetrics, MenuShowRequest,
    MenuWindowInput, Rect,
};
use rcm_core::{Menu, ui};

use crate::win32;

/// Reactor identifies window levels by their raw `HWND`.
///
/// `HWND`s are opaque handles that fit in `isize`; `MenuState` only needs
/// `Copy + Ord` to key its depth map.
pub type RawWindow = isize;

/// The Reactor half of the menu: everything that touches Win32.
pub struct ReactorHost {
    scale: f64,
}

impl ReactorHost {
    fn new() -> Self {
        Self { scale: 1.0 }
    }

    /// Record the DPI scale reported by the window measurer.
    fn set_scale(&mut self, scale: f64) {
        if scale > 0.0 {
            self.scale = scale;
        }
    }
}

impl MenuHost for ReactorHost {
    type Window = RawWindow;

    /// Reactor can only open windows from inside a component publication, so
    /// this cannot work: the component creates the window and calls
    /// [`MenuController::adopt`] instead. See `menu_window::MenuWindow`.
    fn open_window(&mut self, _input: &MenuWindowInput) -> Option<Self::Window> {
        None
    }

    fn place_window(&mut self, window: Self::Window, _input: &MenuWindowInput, rect: Rect) {
        // The size was applied by `prepare_popup` (and re-applied by Reactor when
        // the content was measured), so this only positions and reveals. Setting
        // a size here would fight Reactor's DIP→pixel conversion.
        win32::place_popup(as_raw(window), rect.x, rect.y);
        win32::force_foreground(as_raw(window));
    }

    fn hide_window(&mut self, window: Self::Window) {
        win32::hide(as_raw(window));
    }

    fn close_window(&mut self, window: Self::Window) {
        // Hide first, then ask the window to close. `WM_CLOSE` is the only
        // handle we have on a Reactor window's lifetime, and it goes through
        // the framework's own message handling — if that is refused or delayed,
        // a window that was only asked to close would stay visible and the
        // screen would accumulate menus. Hiding guarantees it is gone from view
        // immediately; the close then releases it.
        win32::hide(as_raw(window));
        win32::post_close(as_raw(window));
    }

    fn focus_window(&mut self, window: Self::Window) {
        win32::force_foreground(as_raw(window));
    }

    fn is_window_focused(&self, window: Self::Window) -> bool {
        win32::foreground_raw() as isize == window
    }

    fn work_areas(&self) -> Vec<Rect> {
        win32::list_work_areas()
    }

    fn scale_factor(&self, _window: Self::Window) -> f64 {
        self.scale
    }
}

#[inline]
fn as_raw(window: RawWindow) -> *mut core::ffi::c_void {
    window as *mut core::ffi::c_void
}

fn controller() -> &'static Mutex<MenuController<ReactorHost>> {
    static CONTROLLER: OnceLock<Mutex<MenuController<ReactorHost>>> = OnceLock::new();
    CONTROLLER.get_or_init(|| {
        Mutex::new(MenuController::new(
            ReactorHost::new(),
            MenuMetrics::DEFAULT,
        ))
    })
}

/// Run `f` against the global controller.
fn with<R>(f: impl FnOnce(&mut MenuController<ReactorHost>) -> R) -> Option<R> {
    controller().lock().ok().map(|mut guard| f(&mut guard))
}

// ═══════════════════════════════════════════════════════════════════════════
// Public surface used by the UI components
// ═══════════════════════════════════════════════════════════════════════════

/// Prepare a fresh root menu and return the level the component must adopt.
pub fn show_root(menu: Menu, at: ui::Point) -> Option<MenuShowRequest> {
    with(|c| {
        c.set_icons_enabled(rcm_core::config::is_icons());
        c.set_dev_mode(rcm_core::config::is_dev());
        c.show_root(menu, at)
    })
}

/// Turn a freshly created window into a hidden, borderless popup of `size`.
///
/// Called from within `run_window`, i.e. the earliest moment the handle exists.
/// Reactor reveals a window the instant it is created, so anything deferred to
/// the next publication is a visible flash of a misplaced, titled window.
pub fn prepare_popup(window: RawWindow, size: ui::Size) {
    win32::prepare_popup(as_raw(window), size.width, size.height);
}

/// Register a window the component created.
///
/// Returns whether the level was still open; `false` means the menu was
/// dismissed while the window was being created, and the caller should close it.
pub fn adopt(depth: usize, window: RawWindow, scale: f64) -> bool {
    with(|c| {
        c.host_mut().set_scale(scale);
        c.adopt(depth, window)
    })
    .unwrap_or(false)
}

/// Place a level's window.
///
/// Pass the real frontend [`Measurement`] once available, or
/// [`Measurement::exact`] of the estimated size beforehand so the window is
/// usable immediately.
pub fn place(depth: usize, measurement: Measurement) -> Option<Rect> {
    with(|c| c.place(depth, measurement)).flatten()
}

/// Handle a pointer entering a menu item.
pub fn hover(info: &HoverInfo) -> HoverResult {
    with(|c| c.hover(info)).unwrap_or(HoverResult::Ignored)
}

/// Drive the click-away dismiss and the auto-hide timeout.
pub fn handle_idle() -> bool {
    let focused = win32::foreground_raw();
    with(|c| {
        let ours = focused.is_null() || c.state().is_open_window(focused as isize);
        c.handle_idle(ours)
    })
    .unwrap_or(false)
}

/// Close every menu window.
pub fn hide_all() {
    let _ = with(|c| c.hide_all());
}

/// Run a command's bookkeeping: log it and close the menu unless in dev mode.
pub fn finish_execute(command: &rcm_core::CommandPayload) {
    let _ = with(|c| {
        c.set_dev_mode(rcm_core::config::is_dev());
        c.finish_execute(command);
    });
}

/// Re-read the config-derived settings the controller caches.
pub fn refresh_settings() {
    let _ = with(|c| {
        c.set_icons_enabled(rcm_core::config::is_icons());
        c.set_dev_mode(rcm_core::config::is_dev());
    });
}
