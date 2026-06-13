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

/// Toggle the RCM context menu on/off (CheckMenuItem).
pub const TOGGLE_CTX_ID: &str = "toggle_ctx";
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
/// Download the latest menu JS from the configured remote URL (MenuItem).
pub const UPDATE_ID: &str = "update";

// ── Label constants ──────────────────────────────────────────────────────

pub const QUIT_TEXT: &str = "Quit";
pub const ENABLE_TEXT: &str = "Enable";
pub const DISABLE_TEXT: &str = "Disable";
pub const WIN11_TEXT: &str = "Win11";
pub const CLASSIC_TEXT: &str = "Classic";
pub const REGISTER_TEXT: &str = "Register";
pub const UNREGISTER_TEXT: &str = "Unregister";
pub const DEV_TEXT: &str = "Dev";
pub const ICONS_TEXT: &str = "Icons";
pub const AUTOSTART_TEXT: &str = "Startup";
pub const RESET_TEXT: &str = "Reset";
pub const APPLY_TEXT: &str = "Apply";
pub const UPDATE_TEXT: &str = "Update";

// ── Helpers ──────────────────────────────────────────────────────────────

fn get_toggle_text(is_enabled: bool) -> &'static str {
    if is_enabled {
        DISABLE_TEXT
    } else {
        ENABLE_TEXT
    }
}

/// Determine the active menu style using rcm-reg's `MenuStyle`.
fn current_is_win11() -> bool {
    MenuStyle::current() == MenuStyle::Windows11
}

/// Synchronise both style CheckMenuItems so only one is checked at a time.
fn sync_style_checks<R: tauri::Runtime>(win11: &CheckMenuItem<R>, classic: &CheckMenuItem<R>) {
    let is_win11 = current_is_win11();
    let _ = win11.set_checked(is_win11);
    let _ = classic.set_checked(!is_win11);
}

// ═══════════════════════════════════════════════════════════════════════════
// Tray setup
// ═══════════════════════════════════════════════════════════════════════════

pub fn setup_tray(app: &mut App) -> Result<(), tauri::Error> {
    // ── Style items ──────────────────────────────────────────────────
    let is_win11 = current_is_win11();

    let win11_i = CheckMenuItem::with_id(
        app,
        WIN11_STYLE_ID,
        WIN11_TEXT,
        true,
        is_win11,
        None::<&str>,
    )?;

    let classic_i = CheckMenuItem::with_id(
        app,
        CLASSIC_STYLE_ID,
        CLASSIC_TEXT,
        true,
        !is_win11,
        None::<&str>,
    )?;

    // ── Toggle context menu item ─────────────────────────────────────
    let is_ctx_enabled = !registry::get_context_menu_status();
    let toggle_ctx_i = CheckMenuItem::with_id(
        app,
        TOGGLE_CTX_ID,
        get_toggle_text(is_ctx_enabled),
        true,
        is_ctx_enabled,
        None::<&str>,
    )?;

    // ── Action items ─────────────────────────────────────────────────
    let register_i = MenuItem::with_id(app, REGISTER_ID, REGISTER_TEXT, true, None::<&str>)?;
    let unregister_i = MenuItem::with_id(app, UNREGISTER_ID, UNREGISTER_TEXT, true, None::<&str>)?;
    let dev_i =
        CheckMenuItem::with_id(app, DEV_ID, DEV_TEXT, true, config::is_dev(), None::<&str>)?;
    let icons_i = CheckMenuItem::with_id(
        app,
        ICONS_ID,
        ICONS_TEXT,
        true,
        config::is_icons(),
        None::<&str>,
    )?;
    let autostart_i = CheckMenuItem::with_id(
        app,
        AUTOSTART_ID,
        AUTOSTART_TEXT,
        true,
        registry::is_autostart_enabled(),
        None::<&str>,
    )?;
    let update_i = MenuItem::with_id(app, UPDATE_ID, UPDATE_TEXT, true, None::<&str>)?;
    let reset_i = MenuItem::with_id(app, RESET_ID, RESET_TEXT, true, None::<&str>)?;
    let apply_i = MenuItem::with_id(app, APPLY_ID, APPLY_TEXT, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, QUIT_ID, QUIT_TEXT, true, None::<&str>)?;

    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;
    let sep5 = PredefinedMenuItem::separator(app)?;
    let sep6 = PredefinedMenuItem::separator(app)?;
    let sep7 = PredefinedMenuItem::separator(app)?;
    let sep8 = PredefinedMenuItem::separator(app)?;

    // ── Clones for the event handler ─────────────────────────────────
    let win11_clone = win11_i.clone();
    let classic_clone = classic_i.clone();
    let toggle_ctx_clone = toggle_ctx_i.clone();
    let dev_clone = dev_i.clone();
    let icons_clone = icons_i.clone();
    let autostart_clone = autostart_i.clone();

    // ── Build menu ───────────────────────────────────────────────────
    // Layout:
    //   ✓ Win11 / Classic
    //   ─────────
    //   ✓ Enable / Disable
    //   ─────────
    //     Register / Unregister
    //   ─────────
    //   ✓ Icons
    //   ─────────
    //   ✓ Dev Mode
    //   ─────────
    //   ✓ Auto Start
    //   ─────────
    //     Update          (only if remote_url is set in config)
    //   ─────────
    //     Apply / Reset / Quit

    let has_remote = rcm_core::config::remote_url().is_some();

    let mut items: Vec<&dyn tauri::menu::IsMenuItem<_>> = vec![
        &win11_i,
        &classic_i,
        &sep1,
        &toggle_ctx_i,
        &sep2,
        &register_i,
        &unregister_i,
        &sep3,
        &icons_i,
        &sep4,
        &dev_i,
        &sep5,
        &autostart_i,
    ];

    if has_remote {
        items.push(&sep6);
        items.push(&update_i);
        items.push(&sep7);
    } else {
        items.push(&sep6);
    }

    items.push(&apply_i);
    items.push(&sep8);
    items.push(&reset_i);
    items.push(&quit_i);

    let menu = Menu::with_items(app, &items)?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                QUIT_ID => {
                    app.exit(0);
                }

                // ── Style switching ───────────────────────────────
                WIN11_STYLE_ID => {
                    // Switch to Windows 11 compact menu
                    if let Err(e) = MenuStyle::Windows11.set() {
                        log::error("Tray", &format!("set Win11 style failed: {e}"));
                    }
                    sync_style_checks(&win11_clone, &classic_clone);
                }
                CLASSIC_STYLE_ID => {
                    // Switch to classic Windows 10 expanded menu
                    if let Err(e) = MenuStyle::Classic.set() {
                        log::error("Tray", &format!("set Classic style failed: {e}"));
                    }
                    sync_style_checks(&win11_clone, &classic_clone);
                }

                // ── Toggle RCM context menu ───────────────────────
                TOGGLE_CTX_ID => {
                    let current_status = registry::get_context_menu_status();
                    if current_status {
                        let _ = registry::disable_context_menu();
                    } else {
                        let _ = registry::enable_context_menu();
                    }

                    let new_state = !registry::get_context_menu_status();
                    let _ = toggle_ctx_clone.set_text(get_toggle_text(new_state));
                    let _ = toggle_ctx_clone.set_checked(new_state);

                    registry::restart_explorer();
                }

                // ── Register / Unregister shell extension ────────
                REGISTER_ID => {
                    let _ = rcm_com::cmd::register();
                }
                UNREGISTER_ID => {
                    let _ = rcm_com::cmd::unregister();
                }

                // ── Toggle icons ────────────────────────────────
                ICONS_ID => {
                    let new_val = !config::is_icons();
                    config::set_icons(new_val);
                    let _ = icons_clone.set_checked(new_val);
                    match app.emit("icons-changed", new_val) {
                        Ok(()) => println!("icons-changed: emitted {new_val}"),
                        Err(e) => eprintln!("icons-changed: emit failed: {e}"),
                    }
                }

                // ── Toggle dev mode ──────────────────────────────
                DEV_ID => {
                    let new_val = !config::is_dev();
                    config::set_dev(new_val);
                    let _ = dev_clone.set_checked(new_val);
                    let _ = app.emit("dev-mode", new_val);
                }

                // ── Toggle autostart ──────────────────────────
                AUTOSTART_ID => {
                    if registry::is_autostart_enabled() {
                        match registry::disable_autostart() {
                            Ok(()) => {
                                let _ = autostart_clone.set_checked(false);
                                log::info("Tray", "autostart disabled");
                            }
                            Err(e) => log::error("Tray", &format!("disable autostart failed: {e}")),
                        }
                    } else {
                        match registry::enable_autostart() {
                            Ok(()) => {
                                let _ = autostart_clone.set_checked(true);
                                log::info("Tray", "autostart enabled");
                            }
                            Err(e) => log::error("Tray", &format!("enable autostart failed: {e}")),
                        }
                    }
                }

                // ── Apply (restart Explorer) ─────────────────────
                APPLY_ID => {
                    if let Err(e) = rcm_reg::restart_explorer(Duration::from_secs(3)) {
                        log::error("Tray", &format!("restart Explorer failed: {e}"));
                    }
                }

                // ── Update menu from remote URL ─────────────────
                UPDATE_ID => {
                    match rcm_core::config::remote_url() {
                        Some(url) => {
                            log::info("Tray", &format!("updating menu from {url}"));
                            match rcm_core::menu_defaults::download_menu(&url) {
                                Ok(path) => log::info("Tray", &format!("update saved to {path}")),
                                Err(e) => log::error("Tray", &format!("update failed: {e}")),
                            }
                        }
                        None => {
                            log::error("Tray", "update: no remote URL configured");
                        }
                    }
                }

                // ── Reset to defaults ────────────────────────────
                RESET_ID => {
                    config::reset();
                }

                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
