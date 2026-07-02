// ═══════════════════════════════════════════════════════════════════════════
// Menu manager — central hub for menu window lifecycle.
// Handles show/hide/hover/execute/blur for the multi-window menu.
//
// Layout flow (frontend-driven):
//   1. Rust emits `menu-show` with ideal .rcm-root position (+ parent root X
//      for submenu flip).
//   2. Frontend renders content, measures DOM, resizes window, computes the
//      final position (clamp, flip, edge cases), and shows the window.
//   3. Rust does NOT participate in positioning — it only computes the ideal
//      position for submenus (parent root right edge + gap).
// ═══════════════════════════════════════════════════════════════════════════

use crate::events::{
    AUTO_HIDE_MS, DEEPEST_DEPTH, MAX_SUBMENU_DEPTH, OFF_SCREEN, SUBMENU_GAP, submenu_window_depths,
};
use crate::events::{
    AutoHideEpoch, MenuArc, MenuBlurPayload, MenuExecutePayload, MenuHoverOutPayload,
    MenuHoverPayload, MenuShowPayload,
};
use crate::events::{root_label, submenu_label, window_label};

use rcm_core::runner::execute;
use rcm_core::{config, log};
use std::sync::atomic::Ordering;
use tauri::window::Color;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub struct MenuManager {
    pub menu: MenuArc,
    pub app: tauri::AppHandle,
    pub auto_hide_epoch: AutoHideEpoch,
}

impl MenuManager {
    /// Reset the global auto-hide timer. Call on every user interaction
    /// (show, hover, click). After AUTO_HIDE_MS of inactivity, all menus
    /// are hidden simultaneously.
    pub fn reset_auto_hide(&self) {
        let epoch = self
            .auto_hide_epoch
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);
        let epoch_ref = self.auto_hide_epoch.clone();
        let app = self.app.clone();
        let menu = self.menu.clone();

        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(AUTO_HIDE_MS)).await;
            if epoch_ref.load(Ordering::SeqCst) == epoch {
                log::info("Rust::auto_hide", "timeout — hiding all menus");
                let mgr = MenuManager {
                    menu,
                    app,
                    auto_hide_epoch: epoch_ref,
                };
                mgr.hide_all();
            }
        });
    }

    /// Show the root menu at the cursor position.
    /// Emits `menu-show` — the frontend measures and positions the window.
    pub fn show_root(&self, menu: rcm_core::Menu, x: f64, y: f64) {
        self.hide_all_submenus();

        log::info(
            "Rust::show_root",
            &format!(
                "pos=({x:.0},{y:.0}) groups={} icons={} max_depth={}",
                menu.groups.len(),
                menu.icon_items.len(),
                menu.max_depth()
            ),
        );

        *self.menu.lock().unwrap() = Some(menu.clone());
        DEEPEST_DEPTH.store(0, Ordering::SeqCst);

        // Pre-create submenu windows if needed
        let max_depth = menu.max_depth().min(MAX_SUBMENU_DEPTH);
        for d in 0..max_depth {
            let label = submenu_label(d);
            if self.app.get_webview_window(&label).is_none() {
                self.create_submenu_window(&label);
            }
        }

        let _ = self.app.emit(
            "menu-show",
            MenuShowPayload {
                menu,
                path: vec![],
                x,
                y,
                parent_root_x: None,
            },
        );
        self.reset_auto_hide();
    }

    /// Handle hover on a menu item: compute the ideal submenu position
    /// and emit `menu-show`. The frontend measures and positions the window.
    pub fn handle_hover(&self, payload: MenuHoverPayload) {
        self.reset_auto_hide();

        log::event(
            "RECV",
            "menu-hover",
            &format!("depth={} path={:?}", payload.depth, payload.path),
        );

        let (menu, item) = {
            let guard = self.menu.lock().unwrap();
            let menu = match guard.as_ref() {
                Some(m) => m.clone(),
                None => {
                    log::warn("Rust::handle_hover", "no menu data");
                    return;
                }
            };
            let item = match menu.get_item(&payload.path) {
                Some(i) => i.clone(),
                None => {
                    log::warn("Rust::handle_hover", "item not found");
                    return;
                }
            };
            (menu, item)
        };

        // Leaf or disabled: just hide deeper submenus
        if item.disable || !item.has_children() {
            self.hide_deeper_than(payload.depth);
            return;
        }

        let child_depth = payload.depth + 1;
        if child_depth > MAX_SUBMENU_DEPTH {
            return;
        }

        // Ideal position for the submenu's .rcm-root:
        //   X = parent root right edge + gap
        //   Y = parent root top + hovered item offset
        let ideal_x = payload.root_x + payload.root_w + SUBMENU_GAP;
        let ideal_y = payload.root_y + payload.item_y;

        self.hide_deeper_than(child_depth);
        DEEPEST_DEPTH.store(child_depth, Ordering::SeqCst);

        let _ = self.app.emit(
            "menu-show",
            MenuShowPayload {
                menu,
                path: payload.path,
                x: ideal_x,
                y: ideal_y,
                parent_root_x: Some(payload.root_x),
            },
        );
    }

    /// Hide all submenu windows (depth > 0), keeping the root window.
    pub fn hide_all_submenus(&self) {
        self.hide_deeper_than(0);
    }

    /// Handle hover-out (no-op: sibling hover handles hiding).
    pub fn handle_hover_out(&self, _payload: MenuHoverOutPayload) {}

    /// Handle execute: run the command and close all menus.
    pub fn handle_execute(&self, payload: MenuExecutePayload) {
        self.reset_auto_hide();

        let cmd = payload.command;
        tauri::async_runtime::spawn(async move {
            let result = execute(&cmd).await;
            if !result.success {
                log::error("Rust::handle_execute", &format!("FAILED: {:?}", result));
            }
        });

        if !config::is_dev() {
            self.hide_all();
        }
    }

    /// Handle blur from the deepest menu window — hide all.
    pub fn handle_blur(&self, payload: MenuBlurPayload) {
        if payload.depth == DEEPEST_DEPTH.load(Ordering::SeqCst) {
            self.hide_all();
        }
    }

    /// Hide all menu windows.
    pub fn hide_all(&self) {
        DEEPEST_DEPTH.store(0, Ordering::SeqCst);

        if let Some(win) = self.app.get_webview_window(root_label()) {
            hide_window(&win);
        }
        for d in 0..MAX_SUBMENU_DEPTH {
            if let Some(win) = self.app.get_webview_window(&submenu_label(d)) {
                hide_window(&win);
            }
        }
        let _ = self.app.emit("menu-hide-all", true);
    }

    /// Hide all submenu windows strictly deeper than `depth`.
    pub fn hide_deeper_than(&self, depth: usize) {
        for d in submenu_window_depths().filter(|d| *d > depth) {
            if let Some(win) = self.app.get_webview_window(&window_label(d)) {
                hide_window(&win);
            }
        }
        DEEPEST_DEPTH.store(depth, Ordering::SeqCst);
    }

    /// Create a transparent submenu window.
    pub fn create_submenu_window(&self, label: &str) {
        if self.app.get_webview_window(label).is_some() {
            return;
        }

        let url = format!("index.html#{label}");
        let builder = WebviewWindowBuilder::new(&self.app, label, WebviewUrl::App(url.into()))
            .title("rcm-submenu")
            .decorations(false)
            .background_color(Color(0, 0, 0, 0))
            .position(0., 0.)
            .inner_size(1., 1.)
            .always_on_top(true)
            .skip_taskbar(true)
            .fullscreen(false)
            .visible(false)
            .closable(false)
            .resizable(false)
            .minimizable(false)
            .maximizable(false)
            .focused(false)
            .shadow(false);

        #[cfg(not(target_os = "macos"))]
        let builder = builder.transparent(true);

        if let Err(err) = builder.build() {
            log::error(
                "Rust::create_submenu_window",
                &format!("failed to create '{label}': {err}"),
            );
        }
    }
}

fn hide_window(win: &WebviewWindow) {
    let _ = win.hide();
    let _ = win.set_position(OFF_SCREEN);
}
