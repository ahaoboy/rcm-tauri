//! System tray implementation using the `tray-icon` crate.
//!
//! Mirrors the Tauri tray with the same menu items and functionality.
//! Menu events are polled on the main thread via a crossbeam channel,
//! processed in the egui update loop.

use rcm_core::{config, log, registry};
use std::io::Write;
use std::sync::Mutex;
use tray_icon::{
    TrayIconBuilder,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, accelerator::Accelerator},
};

// ── Menu item IDs (must match Tauri tray for consistency) ───────────────

pub const TOGGLE_CTX_ID: &str = "toggle_ctx";
pub const WIN11_STYLE_ID: &str = "style_win11";
pub const CLASSIC_STYLE_ID: &str = "style_classic";
pub const APPLY_ID: &str = "apply";
pub const REGISTER_ID: &str = "register";
pub const UNREGISTER_ID: &str = "unregister";
pub const DUMP_ENV_ID: &str = "dump_env";
pub const DEV_ID: &str = "dev";
pub const MENU_LITE_ID: &str = "menu_lite";
pub const MENU_FULL_ID: &str = "menu_full";
pub const ICONS_ID: &str = "icons";
pub const LOG_ID: &str = "log";
pub const AUTOSTART_ID: &str = "autostart";
pub const RESET_ID: &str = "reset";
pub const QUIT_ID: &str = "quit";

// ── Display labels ──────────────────────────────────────────────────────

const QUIT_TEXT: &str = "Quit";
const ENABLE_TEXT: &str = "Enable";
const DISABLE_TEXT: &str = "Disable";
const WIN11_TEXT: &str = "Win11";
const CLASSIC_TEXT: &str = "Classic";
const REGISTER_TEXT: &str = "Register";
const UNREGISTER_TEXT: &str = "Unregister";
const DUMP_ENV_TEXT: &str = "DumpEnv";
const DEV_TEXT: &str = "Dev";
const MENU_LITE_TEXT: &str = "Lite";
const MENU_FULL_TEXT: &str = "Full";
const ICONS_TEXT: &str = "Icons";
const LOG_TEXT: &str = "Log";
const AUTOSTART_TEXT: &str = "Startup";
const RESET_TEXT: &str = "Reset";
const APPLY_TEXT: &str = "Apply";

// ── Helpers ─────────────────────────────────────────────────────────────

/// Get the appropriate toggle text based on current state.
fn get_toggle_text(enabled: bool) -> &'static str {
    if enabled { DISABLE_TEXT } else { ENABLE_TEXT }
}

/// Check if the current menu style is Windows 11.
fn current_is_win11() -> bool {
    rcm_com::get_menu_style() == "Win11"
}

/// Synchronize both style CheckMenuItems so only one is checked at a time.
#[allow(dead_code)]
fn sync_checks(win11: &CheckMenuItem, classic: &CheckMenuItem) {
    let is_win11 = current_is_win11();
    let _ = win11.set_checked(is_win11);
    let _ = classic.set_checked(!is_win11);
}

/// Write all current process environment variables to `<exe_path>.env`.
fn dump_env() {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let env_path = {
        let mut p = exe.clone();
        p.set_extension("exe.env");
        p
    };
    let mut vars: Vec<(String, String)> = std::env::vars().collect();
    vars.sort_by(|a, b| a.0.cmp(&b.0));
    if let Ok(mut f) = std::fs::File::create(&env_path) {
        for (k, v) in &vars {
            let _ = writeln!(f, "{k}={v}");
        }
    }
    log::info("tray", &format!("dump_env: {} vars → {}", vars.len(), env_path.display()));
}

// ── Event channel ───────────────────────────────────────────────────────

/// Global menu event receiver, polled by the egui update loop.
static MENU_RX: Mutex<Option<crossbeam_channel::Receiver<MenuEvent>>> = Mutex::new(None);

/// Process all pending tray menu events. Call from the egui update loop.
/// Returns `true` if the app should exit.
pub fn process_events() -> bool {
    let rx_guard = MENU_RX.lock().unwrap();
    if let Some(rx) = rx_guard.as_ref() {
        while let Ok(event) = rx.try_recv() {
            if handle_event(event) {
                return true;
            }
        }
    }
    false
}

/// Handle a single tray menu event. Returns `true` if the app should exit.
fn handle_event(event: MenuEvent) -> bool {
    match event.id().as_ref() {
        QUIT_ID => {
            log::info("tray", "Quit selected, exiting");
            return true;
        }

        // ── Style switching ────────────────────────────────────────
        WIN11_STYLE_ID => {
            let _ = rcm_com::set_win11_menu_style(false);
            log::info("tray", "switched to Win11 style");
        }
        CLASSIC_STYLE_ID => {
            let _ = rcm_com::set_win11_menu_style(true);
            log::info("tray", "switched to Classic style");
        }

        // ── Toggle RCM context menu ────────────────────────────────
        TOGGLE_CTX_ID => {
            if registry::get_context_menu_status() {
                let _ = registry::disable_context_menu();
                log::info("tray", "context menu disabled");
            } else {
                let _ = registry::enable_context_menu();
                log::info("tray", "context menu enabled");
            }
            registry::restart_explorer();
        }

        // ── Register / Unregister shell extension ─────────────────
        REGISTER_ID => {
            let _ = rcm_com::cmd::register();
            log::info("tray", "shell extension registered");
        }
        UNREGISTER_ID => {
            let _ = rcm_com::cmd::unregister();
            log::info("tray", "shell extension unregistered");
        }

        // ── Dump environment variables ────────────────────────────
        DUMP_ENV_ID => dump_env(),

        // ── Menu mode switching ────────────────────────────────────
        MENU_LITE_ID => {
            config::set_lite(true);
            log::info("tray", "switched to lite menu");
        }
        MENU_FULL_ID => {
            config::set_lite(false);
            log::info("tray", "switched to full menu");
        }

        // ── Toggle icons ───────────────────────────────────────────
        ICONS_ID => {
            let new_val = !config::is_icons();
            config::set_icons(new_val);
            log::info("tray", &format!("icons {}", if new_val { "enabled" } else { "disabled" }));
        }

        // ── Toggle dev mode ────────────────────────────────────────
        DEV_ID => {
            let new_val = !config::is_dev();
            config::set_dev(new_val);
            log::info("tray", &format!("dev mode {}", if new_val { "ON" } else { "OFF" }));
        }

        // ── Toggle file logging ────────────────────────────────────
        LOG_ID => {
            use std::sync::atomic::Ordering;
            let new_val = !log::FILE_LOGGING.load(Ordering::Relaxed);
            log::FILE_LOGGING.store(new_val, Ordering::Relaxed);
            log::info("tray", &format!("file logging {} (path: {})",
                if new_val { "ON" } else { "OFF" },
                log::log_path_display()));
        }

        // ── Toggle autostart ───────────────────────────────────────
        AUTOSTART_ID => {
            if registry::is_autostart_enabled() {
                match registry::disable_autostart() {
                    Ok(()) => log::info("tray", "autostart disabled"),
                    Err(e) => log::error("tray", &format!("disable autostart failed: {e}")),
                }
            } else {
                match registry::enable_autostart() {
                    Ok(()) => log::info("tray", "autostart enabled"),
                    Err(e) => log::error("tray", &format!("enable autostart failed: {e}")),
                }
            }
        }

        // ── Apply (restart Explorer) ───────────────────────────────
        APPLY_ID => {
            let _ = rcm_com::restart_explorer();
            log::info("tray", "explorer restart requested");
        }

        // ── Reset to defaults ──────────────────────────────────────
        RESET_ID => {
            config::reset();
            log::info("tray", "config reset to defaults");
        }

        _ => {
            log::info("tray", &format!("unhandled event id: {}", event.id().as_ref()));
        }
    }
    false
}

// ═══════════════════════════════════════════════════════════════════════════
// Tray setup
// ═══════════════════════════════════════════════════════════════════════════

/// Build and display the system tray icon with full context menu.
/// Returns the `TrayIcon` handle which must be kept alive.
pub fn setup_tray() -> Result<tray_icon::TrayIcon, Box<dyn std::error::Error>> {
    let is_win11 = current_is_win11();
    let is_ctx_enabled = !registry::get_context_menu_status();

    // ── Build menu items ──────────────────────────────────────────
    let win11_i = CheckMenuItem::with_id(WIN11_STYLE_ID, WIN11_TEXT, true, is_win11, None::<Accelerator>);
    let classic_i = CheckMenuItem::with_id(CLASSIC_STYLE_ID, CLASSIC_TEXT, true, !is_win11, None::<Accelerator>);
    let toggle_i = CheckMenuItem::with_id(TOGGLE_CTX_ID, get_toggle_text(is_ctx_enabled), true, is_ctx_enabled, None::<Accelerator>);
    let reg_i = MenuItem::with_id(REGISTER_ID, REGISTER_TEXT, true, None::<Accelerator>);
    let unreg_i = MenuItem::with_id(UNREGISTER_ID, UNREGISTER_TEXT, true, None::<Accelerator>);
    let dump_i = MenuItem::with_id(DUMP_ENV_ID, DUMP_ENV_TEXT, true, None::<Accelerator>);
    let lite_i = CheckMenuItem::with_id(MENU_LITE_ID, MENU_LITE_TEXT, true, config::is_lite(), None::<Accelerator>);
    let full_i = CheckMenuItem::with_id(MENU_FULL_ID, MENU_FULL_TEXT, true, !config::is_lite(), None::<Accelerator>);
    let dev_i = CheckMenuItem::with_id(DEV_ID, DEV_TEXT, true, config::is_dev(), None::<Accelerator>);
    let icons_i = CheckMenuItem::with_id(ICONS_ID, ICONS_TEXT, true, config::is_icons(), None::<Accelerator>);
    let log_i = CheckMenuItem::with_id(LOG_ID, LOG_TEXT, true,
        log::FILE_LOGGING.load(std::sync::atomic::Ordering::Relaxed), None::<Accelerator>);
    let auto_i = CheckMenuItem::with_id(AUTOSTART_ID, AUTOSTART_TEXT, true,
        registry::is_autostart_enabled(), None::<Accelerator>);
    let reset_i = MenuItem::with_id(RESET_ID, RESET_TEXT, true, None::<Accelerator>);
    let apply_i = MenuItem::with_id(APPLY_ID, APPLY_TEXT, true, None::<Accelerator>);
    let quit_i = MenuItem::with_id(QUIT_ID, QUIT_TEXT, true, None::<Accelerator>);

    // ── Build menu layout ─────────────────────────────────────────
    //
    //   ✓ Win11 / Classic
    //   ─────────
    //   ✓ Enable / Disable
    //   ─────────
    //     Register / Unregister / Dump Env
    //   ─────────
    //   ✓ Lite / Full / Icons
    //   ─────────
    //   ✓ Dev Mode
    //   ─────────
    //   ✓ Log
    //   ─────────
    //   ✓ Auto Start
    //   ─────────
    //     Apply
    //   ─────────
    //     Reset / Quit
    let menu = Menu::with_items(&[
        &win11_i,
        &classic_i,
        &PredefinedMenuItem::separator(),
        &toggle_i,
        &PredefinedMenuItem::separator(),
        &reg_i,
        &unreg_i,
        &dump_i,
        &PredefinedMenuItem::separator(),
        &lite_i,
        &full_i,
        &icons_i,
        &PredefinedMenuItem::separator(),
        &dev_i,
        &PredefinedMenuItem::separator(),
        &log_i,
        &PredefinedMenuItem::separator(),
        &auto_i,
        &PredefinedMenuItem::separator(),
        &apply_i,
        &PredefinedMenuItem::separator(),
        &reset_i,
        &quit_i,
    ])?;

    let icon = load_icon()?;

    // Store the menu event receiver for polling in the egui update loop
    let rx = MenuEvent::receiver();
    *MENU_RX.lock().unwrap() = Some(rx.clone());
    // Keep the original receiver alive (clone stored above)
    let _ = rx;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(true)
        .with_tooltip("RCM — Right Click Menu")
        .with_icon(icon)
        .build()?;

    log::info("tray", "system tray icon created successfully");
    Ok(tray)
}

// ── Icon loading ────────────────────────────────────────────────────────

/// Load tray icon from file, falling back to a procedural icon.
fn load_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    // Try loading from the exe directory first
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in &["icons/32x32.png", "icons/icon.ico"] {
                let p = dir.join(name);
                if p.exists() {
                    if let Ok(icon) = tray_icon::Icon::from_path(&p, None) {
                        log::info("tray", &format!("loaded icon from {}", p.display()));
                        return Ok(icon);
                    }
                }
            }
        }
    }

    // Try relative path from CWD
    for name in &["icons/32x32.png", "icons/icon.ico"] {
        let p = std::path::Path::new(name);
        if p.exists() {
            if let Ok(icon) = tray_icon::Icon::from_path(p, None) {
                log::info("tray", &format!("loaded icon from {}", p.display()));
                return Ok(icon);
            }
        }
    }

    // Fallback: generate a simple 16x16 RGBA icon procedurally
    log::info("tray", "no icon file found, generating fallback icon");
    generate_fallback_icon()
}

/// Generate a 16x16 RGBA fallback icon (a simple colored circle).
fn generate_fallback_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    let (w, h) = (16u32, 16u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 - 8.0;
            let dy = y as f64 - 8.0;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= 6.0 {
                if d <= 3.0 {
                    // Inner circle: teal
                    rgba.extend_from_slice(&[0, 180, 180, 255]);
                } else {
                    // Outer ring: gold
                    rgba.extend_from_slice(&[255, 200, 0, 255]);
                }
            } else {
                // Transparent background
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Ok(tray_icon::Icon::from_rgba(rgba, w, h)?)
}
