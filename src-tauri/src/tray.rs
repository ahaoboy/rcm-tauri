use crate::registry::*;
use tauri::{
    App,
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::TrayIconBuilder,
};

pub const TOGGLE_CTX_ID: &str = "toggle_ctx";
pub const QUIT_ID: &str = "quit";
pub const QUIT_TEXT: &str = "Quit";
pub const ENABLE_TEXT: &str = "Enable";
pub const DISABLE_TEXT: &str = "Disable";

fn get_toggle_text(is_enabled: bool) -> &'static str {
    if is_enabled {
        DISABLE_TEXT
    } else {
        ENABLE_TEXT
    }
}

pub fn setup_tray(app: &mut App) -> Result<(), tauri::Error> {
    let quit_i = MenuItem::with_id(app, QUIT_ID, QUIT_TEXT, true, None::<&str>)?;

    let is_enabled = !get_context_menu_status();
    let toggle_ctx_i = CheckMenuItem::with_id(
        app,
        TOGGLE_CTX_ID,
        get_toggle_text(is_enabled),
        true,
        is_enabled,
        None::<&str>,
    )?;
    let toggle_ctx_clone = toggle_ctx_i.clone();

    let menu = Menu::with_items(app, &[&toggle_ctx_i, &quit_i])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| {
            if event.id().as_ref() == QUIT_ID {
                app.exit(0);
            } else if event.id().as_ref() == TOGGLE_CTX_ID {
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
        })
        .build(app)?;

    Ok(())
}
