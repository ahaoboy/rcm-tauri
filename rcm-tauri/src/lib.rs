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
use tauri::{Emitter, Listener, Manager};

// ═══════════════════════════════════════════════════════════════════════════
// Tauri commands
// ═══════════════════════════════════════════════════════════════════════════

#[tauri::command]
fn get_config() -> ConfigPayload {
    ConfigPayload {
        dev: config::is_dev(),
        icons: config::is_icons(),
        theme: config::theme().as_str().into(),
        js_url: config::remote_js_url(),
        css_url: config::remote_css_url(),
        config_url: config::remote_config_url(),
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

const DEFAULT_STYLE: &str = include_str!("../../rcm-ui/style.css");

/// Write the embedded default style CSS file next to the exe.
pub fn write_style_defaults() {
    let path = rcm_core::exe_dir().join("style.css");
    if let Err(e) = std::fs::write(&path, DEFAULT_STYLE) {
        eprintln!("write_style_defaults: write {} failed: {e}", path.display());
    } else {
        println!("write_style_defaults: wrote {}", path.display());
    }
}

/// Return CSS content for the frontend.
/// Cached after the first load — all windows share the same CSS.
#[tauri::command]
fn get_style_css() -> String {
    static LOADED: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    LOADED
        .get_or_init(|| {
            let file_path = rcm_core::exe_dir().join("style.css");
            if file_path.exists() {
                match std::fs::read_to_string(&file_path) {
                    Ok(css) => {
                        log::info("Style", "loaded style.css from disk");
                        return css;
                    }
                    Err(e) => log::error(
                        "Style",
                        &format!("read {} failed: {e}", file_path.display()),
                    ),
                }
            }
            if let Err(e) = std::fs::write(&file_path, DEFAULT_STYLE) {
                log::error(
                    "Style",
                    &format!("write {} failed: {e}", file_path.display()),
                );
            } else {
                log::info("Style", "wrote default style.css");
            }
            DEFAULT_STYLE.to_string()
        })
        .clone()
}

// ── Config editor commands ───────────────────────────────────────────────

const VALID_FILES: &[&str] = &["rcm.js", "style.css", "rcm.config.json"];

fn is_valid_config_file(name: &str) -> bool {
    VALID_FILES.contains(&name)
}

/// Read a config file from the exe directory.
#[tauri::command]
fn read_config_file(name: String) -> Result<String, String> {
    if !is_valid_config_file(&name) {
        return Err(format!("Invalid file: {name}"));
    }
    let path = rcm_core::exe_dir().join(&name);
    std::fs::read_to_string(&path).map_err(|e| format!("Read failed: {e}"))
}

/// Save content to a config file in the exe directory.
#[tauri::command]
fn save_config_file(name: String, content: String) -> Result<(), String> {
    if !is_valid_config_file(&name) {
        return Err(format!("Invalid file: {name}"));
    }
    let path = rcm_core::exe_dir().join(&name);
    std::fs::write(&path, &content).map_err(|e| format!("Save failed: {e}"))
}

/// Open a config file with the system default program.
#[tauri::command]
fn open_in_editor(name: String) -> Result<(), String> {
    if !is_valid_config_file(&name) {
        return Err(format!("Invalid file: {name}"));
    }
    let path = rcm_core::exe_dir().join(&name);
    rcm_core::sys_cmd("cmd")
        .args(["/c", "start", "", &path.to_string_lossy()])
        .spawn()
        .map_err(|e| format!("Open failed: {e}"))?;
    Ok(())
}

/// Broadcast updated CSS to all windows.
#[tauri::command]
fn notify_style_updated(app: tauri::AppHandle, css: String) -> Result<(), String> {
    app.emit("style-changed", css)
        .map_err(|e| format!("Emit failed: {e}"))?;
    log::info("Style", "style.css updated — broadcast to all windows");
    Ok(())
}

/// Create the config editor window.
#[tauri::command]
async fn create_config_window(app: tauri::AppHandle) -> Result<(), String> {
    let label = "config-editor";
    if app.get_webview_window(label).is_some() {
        // Already open — focus it
        if let Some(win) = app.get_webview_window(label) {
            let _ = win.show();
            let _ = win.set_focus();
        }
        return Ok(());
    }

    let url = "index.html#config/rcm.config.json".to_string();
    tauri::WebviewWindowBuilder::new(&app, label, tauri::WebviewUrl::App(url.into()))
        .title("RCM Config Editor")
        .inner_size(800.0, 550.0)
        .resizable(true)
        .build()
        .map_err(|e| format!("Failed to create window: {e}"))?;

    Ok(())
}

/// Pull latest rcm.js from configured remote URL.
#[tauri::command]
fn pull_js() -> Result<String, String> {
    let url = rcm_core::config::remote_js_url()
        .ok_or_else(|| "No remote URL configured for rcm.js".to_string())?;
    log::info("Pull", &format!("pulling rcm.js from {url}"));
    let path = rcm_core::menu::download_menu(&url)?;
    log::info("Pull", &format!("rcm.js saved to {path}"));
    Ok(path)
}

/// Pull latest style.css from configured remote URL and broadcast to all windows.
#[tauri::command]
fn pull_css(app: tauri::AppHandle) -> Result<String, String> {
    let url = rcm_core::config::remote_css_url()
        .ok_or_else(|| "No remote URL configured for style.css".to_string())?;
    log::info("Pull", &format!("pulling style.css from {url}"));
    let path = rcm_core::menu::download_style(&url)?;
    log::info("Pull", &format!("style.css saved to {path}"));
    // Broadcast updated CSS to all open menu windows
    let css = std::fs::read_to_string(&path).unwrap_or_default();
    let _ = app.emit("style-changed", css);
    Ok(path)
}

/// Pull latest rcm.config.json from configured remote URL.
#[tauri::command]
fn pull_config() -> Result<String, String> {
    let url = rcm_core::config::remote_config_url()
        .ok_or_else(|| "No remote URL configured for rcm.config.json".to_string())?;
    log::info("Pull", &format!("pulling rcm.config.json from {url}"));
    let path = rcm_core::menu::download_config(&url)?;
    log::info("Pull", &format!("rcm.config.json saved to {path}"));
    Ok(path)
}

// ═══════════════════════════════════════════════════════════════════════════
// Application entry point
// ═══════════════════════════════════════════════════════════════════════════

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Step 1: check if another RCM process is already running
    if pipe::is_rcm_process_running() {
        eprintln!("rcm-tauri: another instance is already running");
        run_error(
            "Another instance of RCM is already running.\n\nPlease close it before starting a new one.",
        );
        return;
    }
    run_app()
}

/// Minimal Tauri app that only shows an error window.
fn run_error(message: &str) {
    let url = format!("index.html#error/{}", urlencoding(message));
    tauri::Builder::default()
        .setup(move |app| {
            tauri::WebviewWindowBuilder::new(app, "rcm-error", tauri::WebviewUrl::App(url.into()))
                .title("RCM Error")
                .inner_size(440.0, 220.0)
                .resizable(false)
                .center()
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error window failed");
}

/// Show a small error window (non-blocking).
#[tauri::command]
async fn show_error(app: tauri::AppHandle, message: String) -> Result<(), String> {
    show_error_window(&app, "RCM Error", &message)
}

fn show_error_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    title: &str,
    message: &str,
) -> Result<(), String> {
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let url = format!("index.html#error/{}", urlencoding(message));
    let label = format!("rcm-error-{n}");
    tauri::WebviewWindowBuilder::new(app, &label, tauri::WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(440.0, 220.0)
        .resizable(false)
        .center()
        .decorations(true)
        .transparent(false)
        .always_on_top(false)
        .skip_taskbar(false)
        .build()
        .map_err(|e| {
            log::error("ErrorWindow", &format!("failed to create: {e}"));
            format!("Failed to create error window: {e}")
        })?;
    Ok(())
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            b'\n' => out.push_str("%0A"),
            _ => {
                let h = format!("%{:02X}", b);
                out.push_str(&h);
            }
        }
    }
    out
}

fn run_app() {
    let menu: MenuArc = Arc::new(Mutex::new(None));
    let auto_hide_epoch: AutoHideEpoch = Arc::new(AtomicU64::new(0));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, args, _cwd| {
            eprintln!(
                "rcm-tauri: another instance is already running (args: {:?})",
                args
            );
        }))
        .manage(menu.clone())
        .manage(auto_hide_epoch.clone())
        .setup(move |app| {
            config::init();
            tray::setup_tray(app)?;

            // Create a hidden window to warm up WebView2
            let warmup = tauri::WebviewWindowBuilder::new(
                app,
                "_warmup",
                tauri::WebviewUrl::App("index.html#warmup".into()),
            )
            .visible(false)
            .inner_size(1.0, 1.0)
            .build()?;

            let w = warmup.clone();
            app.listen("warmup-ready", move |_| {
                println!("warmup: WebView2 ready");
                let _ = w.close();
            });

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
            get_style_css,
            read_config_file,
            save_config_file,
            open_in_editor,
            notify_style_updated,
            create_config_window,
            pull_js,
            pull_css,
            pull_config,
            show_error,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
