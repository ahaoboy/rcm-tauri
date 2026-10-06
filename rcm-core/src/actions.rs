//! Menu and tray actions, shared by every frontend.
//!
//! Everything here is *system* behaviour: reading/writing config, toggling the
//! shell extension, restarting Explorer. None of it touches a UI toolkit, so both
//! frontends call exactly the same code.
//!
//! The functions return what the caller needs to update its UI: `set_*` returns
//! the value that should now be displayed, and fallible actions return
//! `Result<_, String>` carrying a ready-to-show message, so neither frontend
//! re-invents the wording.

use rcm_reg::MenuStyle;

use crate::{config, log, registry};

/// Ids for the tray entries.
///
/// Shared so both frontends route clicks identically, and so the ids stay
/// stable if a frontend is swapped.
pub mod ids {
    pub const CONFIG: &str = "config";
    pub const QUIT: &str = "quit";
}

/// Display labels for the tray entries.
pub mod text {
    pub const CONFIG: &str = "Config";
    pub const QUIT: &str = "Quit";
}

/// Whether the compact Windows 11 context menu is active.
pub fn is_win11() -> bool {
    MenuStyle::current() == MenuStyle::Windows11
}

/// Whether the shell-extension DLL is registered and valid.
pub fn register_status() -> bool {
    rcm_com::cmd::status()
        .map(|s| s.is_valid())
        .unwrap_or(false)
}

/// Switch the context-menu style. Returns the style that is now active.
pub fn set_style(style: MenuStyle) -> Result<MenuStyle, String> {
    style.set().map_err(|e| {
        let msg = format!("set {style:?} style failed: {e}");
        log::error("Tray", &msg);
        msg
    })?;
    Ok(style)
}

/// Register or unregister the shell extension. Returns the resulting status.
///
/// The status is re-read rather than assumed, so a registration that silently
/// failed reports the state that is actually in effect.
pub fn set_registered(register: bool) -> bool {
    let outcome = if register {
        rcm_com::cmd::register()
    } else {
        rcm_com::cmd::unregister()
    };

    if let Err(e) = outcome {
        let action = if register { "register" } else { "unregister" };
        log::error("Tray", &format!("failed to {action} shell extension: {e}"));
    }

    register_status()
}

/// Enable or disable native context-menu blocking.
///
/// Returns a human-readable confirmation on success, or the error message.
/// The shared blocking cache is updated by
/// [`crate::ui::enable_blocking`]/[`crate::ui::disable_blocking`], so every
/// reader agrees without extra bookkeeping.
pub fn set_blocking(enable: bool) -> Result<String, String> {
    let result = if enable {
        crate::ui::enable_blocking()
            .map(|()| "Menu blocking ENABLED — native context menu will be hidden.".to_string())
    } else {
        crate::ui::disable_blocking()
            .map(|()| "Menu blocking DISABLED — native context menu will be shown.".to_string())
    };

    match result {
        Ok(msg) => {
            log::info("Tray", &msg);
            Ok(msg)
        }
        Err(e) => {
            let msg = format!("toggle menu blocking failed: {e}");
            log::error("Tray", &msg);
            Err(msg)
        }
    }
}

/// Enable or disable the "launch at startup" registration. Returns the new value.
pub fn set_autostart(enabled: bool) -> Result<bool, String> {
    let result = if enabled {
        registry::enable_autostart()
    } else {
        registry::disable_autostart()
    };

    if let Err(e) = result {
        let action = if enabled { "enable" } else { "disable" };
        let msg = format!("{action} autostart failed: {e}");
        log::error("Tray", &msg);
        return Err(msg);
    }

    log::info(
        "Tray",
        if enabled {
            "autostart enabled"
        } else {
            "autostart disabled"
        },
    );
    Ok(enabled)
}

/// Restart Explorer so registry changes take effect.
pub fn apply() -> Result<(), String> {
    rcm_reg::restart_explorer(std::time::Duration::from_secs(3)).map_err(|e| {
        let msg = format!("restart Explorer failed: {e}");
        log::error("Tray", &msg);
        msg
    })
}

/// Reset every config and menu file to the embedded defaults.
pub fn reset() {
    config::reset();
    crate::style::write_style_defaults();
}

/// Disable menu blocking ahead of process exit.
///
/// Returns the error message if the shell extension could not be reached, so
/// the caller can log it; quitting proceeds regardless.
pub fn shutdown() -> Result<(), String> {
    rcm_com::disable().map_err(|e| {
        let msg = format!("rcm_com::disable failed: {e}");
        log::error("Shutdown", &msg);
        msg
    })
}
