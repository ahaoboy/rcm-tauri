//! System tray implementation using `tray-icon` crate.
//! Mirrors Tauri tray with same menu items and functionality.
//! Menu events are processed on the main thread via the Slint timer.

use rcm_core::{config, log, registry};
use std::io::Write;
use std::sync::Mutex;
use tray_icon::{
    TrayIconBuilder,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

// ── IDs ─────────────────────────────────────────────────────────────────

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

// ── Labels ──────────────────────────────────────────────────────────────

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

fn get_toggle_text(enabled: bool) -> &'static str {
    if enabled { DISABLE_TEXT } else { ENABLE_TEXT }
}
fn current_is_win11() -> bool { rcm_com::get_menu_style() == "Win11" }
fn sync_checks(win11: &CheckMenuItem, classic: &CheckMenuItem) {
    let is = current_is_win11();
    win11.set_checked(is);
    classic.set_checked(!is);
}
fn dump_env() {
    let exe = match std::env::current_exe() {
        Ok(p) => p, Err(_) => return,
    };
    let ep = { let mut p = exe.clone(); p.set_extension("exe.env"); p };
    let mut vars: Vec<_> = std::env::vars().collect();
    vars.sort_by(|a, b| a.0.cmp(&b.0));
    if let Ok(mut f) = std::fs::File::create(&ep) {
        for (k, v) in &vars { let _ = writeln!(f, "{k}={v}"); }
    }
    println!("[tray] dump_env: {} vars → {}", vars.len(), ep.display());
}

// ── Types ───────────────────────────────────────────────────────────────

pub type IconChangeCallback = Box<dyn Fn(bool) + Send + 'static>;

/// Global menu event receiver, polled by the Slint timer.
static MENU_RX: Mutex<Option<crossbeam_channel::Receiver<MenuEvent>>> = Mutex::new(None);

/// Process pending tray menu events. Call from the Slint timer on the main thread.
pub fn process_events(on_icons_changed: &IconChangeCallback) {
    let rx_guard = MENU_RX.lock().unwrap();
    if let Some(rx) = rx_guard.as_ref() {
        while let Ok(event) = rx.try_recv() {
            drop(rx_guard);
            handle_event(event, on_icons_changed);
            return; // re-lock next iteration
        }
    }
}

fn handle_event(event: MenuEvent, cb: &IconChangeCallback) {
    match event.id().to_string().as_str() {
        QUIT_ID => std::process::exit(0),
        WIN11_STYLE_ID => { let _ = rcm_com::set_win11_menu_style(false); }
        CLASSIC_STYLE_ID => { let _ = rcm_com::set_win11_menu_style(true); }
        TOGGLE_CTX_ID => {
            if registry::get_context_menu_status() {
                let _ = registry::disable_context_menu();
            } else {
                let _ = registry::enable_context_menu();
            }
            registry::restart_explorer();
        }
        REGISTER_ID => { let _ = rcm_com::cmd::register(); }
        UNREGISTER_ID => { let _ = rcm_com::cmd::unregister(); }
        DUMP_ENV_ID => dump_env(),
        MENU_LITE_ID => config::set_lite(true),
        MENU_FULL_ID => config::set_lite(false),
        ICONS_ID => {
            let v = !config::is_icons();
            config::set_icons(v);
            cb(v);
        }
        DEV_ID => { config::set_dev(!config::is_dev()); }
        LOG_ID => {
            use std::sync::atomic::Ordering;
            let v = !log::FILE_LOGGING.load(Ordering::Relaxed);
            log::FILE_LOGGING.store(v, Ordering::Relaxed);
        }
        AUTOSTART_ID => {
            if registry::is_autostart_enabled() {
                let _ = registry::disable_autostart();
            } else {
                let _ = registry::enable_autostart();
            }
        }
        APPLY_ID => registry::restart_explorer(),
        RESET_ID => { config::reset(); cb(false); }
        _ => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tray setup
// ═══════════════════════════════════════════════════════════════════════════

pub fn setup_tray(
    on_icons_changed: IconChangeCallback,
) -> Result<tray_icon::TrayIcon, Box<dyn std::error::Error>> {
    let is_win11 = current_is_win11();
    let is_ctx = !registry::get_context_menu_status();

    let win11_i = CheckMenuItem::with_id(WIN11_STYLE_ID, WIN11_TEXT, true, is_win11, None);
    let classic_i = CheckMenuItem::with_id(CLASSIC_STYLE_ID, CLASSIC_TEXT, true, !is_win11, None);
    let toggle_i = CheckMenuItem::with_id(TOGGLE_CTX_ID, get_toggle_text(is_ctx), true, is_ctx, None);
    let reg_i = MenuItem::with_id(REGISTER_ID, REGISTER_TEXT, true, None);
    let unreg_i = MenuItem::with_id(UNREGISTER_ID, UNREGISTER_TEXT, true, None);
    let dump_i = MenuItem::with_id(DUMP_ENV_ID, DUMP_ENV_TEXT, true, None);
    let lite_i = CheckMenuItem::with_id(MENU_LITE_ID, MENU_LITE_TEXT, true, config::is_lite(), None);
    let full_i = CheckMenuItem::with_id(MENU_FULL_ID, MENU_FULL_TEXT, true, !config::is_lite(), None);
    let dev_i = CheckMenuItem::with_id(DEV_ID, DEV_TEXT, true, config::is_dev(), None);
    let icons_i = CheckMenuItem::with_id(ICONS_ID, ICONS_TEXT, true, config::is_icons(), None);
    let log_i = CheckMenuItem::with_id(LOG_ID, LOG_TEXT, true,
        log::FILE_LOGGING.load(std::sync::atomic::Ordering::Relaxed), None);
    let auto_i = CheckMenuItem::with_id(AUTOSTART_ID, AUTOSTART_TEXT, true,
        registry::is_autostart_enabled(), None);
    let reset_i = MenuItem::with_id(RESET_ID, RESET_TEXT, true, None);
    let apply_i = MenuItem::with_id(APPLY_ID, APPLY_TEXT, true, None);
    let quit_i = MenuItem::with_id(QUIT_ID, QUIT_TEXT, true, None);

    let menu = Menu::with_items(&[
        &win11_i, &classic_i,
        &PredefinedMenuItem::separator(),
        &toggle_i,
        &PredefinedMenuItem::separator(),
        &reg_i, &unreg_i, &dump_i,
        &PredefinedMenuItem::separator(),
        &lite_i, &full_i, &icons_i,
        &PredefinedMenuItem::separator(),
        &dev_i,
        &PredefinedMenuItem::separator(),
        &log_i,
        &PredefinedMenuItem::separator(),
        &auto_i,
        &PredefinedMenuItem::separator(),
        &apply_i,
        &PredefinedMenuItem::separator(),
        &reset_i, &quit_i,
    ])?;

    let icon = load_icon()?;

    // Store the menu event receiver for polling in the timer
    let rx = MenuEvent::receiver();
    *MENU_RX.lock().unwrap() = Some(rx.clone());
    // Keep the original receiver alive
    std::mem::forget(rx);

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("RCM — Right Click Menu")
        .with_icon(icon)
        .build()?;

    println!("[tray] System tray created");
    Ok(tray)
}

// ── Icon loading ────────────────────────────────────────────────────────

fn load_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in &["icons/32x32.png", "icons/icon.ico"] {
                let p = dir.join(name);
                if p.exists() {
                    if let Ok(icon) = tray_icon::Icon::from_path(&p, None) {
                        return Ok(icon);
                    }
                }
            }
        }
    }
    if let Ok(icon) = tray_icon::Icon::from_path("icons/32x32.png", None) {
        return Ok(icon);
    }
    // Fallback: 16x16 RGBA
    let (w, h) = (16u32, 16u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 - 8.0;
            let dy = y as f64 - 8.0;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= 6.0 {
                if d <= 3.0 {
                    rgba.extend_from_slice(&[0, 180, 180, 255]);
                } else {
                    rgba.extend_from_slice(&[255, 200, 0, 255]);
                }
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Ok(tray_icon::Icon::from_rgba(rgba, w, h)?)
}
