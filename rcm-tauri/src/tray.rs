//! System-tray integration.
//!
//! Deliberately minimal: the tray holds only the two actions that have to work
//! when no window is open. Everything else it used to expose — style, blocking,
//! theme, autostart, the remote pulls — now lives in the config window, which has
//! room to label and explain each one.
//!
//! The menu is therefore fixed after setup: nothing to keep a handle for, and
//! nothing to resynchronise when a setting changes.

use rcm_core::actions::{self, ids, text};
use rcm_core::log;
use tauri::{
    App,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

/// Build the tray icon and its menu.
pub fn setup_tray(app: &mut App) -> Result<(), tauri::Error> {
    let config_i = MenuItem::with_id(app, ids::CONFIG, text::CONFIG, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, ids::QUIT, text::QUIT, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&config_i, &quit_i])?;

    let mut builder = TrayIconBuilder::new()
        .tooltip(app.config().product_name.as_deref().unwrap_or("rcm-tauri"))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            ids::CONFIG => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::create_config_window(app_handle).await {
                        log::error("Tray", &e);
                    }
                });
            }
            ids::QUIT => {
                // Disable blocking first: a shell extension still suppressing the
                // native menu with no process behind it would leave the user with
                // no context menu at all.
                let _ = actions::shutdown();
                app.exit(0);
            }
            _ => {}
        });

    // Missing icon is not fatal here: without one the tray entry is simply
    // invisible, which is worth a log rather than a panic on startup.
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    } else {
        log::warn(
            "Tray",
            "no window icon configured; the tray entry may be blank",
        );
    }

    builder.build(app)?;
    Ok(())
}
