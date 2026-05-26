use crate::registry::*;
use tauri::{
    App,
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
/// Exit the application (MenuItem).
pub const QUIT_ID: &str = "quit";

// ── Label constants ──────────────────────────────────────────────────────

pub const QUIT_TEXT: &str = "Quit";
pub const ENABLE_TEXT: &str = "Enable";
pub const DISABLE_TEXT: &str = "Disable";
pub const WIN11_TEXT: &str = "Win11";
pub const CLASSIC_TEXT: &str = "Classic";
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
    let apply_i = MenuItem::with_id(app, APPLY_ID, APPLY_TEXT, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, QUIT_ID, QUIT_TEXT, true, None::<&str>)?;

    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    // ── Clones for the event handler ─────────────────────────────────
    let win11_clone = win11_i.clone();
    let classic_clone = classic_i.clone();
    let toggle_ctx_clone = toggle_ctx_i.clone();

    // ── Build menu ───────────────────────────────────────────────────
    // Layout:
    //   ✓ Win11
    //   ✓ 经典样式 (Classic)
    //   ─────────
    //   ✓ Enable / Disable
    //   ─────────
    //     Apply
    //     Quit
    let menu = Menu::with_items(
        app,
        &[
            &win11_i,
            &classic_i,
            &sep1,
            &toggle_ctx_i,
            &sep2,
            &apply_i,
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

                // ── Apply (restart Explorer) ─────────────────────
                APPLY_ID => {
                    let _ = rcm_com::restart_explorer();
                }

                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
