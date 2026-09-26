//! System-tray integration.
//!
//! Builds the native tray menu and keeps its checkmarks in sync. All *system*
//! behaviour lives in [`rcm_core::actions`], shared with the Reactor build, so
//! this module is only the Tauri presentation: which item to tick, and which
//! event to emit to the frontend.

use rcm_core::actions::{self, ids, text};
use rcm_core::config;
use rcm_reg::MenuStyle;
use tauri::{
    App, Emitter,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
};

// ═══════════════════════════════════════════════════════════════════════════
// Checkmark helpers
// ═══════════════════════════════════════════════════════════════════════════

fn sync_style_checks<R: tauri::Runtime>(win11: &CheckMenuItem<R>, classic: &CheckMenuItem<R>) {
    let win11_active = actions::is_win11();
    let _ = win11.set_checked(win11_active);
    let _ = classic.set_checked(!win11_active);
}

fn sync_blocking_checks<R: tauri::Runtime>(enable: &CheckMenuItem<R>, disable: &CheckMenuItem<R>) {
    let enabled = rcm_core::ui::is_blocking_enabled();
    let _ = enable.set_checked(enabled);
    let _ = disable.set_checked(!enabled);
}

// ═══════════════════════════════════════════════════════════════════════════
// Event handlers
// ═══════════════════════════════════════════════════════════════════════════

fn handle_style_switch<R: tauri::Runtime>(
    style: MenuStyle,
    win11: &CheckMenuItem<R>,
    classic: &CheckMenuItem<R>,
) {
    if actions::set_style(style).is_ok() {
        sync_style_checks(win11, classic);
    }
}

fn handle_register_toggle<R: tauri::Runtime>(register: bool, item: &CheckMenuItem<R>) {
    let _ = item.set_checked(actions::set_registered(register));
}

fn handle_blocking_toggle<R: tauri::Runtime>(
    enable: bool,
    enable_i: &CheckMenuItem<R>,
    disable_i: &CheckMenuItem<R>,
) {
    if actions::set_blocking(enable).is_ok() {
        sync_blocking_checks(enable_i, disable_i);
    }
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

pub fn setup_tray(app: &mut App) -> Result<(), tauri::Error> {
    // ── Create menu items ────────────────────────────────────────────
    let win11_i = CheckMenuItem::with_id(
        app,
        ids::WIN11_STYLE,
        text::WIN11,
        true,
        actions::is_win11(),
        None::<&str>,
    )?;
    let classic_i = CheckMenuItem::with_id(
        app,
        ids::CLASSIC_STYLE,
        text::CLASSIC,
        true,
        !actions::is_win11(),
        None::<&str>,
    )?;
    let register_i = CheckMenuItem::with_id(
        app,
        ids::REGISTER,
        text::REGISTER,
        true,
        actions::register_status(),
        None::<&str>,
    )?;
    let unregister_i =
        MenuItem::with_id(app, ids::UNREGISTER, text::UNREGISTER, true, None::<&str>)?;

    let blocking = rcm_core::ui::is_blocking_enabled();
    let enable_i =
        CheckMenuItem::with_id(app, ids::ENABLE, text::ENABLE, true, blocking, None::<&str>)?;
    let disable_i = CheckMenuItem::with_id(
        app,
        ids::DISABLE,
        text::DISABLE,
        true,
        !blocking,
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
    let quit_i = MenuItem::with_id(app, ids::QUIT, text::QUIT, true, None::<&str>)?;

    // ── Clones for the event handler ─────────────────────────────────
    let win11_clone = win11_i.clone();
    let classic_clone = classic_i.clone();
    let register_clone = register_i.clone();
    let enable_clone = enable_i.clone();
    let disable_clone = disable_i.clone();
    let dev_clone = dev_i.clone();
    let icons_clone = icons_i.clone();
    let autostart_clone = autostart_i.clone();
    let theme_sys_clone = theme_sys_i.clone();
    let theme_light_clone = theme_light_i.clone();
    let theme_dark_clone = theme_dark_i.clone();

    // ── Build the menu (3 groups) ────────────────────────────────────
    //
    //   ✓ Win11 / Classic          ← Style
    //   ─────────
    //     Register / Unregister    ← Preferences
    //   ✓ Icons  (debug)
    //   ✓ Dev    (debug)
    //   ✓ Auto Start
    //   Theme ▸
    //   ─────────
    //   Config / Reset / Apply / Quit

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
        &win11_i,
        &classic_i,
        &separator_prefs,
        &register_i,
        &unregister_i,
        &enable_i,
        &disable_i,
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
    items.push(&quit_i);

    let menu = Menu::with_items(app, &items)?;

    // ── Build the tray ───────────────────────────────────────────────

    let _tray = TrayIconBuilder::new()
        .tooltip(app.config().product_name.as_deref().unwrap_or("rcm-tauri"))
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            ids::QUIT => {
                let _ = actions::shutdown();
                app.exit(0);
            }
            ids::WIN11_STYLE => {
                handle_style_switch(MenuStyle::Windows11, &win11_clone, &classic_clone)
            }
            ids::CLASSIC_STYLE => {
                handle_style_switch(MenuStyle::Classic, &win11_clone, &classic_clone)
            }
            ids::REGISTER => handle_register_toggle(true, &register_clone),
            ids::UNREGISTER => handle_register_toggle(false, &register_clone),
            ids::ENABLE => handle_blocking_toggle(true, &enable_clone, &disable_clone),
            ids::DISABLE => handle_blocking_toggle(false, &enable_clone, &disable_clone),
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
            _ => {}
        })
        .build(app)?;

    Ok(())
}
