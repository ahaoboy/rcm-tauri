use rcm_core::registry;
use rcm_core::{config, log};
use rcm_reg::MenuStyle;
use std::time::Duration;
use tauri::{
    App, Emitter,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

// ── Menu item IDs ────────────────────────────────────────────────────────

/// Switch to Windows 11 compact context menu style (CheckMenuItem).
pub const WIN11_STYLE_ID: &str = "style_win11";
/// Switch to classic Windows 10 context menu style (CheckMenuItem).
pub const CLASSIC_STYLE_ID: &str = "style_classic";
/// Restart Windows Explorer to apply registry changes (MenuItem).
pub const APPLY_ID: &str = "apply";
/// Register the shell extension DLL (MenuItem).
pub const REGISTER_ID: &str = "register";
/// Unregister the shell extension DLL (MenuItem).
pub const UNREGISTER_ID: &str = "unregister";
/// Toggle dev mode — when on, the menu window stays open on focus loss (CheckMenuItem).
pub const DEV_ID: &str = "dev";
/// Toggle icon ribbon visibility (CheckMenuItem).
pub const ICONS_ID: &str = "icons";
/// Toggle autostart — when on, the app launches at Windows startup (CheckMenuItem).
pub const AUTOSTART_ID: &str = "autostart";
/// Reset all config and menu files to embedded defaults (MenuItem).
pub const RESET_ID: &str = "reset";
/// Exit the application (MenuItem).
pub const QUIT_ID: &str = "quit";
/// Open the config editor window (MenuItem).
pub const CONFIG_ID: &str = "config";
/// Download the latest menu JS from the configured remote URL (MenuItem).
pub const SYNC_ID: &str = "sync";

// ── Label constants ──────────────────────────────────────────────────────

pub const QUIT_TEXT: &str = "Quit";
pub const WIN11_TEXT: &str = "Win11";
pub const CLASSIC_TEXT: &str = "Classic";
pub const REGISTER_TEXT: &str = "Register";
pub const UNREGISTER_TEXT: &str = "Unregister";
pub const DEV_TEXT: &str = "Dev";
pub const ICONS_TEXT: &str = "Icons";
pub const AUTOSTART_TEXT: &str = "Startup";
pub const RESET_TEXT: &str = "Reset";
pub const APPLY_TEXT: &str = "Apply";
pub const SYNC_TEXT: &str = "Sync";
pub const CONFIG_TEXT: &str = "Config";

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn is_win11() -> bool {
    MenuStyle::current() == MenuStyle::Windows11
}

fn register_status() -> bool {
    rcm_com::cmd::status()
        .map(|s| s.is_valid())
        .unwrap_or(false)
}

fn sync_style_checks<R: tauri::Runtime>(win11: &CheckMenuItem<R>, classic: &CheckMenuItem<R>) {
    let win11_active = is_win11();
    let _ = win11.set_checked(win11_active);
    let _ = classic.set_checked(!win11_active);
}

// ═══════════════════════════════════════════════════════════════════════════
// Event handlers
// ═══════════════════════════════════════════════════════════════════════════

fn handle_style_switch<R: tauri::Runtime>(
    style: MenuStyle,
    win11: &CheckMenuItem<R>,
    classic: &CheckMenuItem<R>,
) {
    if let Err(e) = style.set() {
        log::error("Tray", &format!("set {style:?} style failed: {e}"));
    }
    sync_style_checks(win11, classic);
}

fn handle_register_toggle<R: tauri::Runtime>(register: bool, item: &CheckMenuItem<R>) {
    if register {
        let _ = rcm_com::cmd::register();
    } else {
        let _ = rcm_com::cmd::unregister();
    }
    let _ = item.set_checked(register_status());
}

fn handle_icons_toggle<R: tauri::Runtime>(app: &tauri::AppHandle<R>, item: &CheckMenuItem<R>) {
    let val = !config::is_icons();
    config::set_icons(val);
    let _ = item.set_checked(val);
    let _ = app.emit("icons-changed", val);
}

fn handle_dev_toggle<R: tauri::Runtime>(app: &tauri::AppHandle<R>, item: &CheckMenuItem<R>) {
    let val = !config::is_dev();
    config::set_dev(val);
    let _ = item.set_checked(val);
    let _ = app.emit("dev-mode", val);
}

fn handle_autostart_toggle<R: tauri::Runtime>(item: &CheckMenuItem<R>) {
    let (ok, enabled) = if registry::is_autostart_enabled() {
        (registry::disable_autostart().is_ok(), false)
    } else {
        (registry::enable_autostart().is_ok(), true)
    };
    if ok {
        let _ = item.set_checked(enabled);
        log::info(
            "Tray",
            if enabled {
                "autostart enabled"
            } else {
                "autostart disabled"
            },
        );
    } else {
        log::error(
            "Tray",
            if enabled {
                "enable autostart failed"
            } else {
                "disable autostart failed"
            },
        );
    }
}

fn handle_sync() {
    match rcm_core::config::remote_url() {
        Some(url) => {
            log::info("Tray", &format!("syncing menu from {url}"));
            match rcm_core::menu::download_menu(&url) {
                Ok(path) => log::info("Tray", &format!("sync saved to {path}")),
                Err(e) => log::error("Tray", &format!("sync failed: {e}")),
            }
        }
        None => log::error("Tray", "sync: no remote URL configured"),
    }
}

fn handle_apply() {
    if let Err(e) = rcm_reg::restart_explorer(Duration::from_secs(3)) {
        log::error("Tray", &format!("restart Explorer failed: {e}"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tray setup
// ═══════════════════════════════════════════════════════════════════════════

pub fn setup_tray(app: &mut App) -> Result<(), tauri::Error> {
    // ── Create menu items ────────────────────────────────────────────

    let win11_i = CheckMenuItem::with_id(
        app,
        WIN11_STYLE_ID,
        WIN11_TEXT,
        true,
        is_win11(),
        None::<&str>,
    )?;
    let classic_i = CheckMenuItem::with_id(
        app,
        CLASSIC_STYLE_ID,
        CLASSIC_TEXT,
        true,
        !is_win11(),
        None::<&str>,
    )?;
    let register_i = CheckMenuItem::with_id(
        app,
        REGISTER_ID,
        REGISTER_TEXT,
        true,
        register_status(),
        None::<&str>,
    )?;
    let unregister_i = MenuItem::with_id(app, UNREGISTER_ID, UNREGISTER_TEXT, true, None::<&str>)?;
    let icons_i = CheckMenuItem::with_id(
        app,
        ICONS_ID,
        ICONS_TEXT,
        true,
        config::is_icons(),
        None::<&str>,
    )?;
    let dev_i =
        CheckMenuItem::with_id(app, DEV_ID, DEV_TEXT, true, config::is_dev(), None::<&str>)?;
    let autostart_i = CheckMenuItem::with_id(
        app,
        AUTOSTART_ID,
        AUTOSTART_TEXT,
        true,
        registry::is_autostart_enabled(),
        None::<&str>,
    )?;
    let sync_i = MenuItem::with_id(app, SYNC_ID, SYNC_TEXT, true, None::<&str>)?;
    let config_i = MenuItem::with_id(app, CONFIG_ID, CONFIG_TEXT, true, None::<&str>)?;
    let reset_i = MenuItem::with_id(app, RESET_ID, RESET_TEXT, true, None::<&str>)?;
    let apply_i = MenuItem::with_id(app, APPLY_ID, APPLY_TEXT, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, QUIT_ID, QUIT_TEXT, true, None::<&str>)?;

    // ── Clones for event handler ─────────────────────────────────────

    let win11_clone = win11_i.clone();
    let classic_clone = classic_i.clone();
    let register_clone = register_i.clone();
    let dev_clone = dev_i.clone();
    let icons_clone = icons_i.clone();
    let autostart_clone = autostart_i.clone();

    // ── Build menu (3 groups) ────────────────────────────────────────
    //
    //   ✓ Win11 / Classic          ← Style
    //   ─────────
    //     Register / Unregister    ← Preferences
    //   ✓ Icons  (debug)
    //   ✓ Dev    (debug)
    //   ✓ Auto Start
    //   ─────────
    //     Sync Menu  (conditional)    ← System
    //     Reset / Apply / Quit

    let is_debug = cfg!(debug_assertions);
    let has_remote = rcm_core::config::remote_url().is_some();

    let _sep_prefs = PredefinedMenuItem::separator(app)?;
    let _sep_sys = PredefinedMenuItem::separator(app)?;

    // Group 1: Style
    let mut items: Vec<&dyn tauri::menu::IsMenuItem<_>> = vec![
        &win11_i,
        &classic_i,
        // Group 2: Preferences
        &_sep_prefs,
        &register_i,
        &unregister_i,
    ];

    if is_debug {
        items.push(&icons_i);
        items.push(&dev_i);
    }
    items.push(&autostart_i);

    // Group 3: System
    items.push(&_sep_sys);
    if has_remote {
        items.push(&sync_i);
    }
    items.push(&config_i);
    items.push(&reset_i);
    items.push(&apply_i);
    items.push(&quit_i);

    let menu = Menu::with_items(app, &items)?;

    // ── Build tray ──────────────────────────────────────────────────

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            QUIT_ID => app.exit(0),
            WIN11_STYLE_ID => {
                handle_style_switch(MenuStyle::Windows11, &win11_clone, &classic_clone)
            }
            CLASSIC_STYLE_ID => {
                handle_style_switch(MenuStyle::Classic, &win11_clone, &classic_clone)
            }
            REGISTER_ID => handle_register_toggle(true, &register_clone),
            UNREGISTER_ID => handle_register_toggle(false, &register_clone),
            ICONS_ID => handle_icons_toggle(app, &icons_clone),
            DEV_ID => handle_dev_toggle(app, &dev_clone),
            AUTOSTART_ID => handle_autostart_toggle(&autostart_clone),
            APPLY_ID => handle_apply(),
            SYNC_ID => handle_sync(),
            CONFIG_ID => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::create_config_window(app_handle).await;
                });
            }
            RESET_ID => {
                config::reset();
                crate::write_style_defaults();
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
