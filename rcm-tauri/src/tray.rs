//! System-tray integration.
//!
//! Builds the native tray menu and keeps its checkmarks in sync. All *system*
//! behaviour lives in [`rcm_core::actions`], shared with the Reactor build, so
//! this module is only the Tauri presentation: which item to tick, and which
//! event to emit to the frontend.

use std::sync::{Mutex, OnceLock};

use rcm_core::actions::{self, ids, text};
use rcm_core::config;
use rcm_core::log;
use tauri::{
    App, Emitter,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{TrayIcon, TrayIconBuilder},
};

// ═══════════════════════════════════════════════════════════════════════════
// Event handlers
// ═══════════════════════════════════════════════════════════════════════════

/// Switch the context-menu style and relabel the entry.
///
/// The entry names the style clicking selects, so it has to be relabelled after
/// every switch — otherwise it would keep offering the style just chosen.
fn handle_style_toggle<R: tauri::Runtime>(item: &MenuItem<R>) {
    match actions::toggle_style() {
        Ok(_) => {
            let _ = item.set_text(actions::style_entry_label());
        }
        Err(e) => log::error("Tray", &e),
    }
}

/// Toggle native context-menu blocking and relabel the entry.
fn handle_blocking_toggle<R: tauri::Runtime>(item: &MenuItem<R>) {
    match actions::toggle_blocking() {
        Ok(_) => {
            let _ = item.set_text(actions::blocking_entry_label());
        }
        Err(e) => log::error("Tray", &e),
    }
}

fn handle_register_toggle<R: tauri::Runtime>(item: &MenuItem<R>) {
    let _ = item.set_text(actions::register_entry_label());
    let _ = actions::toggle_registered();
}

fn handle_icons_toggle<R: tauri::Runtime>(app: &tauri::AppHandle<R>, item: &CheckMenuItem<R>) {
    let value = actions::toggle_icons();
    let _ = item.set_checked(value);
    let _ = app.emit("icons-changed", value);
}

fn handle_dev_toggle<R: tauri::Runtime>(app: &tauri::AppHandle<R>, item: &CheckMenuItem<R>) {
    let value = actions::toggle_dev();
    let _ = item.set_checked(value);
    let _ = app.emit("dev-mode", value);
}

fn handle_autostart_toggle<R: tauri::Runtime>(item: &CheckMenuItem<R>) {
    if let Ok(value) = actions::toggle_autostart() {
        let _ = item.set_checked(value);
    }
}

fn handle_theme<R: tauri::Runtime>(
    theme: config::Theme,
    app: &tauri::AppHandle<R>,
    sys: &CheckMenuItem<R>,
    light: &CheckMenuItem<R>,
    dark: &CheckMenuItem<R>,
) {
    let theme = actions::set_theme(theme);
    let _ = sys.set_checked(theme == config::Theme::System);
    let _ = light.set_checked(theme == config::Theme::Light);
    let _ = dark.set_checked(theme == config::Theme::Dark);
    let _ = app.emit("theme-changed", theme.as_str());
}

fn handle_apply() {
    let _ = actions::apply();
}

// ═══════════════════════════════════════════════════════════════════════════
// Tray setup
// ═══════════════════════════════════════════════════════════════════════════

/// The app handle, kept so the tray can be rebuilt after a settings change.
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();

/// The live tray icon, replaced on every rebuild.
static TRAY: Mutex<Option<TrayIcon>> = Mutex::new(None);

/// Install the tray icon.
pub fn setup_tray(app: &mut App) -> Result<(), tauri::Error> {
    let _ = APP.set(app.handle().clone());
    install()
}

/// Rebuild the tray so it matches the current settings.
///
/// The tray mirrors settings the settings tab can also change, and a menu item's
/// label cannot be changed after its menu was built, so the tray is replaced
/// wholesale. That keeps a single definition of "the tray matches the
/// settings", rather than a second one that resyncs items in place and can drift
/// from the first.
///
/// A no-op before [`setup_tray`] has run.
pub fn refresh() {
    if let Err(e) = install() {
        log::error("Tray", &format!("refresh failed: {e}"));
    }
}

fn install() -> Result<(), tauri::Error> {
    let handle = match APP.get() {
        Some(handle) => handle.clone(),
        // Before setup there is no handle to build menu items with.
        None => return Ok(()),
    };
    let app = &handle;

    // ── Create menu items ───────────────────────────────────────────
    //
    // Style, registration and blocking are each a single entry: they are two
    // states of one setting, so showing both (one ticked) only repeats what the
    // label can say on its own.
    let style_i = MenuItem::with_id(
        app,
        ids::STYLE_TOGGLE,
        actions::style_entry_label(),
        true,
        None::<&str>,
    )?;
    let register_i = MenuItem::with_id(
        app,
        ids::REGISTER_TOGGLE,
        actions::register_entry_label(),
        true,
        None::<&str>,
    )?;
    let blocking_i = MenuItem::with_id(
        app,
        ids::BLOCKING_TOGGLE,
        actions::blocking_entry_label(),
        true,
        None::<&str>,
    )?;

    let theme_sys_i = CheckMenuItem::with_id(
        app,
        ids::THEME_SYSTEM,
        text::THEME_SYSTEM,
        true,
        config::theme() == config::Theme::System,
        None::<&str>,
    )?;
    let theme_light_i = CheckMenuItem::with_id(
        app,
        ids::THEME_LIGHT,
        text::THEME_LIGHT,
        true,
        config::theme() == config::Theme::Light,
        None::<&str>,
    )?;
    let theme_dark_i = CheckMenuItem::with_id(
        app,
        ids::THEME_DARK,
        text::THEME_DARK,
        true,
        config::theme() == config::Theme::Dark,
        None::<&str>,
    )?;
    let icons_i = CheckMenuItem::with_id(
        app,
        ids::ICONS,
        text::ICONS,
        true,
        config::is_icons(),
        None::<&str>,
    )?;
    let dev_i = CheckMenuItem::with_id(
        app,
        ids::DEV,
        text::DEV,
        true,
        config::is_dev(),
        None::<&str>,
    )?;
    let autostart_i = CheckMenuItem::with_id(
        app,
        ids::AUTOSTART,
        text::AUTOSTART,
        true,
        rcm_core::registry::is_autostart_enabled(),
        None::<&str>,
    )?;
    let config_i = MenuItem::with_id(app, ids::CONFIG, text::CONFIG, true, None::<&str>)?;
    let reset_i = MenuItem::with_id(app, ids::RESET, text::RESET, true, None::<&str>)?;
    let apply_i = MenuItem::with_id(app, ids::APPLY, text::APPLY, true, None::<&str>)?;
    let about_i = MenuItem::with_id(app, ids::ABOUT, text::ABOUT, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, ids::QUIT, text::QUIT, true, None::<&str>)?;

    // ── Clones for the event handler ─────────────────────────────────
    let style_clone = style_i.clone();
    let blocking_clone = blocking_i.clone();
    let register_clone = register_i.clone();
    let dev_clone = dev_i.clone();
    let icons_clone = icons_i.clone();
    let autostart_clone = autostart_i.clone();
    let theme_sys_clone = theme_sys_i.clone();
    let theme_light_clone = theme_light_i.clone();
    let theme_dark_clone = theme_dark_i.clone();

    // ── Build the menu (3 groups) ────────────────────────────────────
    //
    //   Classic                    ← Style (click switches to the other)
    //   ─────────
    //     Register                 ← Registration (click switches)
    //   Disable                    ← Blocking (click switches to the other)
    //   ✓ Icons  (debug)
    //   ✓ Dev    (debug)
    //   ✓ Auto Start
    //   Theme ▸
    //   ─────────
    //   Config / Reset / Apply / About / Quit

    let is_debug = cfg!(debug_assertions);
    let separator_prefs = PredefinedMenuItem::separator(app)?;
    let separator_system = PredefinedMenuItem::separator(app)?;

    let theme_menu = Submenu::with_items(
        app,
        text::THEME,
        true,
        &[&theme_sys_i, &theme_light_i, &theme_dark_i],
    )?;

    let mut items: Vec<&dyn tauri::menu::IsMenuItem<_>> = vec![
        &style_i,
        &separator_prefs,
        &register_i,
        &blocking_i,
    ];

    if is_debug {
        items.push(&icons_i);
        items.push(&dev_i);
    }
    items.push(&autostart_i);
    items.push(&theme_menu);

    items.push(&separator_system);
    items.push(&config_i);
    items.push(&reset_i);
    items.push(&apply_i);
    items.push(&about_i);
    items.push(&quit_i);

    let menu = Menu::with_items(app, &items)?;

    // ── Build the tray ───────────────────────────────────────────────

    let tray = TrayIconBuilder::new()
        .tooltip(app.config().product_name.as_deref().unwrap_or("rcm-tauri"))
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            ids::QUIT => {
                let _ = actions::shutdown();
                app.exit(0);
            }
            ids::STYLE_TOGGLE => handle_style_toggle(&style_clone),
            ids::REGISTER_TOGGLE => handle_register_toggle(&register_clone),
            ids::BLOCKING_TOGGLE => handle_blocking_toggle(&blocking_clone),
            ids::ICONS => handle_icons_toggle(app, &icons_clone),
            ids::DEV => handle_dev_toggle(app, &dev_clone),
            ids::AUTOSTART => handle_autostart_toggle(&autostart_clone),
            ids::THEME_SYSTEM => handle_theme(
                config::Theme::System,
                app,
                &theme_sys_clone,
                &theme_light_clone,
                &theme_dark_clone,
            ),
            ids::THEME_LIGHT => handle_theme(
                config::Theme::Light,
                app,
                &theme_sys_clone,
                &theme_light_clone,
                &theme_dark_clone,
            ),
            ids::THEME_DARK => handle_theme(
                config::Theme::Dark,
                app,
                &theme_sys_clone,
                &theme_light_clone,
                &theme_dark_clone,
            ),
            ids::APPLY => handle_apply(),
            ids::CONFIG => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::create_config_window(app_handle).await;
                });
            }
            ids::RESET => actions::reset(),
            ids::ABOUT => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::create_about_window(app_handle).await {
                        log::error("About", &e);
                    }
                });
            }
            _ => {}
        })
        .build(app)?;

    // Hide the previous icon before dropping it, so the shell cannot briefly
    // show two.
    let previous = TRAY.lock().ok().and_then(|mut slot| slot.replace(tray));
    if let Some(previous) = previous {
        let _ = previous.set_visible(false);
    }

    Ok(())
}
