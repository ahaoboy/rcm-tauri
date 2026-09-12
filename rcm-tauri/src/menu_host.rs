//! Tauri implementation of [`rcm_core::ui::MenuHost`].
//!
//! Window identity is the Tauri window *label* — `&'static str`, so it satisfies
//! the `Copy + Ord` bound without allocating, and the labels are the ones the
//! frontend already routes on (`index.html#<label>`).
//!
//! # Division of labour
//!
//! All layout decisions are made by `rcm_core::ui`. This host only:
//!
//! - **`open_window`** — ensure the window exists, then emit `menu-show` so the
//!   WebView renders the level. No geometry is applied.
//! - **`place_window`** — apply the final rectangle the controller computed:
//!   resize, move, reveal, focus.
//! - **`hide_window` / `close_window` / `is_window_focused` / `work_areas`**.
//!
//! The frontend never computes a position; it only measures its own DOM and
//! reports the result back (see `rcm-ui/utils/measure.ts`).

use rcm_core::ui::{MenuHost, MenuWindowInput, Rect};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::events::{OFF_SCREEN, ROOT_LABEL, label_for_depth, menu_show_payload};

/// Upper bound on the number of interned `submenu-N` labels.
///
/// Label-addressed frontends need `&'static str` identities, so the pool is
/// fixed and spelled out below; this is the array size it can index into.
const MAX_SUBMENU_LABELS: usize = 8;

/// Interned `submenu-N` labels.
///
/// Label-addressed frontends need `&'static str` identities, so the pool is
/// fixed and small.
const SUBMENU_LABELS: [&str; MAX_SUBMENU_LABELS] = [
    "submenu-0",
    "submenu-1",
    "submenu-2",
    "submenu-3",
    "submenu-4",
    "submenu-5",
    "submenu-6",
    "submenu-7",
];

/// A [`MenuHost`] backed by Tauri webview windows addressed by label.
pub struct TauriHost {
    app: tauri::AppHandle,
}

impl TauriHost {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }

    /// The Tauri app handle this host wraps.
    pub fn app(&self) -> &tauri::AppHandle {
        &self.app
    }

    fn window(&self, label: &'static str) -> Option<tauri::WebviewWindow> {
        self.app.get_webview_window(label)
    }

    /// Create the transparent, hidden submenu window for `label` if needed.
    ///
    /// Returns whether the window is now available.
    pub fn ensure_labeled(&mut self, label: &str) -> bool {
        let Some(label) = intern_label(label) else {
            return false;
        };
        if self.app.get_webview_window(label).is_some() {
            return true;
        }

        let url = format!("index.html#{label}");
        let result = WebviewWindowBuilder::new(&self.app, label, WebviewUrl::App(url.into()))
            .title("rcm-submenu")
            .decorations(false)
            .position(OFF_SCREEN.x, OFF_SCREEN.y)
            .inner_size(1.0, 1.0)
            .always_on_top(true)
            .skip_taskbar(true)
            .fullscreen(false)
            .visible(false)
            .closable(false)
            .resizable(false)
            .minimizable(false)
            .maximizable(false)
            .focused(false)
            .shadow(false)
            .transparent(true)
            .build();

        match result {
            Ok(_) => true,
            Err(err) => {
                rcm_core::log::error(
                    "TauriHost::ensure_labeled",
                    &format!("failed to create '{label}': {err}"),
                );
                false
            }
        }
    }
}

impl MenuHost for TauriHost {
    type Window = &'static str;

    /// Ensure the window exists and tell the frontend which level to render.
    ///
    /// No geometry is applied — the controller calls [`Self::place_window`] once
    /// the frontend has measured its content.
    fn open_window(&mut self, input: &MenuWindowInput) -> Option<&'static str> {
        let label = label_for_depth(input.depth());
        if !self.ensure_labeled(label) {
            return None;
        }

        // Park the window off-screen so a stale frame from the previous show is
        // never visible while the new level renders.
        if let Some(window) = self.window(label) {
            let _ = window.set_position(OFF_SCREEN);
        }

        if let Err(e) = self.app.emit("menu-show", menu_show_payload(input)) {
            rcm_core::log::error("TauriHost::open_window", &format!("emit failed: {e}"));
            return None;
        }
        Some(label)
    }

    /// Apply the controller's final rectangle, reveal and focus the window.
    fn place_window(&mut self, window: Self::Window, _input: &MenuWindowInput, rect: Rect) {
        let Some(win) = self.window(window) else {
            return;
        };

        let _ = win.set_size(tauri::PhysicalSize::new(rect.width, rect.height));
        let _ = win.set_position(tauri::PhysicalPosition::new(rect.x, rect.y));
        let _ = win.set_always_on_top(true);
        let _ = win.show();
        let _ = win.set_focus();
    }

    fn hide_window(&mut self, window: Self::Window) {
        if let Some(win) = self.window(window) {
            let _ = win.hide();
            let _ = win.set_position(OFF_SCREEN);
        }
    }

    /// Tauri windows are reused, so closing means hiding and parking off-screen.
    fn close_window(&mut self, window: Self::Window) {
        self.hide_window(window);
    }

    fn focus_window(&mut self, window: Self::Window) {
        if let Some(win) = self.window(window) {
            let _ = win.set_always_on_top(true);
            let _ = win.set_focus();
        }
    }

    fn is_window_focused(&self, window: Self::Window) -> bool {
        self.window(window)
            .and_then(|w| w.is_focused().ok())
            .unwrap_or(true)
    }

    fn work_areas(&self) -> Vec<Rect> {
        self.app
            .available_monitors()
            .map(|monitors| {
                monitors
                    .into_iter()
                    .map(|monitor| {
                        let pos = monitor.position();
                        let size = monitor.size();
                        Rect::new(pos.x, pos.y, size.width as i32, size.height as i32)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn scale_factor(&self, window: Self::Window) -> f64 {
        self.window(window)
            .and_then(|w| w.scale_factor().ok())
            .unwrap_or(1.0)
    }
}

/// Intern one of the `submenu-N` labels (or the root label).
///
/// Returns `None` for anything outside the interned range, which can only happen
/// if the depth limit in `rcm_core::ui::MenuMetrics` is raised above the number
/// of pre-created windows.
pub fn intern_label(label: &str) -> Option<&'static str> {
    if label == ROOT_LABEL {
        return Some(ROOT_LABEL);
    }
    SUBMENU_LABELS
        .iter()
        .copied()
        .find(|candidate| *candidate == label)
        .or_else(|| {
            rcm_core::log::warn("TauriHost", &format!("no pre-created window for '{label}'"));
            None
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interns_the_root_label() {
        assert_eq!(intern_label(ROOT_LABEL), Some(ROOT_LABEL));
    }

    #[test]
    fn interns_submenu_labels() {
        assert_eq!(intern_label("submenu-0"), Some("submenu-0"));
        assert_eq!(intern_label("submenu-2"), Some("submenu-2"));
    }

    #[test]
    fn rejects_unknown_labels() {
        assert_eq!(intern_label("submenu-99"), None);
        assert_eq!(intern_label("nonsense"), None);
    }
}
