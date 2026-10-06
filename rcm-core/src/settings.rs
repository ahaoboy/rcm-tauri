//! The user-facing settings, gathered from wherever each one is stored.
//!
//! There is no single settings file: the menu style and the startup entry live in
//! the registry, the shell-extension registration in the extension's own state,
//! the blocking flag in the shell extension, and the rest in `rcm.config.json`.
//! This module is the one place that knows the full set, so the tray and the
//! settings tab read and write exactly the same things.
//!
//! The remote-sync URLs are deliberately *not* here: they are text the config
//! editor already edits as JSON, and duplicating them as a form would give two
//! places to change the same value.

use serde::{Deserialize, Serialize};

use crate::{actions, config, registry, ui};

/// Context-menu style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    /// The compact Windows 11 menu.
    Win11,
    /// The classic (Windows 10) menu.
    Classic,
}

impl Style {
    /// The style currently in effect.
    pub fn current() -> Self {
        if actions::is_win11() {
            Self::Win11
        } else {
            Self::Classic
        }
    }
}

/// Every user-facing setting.
///
/// A snapshot, not a live view: [`Settings::current`] reads each source, so the
/// values are consistent with each other even though they live in several places.
#[derive(Debug, Clone, Serialize)]
pub struct Settings {
    /// Which context-menu style is active.
    pub style: Style,
    /// Menu theme preference.
    pub theme: config::Theme,
    /// Show the icon ribbon.
    pub icons: bool,
    /// Dev mode (keeps the menu open after a command).
    pub dev: bool,
    /// Hide the native context menu.
    pub blocking: bool,
    /// Shell extension registered.
    pub registered: bool,
    /// Path the registration points at.
    ///
    /// The registered path when the extension is registered, otherwise the one
    /// RCM expects next to the executable — so the value is meaningful before
    /// registering too.
    pub extension_dll: String,
    /// Launch at startup.
    pub autostart: bool,
}

impl Settings {
    /// Read every setting from its source.
    pub fn current() -> Self {
        // The registered path is what matters when it exists: it says which DLL
        // Explorer will actually load, which need not be the one we shipped.
        let registered_dll = rcm_com::cmd::status()
            .ok()
            .and_then(|status| status.inproc_path);

        Self {
            style: Style::current(),
            theme: config::theme(),
            icons: config::is_icons(),
            dev: config::is_dev(),
            blocking: ui::is_blocking_enabled(),
            registered: actions::register_status(),
            extension_dll: registered_dll.unwrap_or_else(|| {
                crate::exe_dir()
                    .join(EXTENSION_DLL_FILE)
                    .to_string_lossy()
                    .into_owned()
            }),
            autostart: registry::is_autostart_enabled(),
        }
    }
}

/// File name of the shell-extension DLL, next to the executable.
pub const EXTENSION_DLL_FILE: &str = "rcm_com.dll";

/// Settings to change. `None` leaves a field alone.
///
/// A patch rather than a whole [`Settings`] so a caller can change one setting
/// without having to send back values it never touched — and without the risk of
/// writing a stale value over a change made elsewhere in the meantime.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SettingsPatch {
    /// Switch the context-menu style.
    pub style: Option<Style>,
    /// Set the menu theme.
    pub theme: Option<config::Theme>,
    /// Show or hide the icon ribbon.
    pub icons: Option<bool>,
    /// Enable or disable dev mode.
    pub dev: Option<bool>,
    /// Hide or restore the native context menu.
    pub blocking: Option<bool>,
    /// Register or unregister the shell extension.
    pub registered: Option<bool>,
    /// Enable or disable launching at startup.
    pub autostart: Option<bool>,
}

/// Apply `patch` and return the resulting state.
///
/// Returns the **full** snapshot rather than the requested values: several of
/// these settings can be normalised or refused by the OS (registration, blocking,
/// the startup entry), and returning what is actually in effect stops the UI from
/// showing a state that was never applied.
///
/// An `Err` means the change did not take: the caller should re-read rather than
/// assume, since an earlier field in the patch may already have been applied.
pub fn update(patch: SettingsPatch) -> Result<Settings, String> {
    if let Some(style) = patch.style {
        let style = match style {
            Style::Win11 => rcm_reg::MenuStyle::Windows11,
            Style::Classic => rcm_reg::MenuStyle::Classic,
        };
        actions::set_style(style)?;
    }

    if let Some(registered) = patch.registered {
        actions::set_registered(registered);
    }

    if let Some(blocking) = patch.blocking {
        actions::set_blocking(blocking)?;
    }

    if let Some(autostart) = patch.autostart {
        actions::set_autostart(autostart)?;
    }

    if let Some(theme) = patch.theme {
        config::set_theme(theme);
    }

    if let Some(icons) = patch.icons {
        config::set_icons(icons);
    }

    if let Some(dev) = patch.dev {
        config::set_dev(dev);
    }

    Ok(Settings::current())
}
