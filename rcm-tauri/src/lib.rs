// ═══════════════════════════════════════════════════════════════════════════
// Application entry point — module declarations, Tauri commands, setup.
// ═══════════════════════════════════════════════════════════════════════════

pub mod events;
pub mod layout;
pub mod monitor;
pub mod pipe;
pub mod tray;

use crate::events::{AutoHideEpoch, ConfigPayload, MenuArc, submenu_indices, submenu_label};
use crate::events::{MenuBlurPayload, MenuExecutePayload, MenuHoverOutPayload, MenuHoverPayload};
use crate::layout::MenuManager;
use rcm_core::{config, log};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use tauri::{Listener, Manager};

// ═══════════════════════════════════════════════════════════════════════════
// Tauri commands
// ═══════════════════════════════════════════════════════════════════════════

#[tauri::command]
fn get_config() -> ConfigPayload {
    ConfigPayload {
        dev: config::is_dev(),
        icons: config::is_icons(),
    }
}

/// Create a submenu window (called from frontend for lazy init).
#[tauri::command]
async fn create_window(app: tauri::AppHandle, label: String) {
    let mgr = MenuManager {
        menu: Arc::new(Mutex::new(None)),
        app,
        auto_hide_epoch: Arc::new(AtomicU64::new(0)),
    };
    mgr.create_submenu_window(&label);
}

/// Return CSS content for the frontend.
/// - If `style.css` exists next to the executable, return its content.
/// - Otherwise, write the default `style.css` to the exe directory and return it.
#[tauri::command]
fn get_style_css() -> String {
    let default_css = include_str!("../../rcm-ui/styles/style.css");
    let style_path = rcm_core::exe_dir().join("style.css");

    if style_path.exists() {
        std::fs::read_to_string(&style_path).unwrap_or_else(|_| default_css.to_string())
    } else {
        let _ = std::fs::write(&style_path, default_css);
        default_css.to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Application entry point
// ═══════════════════════════════════════════════════════════════════════════

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if pipe::check_client_cli() {
        return;
    }

    let menu: MenuArc = Arc::new(Mutex::new(None));
    let auto_hide_epoch: AutoHideEpoch = Arc::new(AtomicU64::new(0));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .manage(menu.clone())
        .manage(auto_hide_epoch.clone())
        .setup(move |app| {
            config::init();
            tray::setup_tray(app)?;

            let epoch = auto_hide_epoch.clone();

            // Pre-create submenu windows
            for d in submenu_indices() {
                let label = submenu_label(d);
                let mgr = MenuManager {
                    menu: menu.clone(),
                    app: app.app_handle().clone(),
                    auto_hide_epoch: epoch.clone(),
                };
                mgr.create_submenu_window(&label);
            }

            // Register event listeners for frontend → backend communication
            let app_handle = app.app_handle().clone();
            let menu_clone = menu.clone();

            // ── Frontend log bridge ─────────────────────────────────
            app_handle.listen("log-event", move |event| {
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    let tag = payload["tag"].as_str().unwrap_or("FE");
                    let msg = payload["msg"].as_str().unwrap_or("");
                    log::frontend(tag, msg);
                }
            });

            // Listen for hover events
            let ah1 = app_handle.clone();
            let m1 = menu_clone.clone();
            let e1 = epoch.clone();
            app_handle.listen("menu-hover", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuHoverPayload>(event.payload()) {
                    let mgr = MenuManager {
                        menu: m1.clone(),
                        app: ah1.clone(),
                        auto_hide_epoch: e1.clone(),
                    };
                    mgr.handle_hover(payload);
                }
            });

            // Listen for hover-out events
            let ah2 = app_handle.clone();
            let m2 = menu_clone.clone();
            let e2 = epoch.clone();
            app_handle.listen("menu-hover-out", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuHoverOutPayload>(event.payload()) {
                    let mgr = MenuManager {
                        menu: m2.clone(),
                        app: ah2.clone(),
                        auto_hide_epoch: e2.clone(),
                    };
                    mgr.handle_hover_out(payload);
                }
            });

            // Listen for execute events
            let ah3 = app_handle.clone();
            let m3 = menu_clone.clone();
            let e3 = epoch.clone();
            app_handle.listen("menu-execute", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuExecutePayload>(event.payload()) {
                    let mgr = MenuManager {
                        menu: m3.clone(),
                        app: ah3.clone(),
                        auto_hide_epoch: e3.clone(),
                    };
                    mgr.handle_execute(payload);
                }
            });

            // Listen for close-all from frontend (e.g. Escape key)
            let ah4 = app_handle.clone();
            let m4 = menu_clone.clone();
            let e4 = epoch.clone();
            app_handle.listen("menu-close-all", move |_| {
                if !config::is_dev() {
                    let mgr = MenuManager {
                        menu: m4.clone(),
                        app: ah4.clone(),
                        auto_hide_epoch: e4.clone(),
                    };
                    mgr.hide_all();
                }
            });

            // Listen for blur events — only hide all if deepest window lost focus
            let ah5 = app_handle.clone();
            let m5 = menu_clone.clone();
            let e5 = epoch.clone();
            app_handle.listen("menu-blur", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuBlurPayload>(event.payload()) {
                    let mgr = MenuManager {
                        menu: m5.clone(),
                        app: ah5.clone(),
                        auto_hide_epoch: e5.clone(),
                    };
                    mgr.handle_blur(payload);
                }
            });

            // Start the external event monitor
            monitor::start_monitoring(app_handle, menu, epoch);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_window,
            get_config,
            get_style_css
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
