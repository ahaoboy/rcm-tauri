// ═══════════════════════════════════════════════════════════════════════════
// Menu manager — central hub for all menu window logic.
// Handles show/hide/hover/execute/blur for the multi-window context menu.
// ═══════════════════════════════════════════════════════════════════════════

use crate::events::{
    AutoHideEpoch, MenuArc, MenuBlurPayload, MenuExecutePayload, MenuHoverOutPayload,
    MenuHoverPayload, MenuShowPayload,
};
use crate::events::{
    AUTO_HIDE_MS, DEEPEST_DEPTH, MAX_SUBMENU_DEPTH, OFF_SCREEN, SUBMENU_GAP,
};
use crate::events::{root_label, submenu_label, window_label};

use rcm_core::{config, log};
use std::sync::atomic::Ordering;
use tauri::window::Color;
use tauri::{Emitter, Manager, PhysicalPosition, WebviewUrl};

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
    pub fn show_root(&self, menu: rcm_core::Menu, x: f64, y: f64) {
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

        // Determine how many submenu windows we need
        let max_depth = menu.max_depth().min(MAX_SUBMENU_DEPTH);
        for d in 0..max_depth {
            let label = submenu_label(d);
            if self.app.get_webview_window(&label).is_none() {
                log::info("Rust::show_root", &format!("creating window '{label}'"));
                self.create_submenu_window(&label);
            }
        }

        // ── Send raw position; frontend will clamp after measuring DOM ──
        let payload = MenuShowPayload {
            menu,
            path: vec![],
            x,
            y,
            parent_x: None,
            parent_y: None,
            parent_w: None,
        };

        let label = root_label();
        if let Some(win) = self.app.get_webview_window(label) {
            let _ = win.set_position(PhysicalPosition { x, y });
        }
        log::event("SEND", "menu-show", &format!("to={label} path=[]"));
        let _ = self.app.emit("menu-show", payload);
        self.reset_auto_hide();
    }

    /// Handle hover on a menu item: show submenu if the item has children.
    pub fn handle_hover(&self, payload: MenuHoverPayload) {
        log::event(
            "RECV",
            "menu-hover",
            &format!(
                "depth={} path={:?} parent=({:.0},{:.0})",
                payload.depth, payload.path, payload.parent_x, payload.parent_y
            ),
        );

        let menu_guard = self.menu.lock().unwrap();
        let menu = match menu_guard.as_ref() {
            Some(m) => m,
            None => {
                log::warn("Rust::handle_hover", "no menu data, ignoring");
                return;
            }
        };

        // Navigate to the hovered item
        let item = match menu.get_item(&payload.path) {
            Some(i) => i,
            None => {
                log::warn(
                    "Rust::handle_hover",
                    &format!("item not found at path {:?}", payload.path),
                );
                return;
            }
        };

        log::info(
            "Rust::handle_hover",
            &format!(
                "item='{}' has_children={} disable={}",
                item.label,
                item.has_children(),
                item.disable
            ),
        );

        // If the item is disabled or has no children, hide deeper submenus
        if item.disable || !item.has_children() {
            drop(menu_guard);
            log::info(
                "Rust::handle_hover",
                &format!("leaf/disabled, hiding deeper than {}", payload.depth),
            );
            self.hide_deeper_than(payload.depth);
            return;
        }

        // Compute child depth
        let child_depth = payload.depth + 1;
        if child_depth > MAX_SUBMENU_DEPTH {
            log::warn("Rust::handle_hover", "max depth reached, ignoring");
            return;
        }

        // ── Horizontal positioning ───────────────────────────────────────
        // Position submenu at parent's .rcm-root right edge + gap.
        // #root padding is identical in both windows, so it cancels out.
        let sub_x = if payload.parent_content_w > 0.0 {
            payload.parent_x + payload.parent_content_w + SUBMENU_GAP
        } else if payload.content_right > 0.0 {
            payload.content_right
        } else {
            // Fallback for old frontends
            payload.parent_x + payload.parent_w - 28.0
        };

        // ── Vertical positioning ─────────────────────────────────────────
        // Frontend sends item_y in physical px (DPI-corrected), so we can
        // position the submenu directly without CSS constant estimates.
        let ideal_y = payload.parent_y + payload.item_y;

        // Safety clamp: don't let the submenu start below the parent window.
        let parent_bottom = payload.parent_y + payload.parent_h;
        let sub_y = if ideal_y > parent_bottom {
            let clamped = parent_bottom;
            log::info(
                "Rust::handle_hover",
                &format!(
                    "pos: ideal_y={:.0} parent_bottom={:.0} → clamped_y={:.0}",
                    ideal_y, parent_bottom, clamped
                ),
            );
            clamped
        } else {
            ideal_y
        };

        // ── Frontend will clamp to monitor after measuring real DOM size ──

        // ── Debug: log all positioning inputs ───────────────────────────
        log::info(
            "Rust::handle_hover",
            &format!(
                "pos_debug: parent=({:.0},{:.0}) parent_w={:.0} parent_h={:.0} parentCW={:.0} parentCH={:.0} | \
             item=({:.0},{:.0}) item_w={:.0} item_h={:.0} | \
             children={} | sub=({:.0},{:.0})",
                payload.parent_x,
                payload.parent_y,
                payload.parent_w,
                payload.parent_h,
                payload.parent_content_w,
                payload.parent_content_h,
                payload.item_x,
                payload.item_y,
                payload.item_w,
                payload.item_h,
                item.items.len(),
                sub_x,
                sub_y
            ),
        );
        let child_label = window_label(child_depth);

        log::info(
            "Rust::handle_hover",
            &format!("showing '{child_label}' at ({sub_x:.0},{sub_y:.0}) depth={child_depth}"),
        );

        // Only hide windows deeper than the child we're about to show
        drop(menu_guard);
        self.hide_deeper_than(child_depth);

        let menu_guard2 = self.menu.lock().unwrap();
        let menu = match menu_guard2.as_ref() {
            Some(m) => m,
            None => return,
        };

        let show_payload = MenuShowPayload {
            menu: menu.clone(),
            path: payload.path.clone(),
            x: sub_x,
            y: sub_y,
            parent_x: Some(payload.parent_x),
            parent_y: Some(payload.parent_y),
            parent_w: Some(payload.parent_w),
        };

        if let Some(win) = self.app.get_webview_window(&child_label) {
            let _ = win.set_position(PhysicalPosition { x: sub_x, y: sub_y });
            // Frontend will show after clamping to monitor
        }
        DEEPEST_DEPTH.store(child_depth, Ordering::SeqCst);
        log::info(
            "Rust::handle_hover",
            &format!("DEEPEST_DEPTH={child_depth}"),
        );
        log::event(
            "SEND",
            "menu-show",
            &format!("to={child_label} path={:?}", payload.path),
        );
        let _ = self.app.emit("menu-show", show_payload);
        self.reset_auto_hide();
    }

    /// Handle hover-out: hide all windows deeper than this one.
    pub fn handle_hover_out(&self, payload: MenuHoverOutPayload) {
        // Don't hide on hover-out immediately; let the next hover handle it.
        // Only hide if mouse truly left all menu windows.
        // For simplicity, we use a small delay approach handled in frontend.
        // The hover on a sibling item will trigger hide_deeper_than anyway.
        let _ = payload;
    }

    /// Handle execute: run the command and close all menus.
    pub fn handle_execute(&self, payload: MenuExecutePayload) {
        log::event(
            "RECV",
            "menu-execute",
            &format!("exe='{}' path={:?}", payload.command.exe, payload.path),
        );

        // Reset auto-hide on interaction
        self.reset_auto_hide();

        let cmd = payload.command;
        tauri::async_runtime::spawn(async move {
            let result = crate::cmd::execute(cmd).await;
            if !result.success {
                log::error("Rust::handle_execute", &format!("FAILED: {:?}", result));
            } else {
                log::info("Rust::handle_execute", "OK");
            }
        });

        if !config::is_dev() {
            log::info(
                "Rust::handle_execute",
                &format!("hiding all (dev={})", config::is_dev()),
            );
            self.hide_all();
        }
    }

    /// Handle blur from a menu window.
    pub fn handle_blur(&self, payload: MenuBlurPayload) {
        let deepest = DEEPEST_DEPTH.load(Ordering::SeqCst);
        log::event(
            "RECV",
            "menu-blur",
            &format!("depth={} deepest={}", payload.depth, deepest),
        );

        if payload.depth != deepest {
            log::info("Rust::handle_blur", "IGNORED (depth != deepest)");
            return;
        }
        log::info("Rust::handle_blur", "MATCH → hiding all");
        self.hide_all();
    }

    /// Hide all menu windows.
    pub fn hide_all(&self) {
        log::info("Rust::hide_all", "START");
        DEEPEST_DEPTH.store(0, Ordering::SeqCst);
        // Hide root
        if let Some(win) = self.app.get_webview_window(root_label()) {
            let _ = win.hide();
            let _ = win.set_position(OFF_SCREEN);
        }
        // Hide submenus
        for d in 0..MAX_SUBMENU_DEPTH {
            let label = submenu_label(d);
            if let Some(win) = self.app.get_webview_window(&label) {
                let _ = win.hide();
                let _ = win.set_position(OFF_SCREEN);
            }
        }
        log::event("SEND", "menu-hide-all", "broadcast");
        let _ = self.app.emit("menu-hide-all", true);
    }

    /// Hide all submenu windows strictly deeper than `depth`,
    /// then update DEEPEST_DEPTH to reflect the new reality.
    pub fn hide_deeper_than(&self, depth: usize) {
        for d in (depth + 1)..=MAX_SUBMENU_DEPTH {
            let label = window_label(d);
            if let Some(win) = self.app.get_webview_window(&label) {
                let _ = win.hide();
                let _ = win.set_position(OFF_SCREEN);
            }
        }
        // After hiding deeper windows, `depth` is now the deepest visible
        log::info(
            "Rust::hide_deeper_than",
            &format!("hid >{depth}, DEEPEST_DEPTH→{depth}"),
        );
        DEEPEST_DEPTH.store(depth, Ordering::SeqCst);
    }

    /// Create a transparent submenu window.
    pub fn create_submenu_window(&self, label: &str) {
        if self.app.get_webview_window(label).is_some() {
            return;
        }

        let url = format!("index.html#{label}");
        let builder =
            tauri::WebviewWindowBuilder::new(&self.app, label, WebviewUrl::App(url.into()))
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

        builder.build().unwrap();
    }
}
