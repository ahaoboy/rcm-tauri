use crate::registry::*;
use std::io::Write;
use tauri::{
    App,
    Emitter,
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
/// Dump all environment variables to a .env file next to the exe (MenuItem).
pub const DUMP_ENV_ID: &str = "dump_env";
/// Toggle dev mode — when on, the menu window stays open on focus loss (CheckMenuItem).
pub const DEV_ID: &str = "dev";
/// Switch to lite menu (CheckMenuItem).
pub const MENU_LITE_ID: &str = "menu_lite";
/// Switch to full menu (CheckMenuItem).
pub const MENU_FULL_ID: &str = "menu_full";
/// Toggle icon ribbon visibility (CheckMenuItem).
pub const ICONS_ID: &str = "icons";
/// Reset all config and menu files to embedded defaults (MenuItem).
pub const RESET_ID: &str = "reset";
/// Exit the application (MenuItem).
pub const QUIT_ID: &str = "quit";

// ── Label constants ──────────────────────────────────────────────────────

pub const QUIT_TEXT: &str = "Quit";
pub const ENABLE_TEXT: &str = "Enable";
pub const DISABLE_TEXT: &str = "Disable";
pub const WIN11_TEXT: &str = "Win11";
pub const CLASSIC_TEXT: &str = "Classic";
pub const REGISTER_TEXT: &str = "Register";
pub const UNREGISTER_TEXT: &str = "Unregister";
pub const DUMP_ENV_TEXT: &str = "Dump Env";
pub const DEV_TEXT: &str = "Dev Mode";
pub const MENU_LITE_TEXT: &str = "Lite Menu";
pub const MENU_FULL_TEXT: &str = "Full Menu";
pub const ICONS_TEXT: &str = "Icons";
pub const RESET_TEXT: &str = "Reset to defaults";
pub const APPLY_TEXT: &str = "Apply";

// ── Helpers ──────────────────────────────────────────────────────────────

fn get_toggle_text(is_enabled: bool) -> &'static str {
    if is_enabled {
        DISABLE_TEXT
    } else {
        ENABLE_TEXT
    }
}

/// Determine the active menu style using rcm-com's `get_menu_style()`.
fn current_is_win11() -> bool {
    rcm_com::get_menu_style() == "Win11"
}

/// Synchronise both style CheckMenuItems so only one is checked at a time.
fn sync_style_checks<R: tauri::Runtime>(win11: &CheckMenuItem<R>, classic: &CheckMenuItem<R>) {
    let is_win11 = current_is_win11();
    let _ = win11.set_checked(is_win11);
    let _ = classic.set_checked(!is_win11);
}

/// Write all current process environment variables to `<exe_path>.env`.
fn dump_env() {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("dump_env: failed to get exe path: {e}");
            return;
        }
    };

    let env_path = {
        let mut p = exe.clone();
        p.set_extension("exe.env");
        p
    };

    let mut vars: Vec<(String, String)> = std::env::vars().collect();
    vars.sort_by(|a, b| a.0.cmp(&b.0));

    let mut file = match std::fs::File::create(&env_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("dump_env: failed to create {}: {}", env_path.display(), e);
            return;
        }
    };

    for (key, value) in &vars {
        let _ = writeln!(file, "{key}={value}");
    }

    println!("dump_env: wrote {} vars to {}", vars.len(), env_path.display());
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
    let is_ctx_enabled = !get_context_menu_status();
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
    let dump_env_i = MenuItem::with_id(app, DUMP_ENV_ID, DUMP_ENV_TEXT, true, None::<&str>)?;
    let menu_lite_i = CheckMenuItem::with_id(app, MENU_LITE_ID, MENU_LITE_TEXT, true, crate::config::is_lite(), None::<&str>)?;
    let menu_full_i = CheckMenuItem::with_id(app, MENU_FULL_ID, MENU_FULL_TEXT, true, !crate::config::is_lite(), None::<&str>)?;
    let dev_i = CheckMenuItem::with_id(app, DEV_ID, DEV_TEXT, true, crate::config::is_dev(), None::<&str>)?;
    let icons_i = CheckMenuItem::with_id(app, ICONS_ID, ICONS_TEXT, true, crate::config::is_icons(), None::<&str>)?;
    let reset_i = MenuItem::with_id(app, RESET_ID, RESET_TEXT, true, None::<&str>)?;
    let apply_i = MenuItem::with_id(app, APPLY_ID, APPLY_TEXT, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, QUIT_ID, QUIT_TEXT, true, None::<&str>)?;

    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;
    let sep5 = PredefinedMenuItem::separator(app)?;
    let sep6 = PredefinedMenuItem::separator(app)?;

    // ── Clones for the event handler ─────────────────────────────────
    let win11_clone = win11_i.clone();
    let classic_clone = classic_i.clone();
    let toggle_ctx_clone = toggle_ctx_i.clone();
    let dev_clone = dev_i.clone();
    let menu_lite_clone = menu_lite_i.clone();
    let menu_full_clone = menu_full_i.clone();
    let icons_clone = icons_i.clone();

    // ── Build menu ───────────────────────────────────────────────────
    // Layout:
    //   ✓ Win11
    //   ✓ 经典样式 (Classic)
    //   ─────────
    //   ✓ Enable / Disable
    //   ─────────
    //     Register
    //     Unregister
    //     Dump Env
    //   ─────────
    //   ✓ Lite Menu
    //   ✓ Full Menu
    //   ✓ Icons
    //   ✓ Dev Mode
    //   ─────────
    //     Apply
    //     Reset to defaults
    //     Quit
    let menu = Menu::with_items(
        app,
        &[
            &win11_i,
            &classic_i,
            &sep1,
            &toggle_ctx_i,
            &sep2,
            &register_i,
            &unregister_i,
            &dump_env_i,
            &sep3,
            &menu_lite_i,
            &menu_full_i,
            &icons_i,
            &sep4,
            &dev_i,
            &sep5,
            &apply_i,
            &sep6,
            &reset_i,
            &quit_i,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
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
                    let _ = rcm_com::set_win11_menu_style(false);
                    sync_style_checks(&win11_clone, &classic_clone);
                }
                CLASSIC_STYLE_ID => {
                    // Switch to classic Windows 10 expanded menu
                    let _ = rcm_com::set_win11_menu_style(true);
                    sync_style_checks(&win11_clone, &classic_clone);
                }

                // ── Toggle RCM context menu ───────────────────────
                TOGGLE_CTX_ID => {
                    let current_status = get_context_menu_status();
                    if current_status {
                        let _ = disable_context_menu();
                    } else {
                        let _ = enable_context_menu();
                    }

                    let new_state = !get_context_menu_status();
                    let _ = toggle_ctx_clone.set_text(get_toggle_text(new_state));
                    let _ = toggle_ctx_clone.set_checked(new_state);

                    restart_explorer();
                }

                // ── Register / Unregister shell extension ────────
                REGISTER_ID => {
                    let _ = rcm_com::cmd::register();
                }
                UNREGISTER_ID => {
                    let _ = rcm_com::cmd::unregister();
                }

                // ── Dump environment variables ────────────────────
                DUMP_ENV_ID => {
                    dump_env();
                }

                // ── Menu mode switching ─────────────────────────
                MENU_LITE_ID => {
                    crate::config::set_lite(true);
                    let _ = menu_lite_clone.set_checked(true);
                    let _ = menu_full_clone.set_checked(false);
                }
                MENU_FULL_ID => {
                    crate::config::set_lite(false);
                    let _ = menu_lite_clone.set_checked(false);
                    let _ = menu_full_clone.set_checked(true);
                }

                // ── Toggle icons ────────────────────────────────
                ICONS_ID => {
                    let new_val = !crate::config::is_icons();
                    crate::config::set_icons(new_val);
                    let _ = icons_clone.set_checked(new_val);
                    match app.emit("icons-changed", new_val) {
                        Ok(()) => println!("icons-changed: emitted {new_val}"),
                        Err(e) => eprintln!("icons-changed: emit failed: {e}"),
                    }
                }

                // ── Toggle dev mode ──────────────────────────────
                DEV_ID => {
                    let new_val = !crate::config::is_dev();
                    crate::config::set_dev(new_val);
                    let _ = dev_clone.set_checked(new_val);
                    let _ = app.emit("dev-mode", new_val);
                }

                // ── Apply (restart Explorer) ─────────────────────
                APPLY_ID => {
                    let _ = rcm_com::restart_explorer();
                }

                // ── Reset to defaults ────────────────────────────
                RESET_ID => {
                    crate::config::reset();
                }

                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
