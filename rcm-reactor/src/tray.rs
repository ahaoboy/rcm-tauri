//! System-tray integration.
//!
//! A dedicated thread owns the tray icon and its native menu (via the
//! `tray-icon` crate, which wraps `muda`) and runs the Win32 message pump that
//! the tray requires.
//!
//! All *system* behaviour lives in [`rcm_core::actions`], shared with the Tauri
//! build. This module only builds the menu, keeps the checkmarks in sync, and
//! translates clicks into [`AppEvent`]s.

use tray_icon::menu::{
    CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use tray_icon::{Icon, TrayIconBuilder};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, TranslateMessage,
};

use rcm_core::actions::{self, ids, text};
use rcm_core::log;
use rcm_reg::MenuStyle;

use crate::events::AppEvent;
use crate::events::push as push_event;

/// Tray icon embedded from the Tauri project's `public/icon-tray.ico`.
const TRAY_ICON: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../public/icon-tray.ico"
));

/// File name the tray writes when extracting the embedded icon.
const TRAY_ICON_FILE: &str = "icon-tray.ico";

// ═══════════════════════════════════════════════════════════════════════════
// Tray state — lives on the tray thread only (muda items are not `Send`).
// ═══════════════════════════════════════════════════════════════════════════

struct TrayState {
    win11: CheckMenuItem,
    classic: CheckMenuItem,
    register: CheckMenuItem,
    enable: CheckMenuItem,
    disable: CheckMenuItem,
    /// Only present in debug builds.
    icons: Option<CheckMenuItem>,
    /// Only present in debug builds.
    dev: Option<CheckMenuItem>,
    autostart: CheckMenuItem,
    theme_system: CheckMenuItem,
    theme_light: CheckMenuItem,
    theme_dark: CheckMenuItem,
}

impl TrayState {
    /// Re-tick the style checkmarks from the current registry state.
    fn sync_style(&self) {
        let win11 = actions::is_win11();
        self.win11.set_checked(win11);
        self.classic.set_checked(!win11);
    }

    /// Re-tick the blocking checkmarks from the shared cache.
    fn sync_blocking(&self) {
        let enabled = rcm_core::ui::is_blocking_enabled();
        self.enable.set_checked(enabled);
        self.disable.set_checked(!enabled);
    }

    /// Re-tick the theme checkmarks for `theme`.
    fn sync_theme(&self, theme: rcm_core::config::Theme) {
        self.theme_system
            .set_checked(theme == rcm_core::config::Theme::System);
        self.theme_light
            .set_checked(theme == rcm_core::config::Theme::Light);
        self.theme_dark
            .set_checked(theme == rcm_core::config::Theme::Dark);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Menu construction
// ═══════════════════════════════════════════════════════════════════════════

fn build_menu() -> Result<(Menu, TrayState), Box<dyn std::error::Error>> {
    let win11 = CheckMenuItem::with_id(
        ids::WIN11_STYLE,
        text::WIN11,
        true,
        actions::is_win11(),
        None,
    );
    let classic = CheckMenuItem::with_id(
        ids::CLASSIC_STYLE,
        text::CLASSIC,
        true,
        !actions::is_win11(),
        None,
    );
    let register = CheckMenuItem::with_id(
        ids::REGISTER,
        text::REGISTER,
        true,
        actions::register_status(),
        None,
    );
    let unregister = MenuItem::with_id(ids::UNREGISTER, text::UNREGISTER, true, None);

    let blocking = rcm_core::ui::is_blocking_enabled();
    let enable = CheckMenuItem::with_id(ids::ENABLE, text::ENABLE, true, blocking, None);
    let disable = CheckMenuItem::with_id(ids::DISABLE, text::DISABLE, true, !blocking, None);

    // Debug-only toggles, matching the Tauri build's `cfg!(debug_assertions)`.
    let is_debug = cfg!(debug_assertions);
    let icons = is_debug.then(|| {
        CheckMenuItem::with_id(
            ids::ICONS,
            text::ICONS,
            true,
            rcm_core::config::is_icons(),
            None,
        )
    });
    let dev = is_debug.then(|| {
        CheckMenuItem::with_id(ids::DEV, text::DEV, true, rcm_core::config::is_dev(), None)
    });

    let autostart = CheckMenuItem::with_id(
        ids::AUTOSTART,
        text::AUTOSTART,
        true,
        rcm_core::registry::is_autostart_enabled(),
        None,
    );

    let theme = rcm_core::config::theme();
    let theme_system = CheckMenuItem::with_id(
        ids::THEME_SYSTEM,
        text::THEME_SYSTEM,
        true,
        theme == rcm_core::config::Theme::System,
        None,
    );
    let theme_light = CheckMenuItem::with_id(
        ids::THEME_LIGHT,
        text::THEME_LIGHT,
        true,
        theme == rcm_core::config::Theme::Light,
        None,
    );
    let theme_dark = CheckMenuItem::with_id(
        ids::THEME_DARK,
        text::THEME_DARK,
        true,
        theme == rcm_core::config::Theme::Dark,
        None,
    );

    let pull_js = MenuItem::with_id(ids::PULL_JS, text::PULL_JS, true, None);
    let pull_css = MenuItem::with_id(ids::PULL_CSS, text::PULL_CSS, true, None);
    let pull_config = MenuItem::with_id(ids::PULL_CONFIG, text::PULL_CONFIG, true, None);
    let config_item = MenuItem::with_id(ids::CONFIG, text::CONFIG, true, None);
    let reset = MenuItem::with_id(ids::RESET, text::RESET, true, None);
    let apply = MenuItem::with_id(ids::APPLY, text::APPLY, true, None);
    let quit = MenuItem::with_id(ids::QUIT, text::QUIT, true, None);

    let separator_prefs = PredefinedMenuItem::separator();
    let separator_system = PredefinedMenuItem::separator();

    let theme_menu = Submenu::with_items(
        text::THEME,
        true,
        &[&theme_system, &theme_light, &theme_dark],
    )?;
    let pull_menu = Submenu::with_items(text::PULL, true, &[&pull_js, &pull_css, &pull_config])?;

    // Top-level items. References are collected first so the conditional
    // entries can be appended.
    let mut items: Vec<&dyn IsMenuItem> = vec![
        &win11,
        &classic,
        &separator_prefs,
        &register,
        &unregister,
        &enable,
        &disable,
    ];
    if let Some(item) = &icons {
        items.push(item);
    }
    if let Some(item) = &dev {
        items.push(item);
    }
    items.push(&autostart);
    items.push(&theme_menu);
    items.push(&separator_system);
    if actions::has_remote() {
        items.push(&pull_menu);
    }
    items.push(&config_item);
    items.push(&reset);
    items.push(&apply);
    items.push(&quit);

    let menu = Menu::with_items(&items)?;

    let state = TrayState {
        win11,
        classic,
        register,
        enable,
        disable,
        icons,
        dev,
        autostart,
        theme_system,
        theme_light,
        theme_dark,
    };
    Ok((menu, state))
}

// ═══════════════════════════════════════════════════════════════════════════
// Event handling
// ═══════════════════════════════════════════════════════════════════════════

/// Report a failed action by opening the shared error window.
fn report_failure(title: String, message: String) {
    push_event(AppEvent::ShowError { title, message });
}

fn handle_menu_event(state: &TrayState, id: &str) {
    match id {
        ids::QUIT => {
            let _ = actions::shutdown();
            std::process::exit(0);
        }

        ids::WIN11_STYLE => {
            if actions::set_style(MenuStyle::Windows11).is_ok() {
                state.sync_style();
            }
        }
        ids::CLASSIC_STYLE => {
            if actions::set_style(MenuStyle::Classic).is_ok() {
                state.sync_style();
            }
        }

        ids::REGISTER => state.register.set_checked(actions::set_registered(true)),
        ids::UNREGISTER => state.register.set_checked(actions::set_registered(false)),

        ids::ENABLE => {
            if actions::set_blocking(true).is_ok() {
                state.sync_blocking();
            }
        }
        ids::DISABLE => {
            if actions::set_blocking(false).is_ok() {
                state.sync_blocking();
            }
        }

        ids::ICONS => {
            if let Some(item) = &state.icons {
                let value = actions::toggle_icons();
                item.set_checked(value);
                push_event(AppEvent::IconsChanged(value));
            }
        }
        ids::DEV => {
            if let Some(item) = &state.dev {
                let value = actions::toggle_dev();
                item.set_checked(value);
                push_event(AppEvent::DevChanged(value));
            }
        }

        ids::AUTOSTART => match actions::toggle_autostart() {
            Ok(value) => state.autostart.set_checked(value),
            Err(e) => report_failure("Startup Toggle Failed".into(), e),
        },

        ids::THEME_SYSTEM => {
            let theme = actions::set_theme(rcm_core::config::Theme::System);
            state.sync_theme(theme);
            push_event(AppEvent::ThemeChanged(theme.as_str().to_string()));
        }
        ids::THEME_LIGHT => {
            let theme = actions::set_theme(rcm_core::config::Theme::Light);
            state.sync_theme(theme);
            push_event(AppEvent::ThemeChanged(theme.as_str().to_string()));
        }
        ids::THEME_DARK => {
            let theme = actions::set_theme(rcm_core::config::Theme::Dark);
            state.sync_theme(theme);
            push_event(AppEvent::ThemeChanged(theme.as_str().to_string()));
        }

        ids::APPLY => {
            if let Err(e) = actions::apply() {
                report_failure("Apply Failed".into(), e);
            }
        }

        ids::PULL_JS | ids::PULL_CSS | ids::PULL_CONFIG => {
            let suffix = match id {
                ids::PULL_JS => "js",
                ids::PULL_CSS => "css",
                _ => "config",
            };
            handle_pull(suffix);
        }

        ids::CONFIG => push_event(AppEvent::OpenConfigEditor),

        ids::RESET => actions::reset(),

        _ => {}
    }
}

/// Download a remote file and, for CSS, tell the UI to restyle.
fn handle_pull(suffix: &str) {
    let Some(file) = actions::PullFile::parse(suffix) else {
        log::error("Tray", &format!("unknown pull target: {suffix}"));
        return;
    };

    match actions::pull(file) {
        Ok(outcome) => {
            // Only CSS affects anything currently on screen.
            if let Some(css) = outcome.style_css() {
                push_event(AppEvent::StyleChanged(css));
            }
        }
        Err(e) => report_failure(format!("Pull {} Failed", file.file_name()), e),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Icon + thread
// ═══════════════════════════════════════════════════════════════════════════

/// Load the tray icon: prefer the `.ico` next to the executable, writing the
/// embedded copy if it is missing.
fn load_icon() -> Option<Icon> {
    let path = rcm_core::exe_dir().join(TRAY_ICON_FILE);
    if !path.exists() {
        let _ = std::fs::write(&path, TRAY_ICON);
    }
    match Icon::from_path(&path, None) {
        Ok(icon) => Some(icon),
        Err(e) => {
            log::error("Tray", &format!("load icon '{}': {e}", path.display()));
            None
        }
    }
}

/// Spawn the tray thread. The thread owns the tray icon and runs the Win32
/// message pump required by `tray-icon`/`muda`.
pub fn start() {
    let spawned = std::thread::Builder::new()
        .name("rcm-tray".to_string())
        .spawn(|| {
            let (menu, state) = match build_menu() {
                Ok(value) => value,
                Err(e) => {
                    log::error("Tray", &format!("build menu failed: {e}"));
                    return;
                }
            };

            let mut builder = TrayIconBuilder::new()
                .with_tooltip(crate::events::ROOT_TITLE)
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(true);
            if let Some(icon) = load_icon() {
                builder = builder.with_icon(icon);
            }

            let _tray = match builder.build() {
                Ok(tray) => tray,
                Err(e) => {
                    log::error("Tray", &format!("create tray icon failed: {e}"));
                    return;
                }
            };
            log::info("Tray", "tray icon ready");

            // Win32 message pump. Menu clicks arrive through muda's global
            // receiver after the message is dispatched.
            let mut msg = MSG::default();
            unsafe {
                loop {
                    // `GetMessageW` returns -1 on error and 0 on `WM_QUIT`, so
                    // only a positive value means "keep pumping". Treating an
                    // error as a message would spin this loop forever.
                    if GetMessageW(&mut msg, None, 0, 0).0 <= 0 {
                        break;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    while let Ok(event) = MenuEvent::receiver().try_recv() {
                        handle_menu_event(&state, event.id().as_ref());
                    }
                }
            }
        });

    if let Err(e) = spawned {
        log::error("Tray", &format!("failed to spawn tray thread: {e}"));
    }
}
