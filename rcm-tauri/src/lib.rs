// ═══════════════════════════════════════════════════════════════════════════
// Application entry point — module declarations, Tauri commands, setup.
// ═══════════════════════════════════════════════════════════════════════════

pub mod events;
pub mod layout;
pub mod menu_host;
pub mod monitor;
pub mod tray;

use crate::events::ConfigPayload;
use crate::events::{MenuExecutePayload, MenuHoverPayload, MenuMeasuredPayload};
use crate::layout::MenuManager;
use rcm_core::{config, log};
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

/// Return CSS content for the frontend.
/// Cached after the first load — all windows share the same CSS.
#[tauri::command]
fn get_style_css() -> String {
    rcm_core::style::load_style_css()
}
// ── Config editor commands ───────────────────────────────────────────────

/// Read a config file from the exe directory.
#[tauri::command]
fn read_config_file(name: String) -> Result<String, String> {
    rcm_core::files::read_config_file(&name)
}

/// Save content to a config file in the exe directory.
#[tauri::command]
fn save_config_file(name: String, content: String) -> Result<(), String> {
    rcm_core::files::save_config_file(&name, &content)
}

/// Open a config file with the system default program.
#[tauri::command]
fn open_in_editor(name: String) -> Result<(), String> {
    rcm_core::files::open_config_file(&name)
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
    // Already open — bring the existing window forward instead of making a second one.
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.show();
        let _ = win.set_focus();
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

/// A single environment variable entry.
#[derive(serde::Serialize)]
struct EnvVar {
    key: String,
    value: String,
}

/// Return every environment variable visible to this process, sorted by key
/// (case-insensitive). Used by the config editor's "env" tab.
#[tauri::command]
fn get_env_vars() -> Vec<EnvVar> {
    let mut vars: Vec<EnvVar> = std::env::vars()
        .map(|(key, value)| EnvVar { key, value })
        .collect();
    vars.sort_by_key(|v| v.key.to_lowercase());
    vars
}

// ═══════════════════════════════════════════════════════════════════════════
// Application entry point
// ═══════════════════════════════════════════════════════════════════════════

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Step 1: check if another RCM process is already running
    if rcm_core::process::is_rcm_process_running() {
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

/// Default error-window size — fits a short one or two line message.
const ERROR_WINDOW_SIZE: (f64, f64) = (440.0, 220.0);

fn show_error_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    title: &str,
    message: &str,
) -> Result<(), String> {
    let url = format!("index.html#error/{}", urlencoding(message));
    create_message_window(app, title, &url, ERROR_WINDOW_SIZE)
}

/// Open the shell-extension diagnostic window.
///
/// Uses its own route (`#shell-ext/…`) rather than the generic `#error/` page,
/// because this one is interactive: it offers a **Retry** button so the user can
/// reconnect after registering the extension and restarting Explorer.
fn show_shell_extension_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    report: &str,
) -> Result<(), String> {
    let url = format!("index.html#shell-ext/{}", urlencoding(report));
    create_message_window(
        app,
        "RCM Shell Extension Not Running",
        &url,
        (640.0, 560.0),
    )
}

/// Create a standalone webview window showing `url`.
fn create_message_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    title: &str,
    url: &str,
    (width, height): (f64, f64),
) -> Result<(), String> {
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let label = format!("rcm-error-{n}");
    tauri::WebviewWindowBuilder::new(app, &label, tauri::WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(width, height)
        .resizable(true)
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

/// Build a detailed report for a failed shell-extension handshake.
///
/// The `rcm_com` pipe lives inside Explorer (it is created by the shell
/// extension DLL when it loads), so a connection failure means the DLL is not
/// loaded — or not registered at all.
///
/// The report is deliberately terse: only the *problems* are listed (never the
/// healthy checks), packed onto a single line each, so the whole thing stays
/// three lines long no matter how the registry looks.
fn shell_extension_report(err: &dyn std::fmt::Display) -> String {
    let dll = rcm_core::exe_dir().join("rcm_com.dll");
    let mut issues: Vec<String> = Vec::new();

    if !dll.exists() {
        issues.push(format!("DLL missing ({})", dll.display()));
    }

    match rcm_com::cmd::status() {
        Ok(s) => {
            if !s.clsid_exists {
                issues.push("CLSID not registered".into());
            } else if s.inproc_path.is_none() {
                issues.push("InProcServer32 missing".into());
            }
            let missing: Vec<&str> = s
                .handlers
                .iter()
                .filter(|h| !h.ok)
                .map(|h| h.label.as_str())
                .collect();
            if !missing.is_empty() {
                issues.push(format!("handlers missing: {}", missing.join(", ")));
            }
            if !s.is_approved {
                issues.push("not in Approved list".into());
            }
        }
        Err(e) => issues.push(format!("status unavailable ({e})")),
    }

    let issues = if issues.is_empty() {
        "none — registration looks correct, the DLL is just not loaded yet".to_string()
    } else {
        issues.join("; ")
    };

    // Drop the crate's generic "Environment Error: " prefix so the line reads
    // as a plain sentence.
    let raw = err.to_string();
    let reason = raw.strip_prefix("Environment Error: ").unwrap_or(&raw);

    format!(
        "Shell extension not loaded — {reason}\n\
         Issues: {issues}\n\
         Fix: tray → Register → Apply → right-click a folder → Retry"
    )
}

/// Outcome of a Retry attempt from the shell-extension window.
#[derive(serde::Serialize)]
struct RetryResult {
    /// `true` when the pipe connected (the menu is now live).
    ok: bool,
    /// Fresh report to display when `ok` is `false`; empty on success.
    report: String,
}

/// Re-probe the shell extension after the user has registered it and restarted
/// Explorer. Called by the Retry button in the diagnostic window.
#[tauri::command]
fn retry_shell_extension() -> RetryResult {
    match rcm_com::enable() {
        Ok(()) => {
            log::info("Startup", "retry: shell extension connected");
            RetryResult {
                ok: true,
                report: String::new(),
            }
        }
        Err(e) => {
            log::error("Startup", &format!("retry failed: {e}"));
            RetryResult {
                ok: false,
                report: shell_extension_report(&e),
            }
        }
    }
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, args, _cwd| {
            eprintln!(
                "rcm-tauri: another instance is already running (args: {:?})",
                args
            );
        }))
        .setup(move |app| {
            config::init();

            // The root window is declared `visible` in tauri.conf.json purely so
            // WebView2 starts warming up; it must not stay on screen, because
            // with no menu data it renders an empty page. The menu controller
            // reveals it (via `place_window`) on the first right-click event.
            if let Some(root) = app.get_webview_window(events::ROOT_LABEL) {
                let _ = root.hide();
            }


            // Probe the shell extension. It owns the `rcm_com` pipe, so without
            // it no right-click event can ever reach us — the app would look
            // broken (blank window, no menu ever appears). Report the DLL path,
            // registry status and the fix instead of failing silently.
            if let Err(e) = rcm_com::enable() {
                log::error("Startup", &format!("rcm_com::enable failed: {e}"));
                let _ = show_shell_extension_window(app.app_handle(), &shell_extension_report(&e));
            }
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

            // Install the shared menu controller and pre-create the pool of
            // reusable submenu windows, so opening one never has to build a
            // webview on the critical path.
            let manager = MenuManager::new(app.app_handle().clone());
            layout::init(app.app_handle().clone());
            manager.pre_create_submenus();
            manager.start_idle_watchdog();

            // Register event listeners for frontend → backend communication.
            let app_handle = app.app_handle().clone();

            // ── Frontend log bridge ─────────────────────────────────
            app_handle.listen("log-event", move |event| {
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    let tag = payload["tag"].as_str().unwrap_or("FE");
                    let msg = payload["msg"].as_str().unwrap_or("");
                    log::frontend(tag, msg);
                }
            });

            // ── Hover: which row the pointer entered ────────────────
            let m1 = manager.clone();
            app_handle.listen("menu-hover", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuHoverPayload>(event.payload()) {
                    m1.handle_hover(&payload);
                }
            });

            // ── Measured: how large the frontend drew the level ─────
            // This is the input that drives placement.
            let m2 = manager.clone();
            app_handle.listen("menu-measured", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuMeasuredPayload>(event.payload()) {
                    m2.handle_measured(&payload);
                }
            });

            // ── Execute ────────────────────────────────────────────
            let m3 = manager.clone();
            app_handle.listen("menu-execute", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuExecutePayload>(event.payload()) {
                    m3.handle_execute(payload);
                }
            });

            // ── Close-all from frontend (e.g. Escape key) ───────────
            let m4 = manager.clone();
            app_handle.listen("menu-close-all", move |_| {
                if !config::is_dev() {
                    m4.hide_all();
                }
            });

            // Dismissal is not event-driven: `start_idle_watchdog` polls which
            // window holds focus, which is the only way to tell "focus moved
            // between our windows" from "focus left the menu".

            // Start the external event monitor
            monitor::start_monitoring(manager);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            get_env_vars,
            retry_shell_extension,
            show_error,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
