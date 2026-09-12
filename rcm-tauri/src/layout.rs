// ═══════════════════════════════════════════════════════════════════════════
// Menu manager — the Tauri frontend's bridge to the shared menu controller.
//
// All layout decisions live in `rcm_core::ui::MenuController`. This module only:
//   - owns the process-wide controller for the Tauri app handle,
//   - forwards frontend events (hover / measured / execute / blur) into it,
//   - applies the resulting window geometry through `TauriHost`.
//
// Flow for one level:
//   1. `show_root` / `handle_hover` → controller builds a `MenuShowRequest`
//   2. `controller.open(&request)`  → TauriHost emits `menu-show`, no geometry
//   3. frontend renders + measures  → emits `menu-measured`
//   4. `handle_measured`            → controller clamps/flips, TauriHost applies
//
// The frontend never computes a position; Rust never asks it for one.
// ═══════════════════════════════════════════════════════════════════════════

use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use rcm_core::runner::execute;
use rcm_core::ui::{HoverInfo, HoverResult, Measurement, MenuController, MenuMetrics, Point};
use rcm_core::{config, log};
use tauri::{Emitter, Manager};

use crate::events::{MenuExecutePayload, MenuHoverPayload, MenuMeasuredPayload, label_for_depth};
use crate::menu_host::TauriHost;

/// How often the click-away / auto-hide watchdog runs.
///
/// Dismissal needs two consecutive polls with focus outside the menu, so the
/// worst-case latency after clicking away is about twice this value. It is kept
/// short because blur events are deliberately not used as a trigger: only a live
/// focus query can tell "focus moved between our windows" from "focus left".
const IDLE_POLL_MS: u64 = 80;

/// The process-wide controller.
///
/// Tauri has a single `AppHandle`, so one controller serves the whole process —
/// which is also what makes the shared `MenuState` bookkeeping meaningful.
static CONTROLLER: OnceLock<Mutex<MenuController<TauriHost>>> = OnceLock::new();

/// Install the controller. Called once from Tauri's `setup`.
pub fn init(app: tauri::AppHandle) {
    let _ = CONTROLLER.set(Mutex::new(MenuController::new(
        TauriHost::new(app),
        MenuMetrics::for_tauri(),
    )));
}

/// Run `f` against the controller, if it has been installed.
fn with<R>(f: impl FnOnce(&mut MenuController<TauriHost>) -> R) -> Option<R> {
    CONTROLLER.get()?.lock().ok().map(|mut guard| f(&mut guard))
}

/// Sync the controller's cached config-derived settings.
fn refresh_settings(c: &mut MenuController<TauriHost>) {
    c.set_icons_enabled(config::is_icons());
    c.set_dev_mode(config::is_dev());
}

/// The manager handed around by the Tauri commands and listeners.
#[derive(Clone)]
pub struct MenuManager {
    app: tauri::AppHandle,
}

impl MenuManager {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }

    /// A [`TauriHost`] over this manager's app handle.
    fn host(&self) -> TauriHost {
        TauriHost::new(self.app.clone())
    }

    /// Ensure a submenu window with an explicit label exists.
    ///
    /// Used by the `create_window` command, which the frontend calls to lazily
    /// initialise a level beyond the pre-created pool.
    pub fn ensure_window_labeled(&self, label: &str) {
        self.host().ensure_labeled(label);
    }

    /// Ensure the pool of reusable submenu windows exists.
    ///
    /// The pool is fixed at [`rcm_core::ui::PRE_CREATED_WINDOWS`] because those
    /// are exactly the labels that can be interned, and every menu depth is
    /// capped at that count anyway.
    pub fn pre_create_submenus(&self) {
        let mut host = self.host();
        for depth in 1..=rcm_core::ui::PRE_CREATED_WINDOWS {
            host.ensure_labeled(label_for_depth(depth));
        }
    }

    // ── Show ────────────────────────────────────────────────────────────

    /// Show the root menu at `at` (the cursor, in physical pixels).
    pub fn show_root(&self, menu: rcm_core::Menu, at: Point) {
        // Pre-create the windows this menu could need, so opening one never has
        // to build a webview on the critical path.
        self.pre_create_submenus();

        let _ = with(|c| {
            refresh_settings(c);
            let request = c.show_root(menu, at);
            log::info(
                "Rust::show_root",
                &format!("at=({}, {}) path={:?}", at.x, at.y, request.path),
            );
            c.open(&request);
        });
    }

    /// Handle the pointer entering a menu item.
    ///
    /// The controller decides whether this opens a submenu and where it goes.
    pub fn handle_hover(&self, payload: &MenuHoverPayload) {
        let info: HoverInfo = payload.to_info();
        let _ = with(|c| match c.hover(&info) {
            HoverResult::Show(request) => {
                c.open(&request);
            }
            HoverResult::Leaf | HoverResult::Ignored => {}
        });
    }

    /// Handle a measurement reported by the frontend.
    ///
    /// This is where the final position is computed *and applied*, using the
    /// same clamp/flip algorithm the Reactor build runs.
    pub fn handle_measured(&self, payload: &MenuMeasuredPayload) {
        let measurement: Measurement = payload.to_measurement();
        if !measurement.is_valid() {
            log::warn("Rust::menu", "ignoring invalid measurement");
            return;
        }

        let _ = with(|c| {
            if let Some(rect) = c.place(payload.depth, measurement) {
                log::event(
                    "Rust::menu",
                    "placed",
                    &format!("depth={} rect={rect}", payload.depth),
                );
            }
        });
    }

    // ── Interaction ─────────────────────────────────────────────────────

    /// Handle execute: run the command and close all menus (unless in dev mode).
    pub fn handle_execute(&self, payload: MenuExecutePayload) {
        let cmd = payload.command;

        let close = with(|c| {
            refresh_settings(c);
            c.close_after_execute(&cmd)
        })
        .unwrap_or(true);

        tauri::async_runtime::spawn(async move {
            execute(&cmd).await;
        });

        if close {
            self.hide_all();
        }
    }

    /// Whether any menu level is currently open.
    pub fn has_open_levels(&self) -> bool {
        with(|c| c.has_levels()).unwrap_or(false)
    }

    /// Whether any open menu window currently holds focus.
    ///
    /// This is the authoritative input to dismissal: asking the OS directly is
    /// the only way to distinguish focus moving between our own windows (normal
    /// — a parent hands it to its submenu) from focus leaving the menu.
    fn foreground_is_ours(&self) -> bool {
        let depths = with(|c| c.state().depths()).unwrap_or_default();
        depths.into_iter().any(|depth| {
            self.app
                .get_webview_window(label_for_depth(depth))
                .and_then(|w| w.is_focused().ok())
                .unwrap_or(false)
        })
    }

    /// Drive the click-away dismiss and the auto-hide timeout.
    pub fn handle_idle(&self) {
        if !self.has_open_levels() {
            return;
        }
        let ours = self.foreground_is_ours();
        let _ = with(|c| c.handle_idle(ours));
    }

    // ── Hide ────────────────────────────────────────────────────────────

    /// Close every menu window.
    pub fn hide_all(&self) {
        let _ = with(|c| c.hide_all());
        let _ = self.app.emit("menu-hide-all", true);
    }

    /// Close every level deeper than `depth`.
    pub fn hide_deeper_than(&self, depth: usize) {
        let _ = with(|c| c.hide_deeper_than(depth));
    }

    /// Start the watchdog that dismisses the menu on click-away or inactivity.
    ///
    /// Spawned once at startup; the manager is `Clone` and cheap to move in.
    pub fn start_idle_watchdog(&self) {
        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(IDLE_POLL_MS)).await;
                manager.handle_idle();
            }
        });
    }
}
