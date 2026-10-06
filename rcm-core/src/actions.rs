//! Menu and tray actions, shared by every frontend.
//!
//! Everything here is *system* behaviour: reading/writing config, toggling the
//! shell extension, or downloading a remote file. None of it touches a UI
//! toolkit, so the Tauri and Reactor trays call exactly the same code and only
//! differ in how they present the result (emit a Tauri event vs. push an
//! in-process event, and which checkmark to tick).
//!
//! The functions therefore return what the caller needs to update its UI:
//!
//! - `toggle_*` returns the new value;
//! - `set_*` returns the value that should now be displayed;
//! - fallible actions return `Result<_, String>` carrying a ready-to-show
//!   message, so neither frontend re-invents the wording.

use rcm_reg::MenuStyle;

use crate::{config, log, menu, registry};

/// Ids for every tray entry.
///
/// Shared so both frontends route clicks identically, and so the ids stay
/// stable if a frontend is swapped.
pub mod ids {
    /// Merged entry for the Win11 / Classic pair — click switches style.
    pub const STYLE_TOGGLE: &str = "style_toggle";
    /// Merged entry for the enable / disable pair — click switches blocking.
    pub const BLOCKING_TOGGLE: &str = "blocking_toggle";
    /// Merged entry for the register / unregister pair — click switches it.
    pub const REGISTER_TOGGLE: &str = "register_toggle";
    pub const APPLY: &str = "apply";
    pub const DEV: &str = "dev";
    pub const ICONS: &str = "icons";
    pub const THEME_SYSTEM: &str = "theme_system";
    pub const THEME_LIGHT: &str = "theme_light";
    pub const THEME_DARK: &str = "theme_dark";
    pub const AUTOSTART: &str = "autostart";
    pub const RESET: &str = "reset";
    pub const QUIT: &str = "quit";
    pub const CONFIG: &str = "config";
    pub const PULL_JS: &str = "pull_js";
    pub const PULL_CSS: &str = "pull_css";
    pub const PULL_CONFIG: &str = "pull_config";
    pub const ABOUT: &str = "about";
}

/// Display labels for the tray entries.
pub mod text {
    pub const QUIT: &str = "Quit";
    pub const WIN11: &str = "Win11";
    pub const CLASSIC: &str = "Classic";
    pub const REGISTER: &str = "Register";
    pub const UNREGISTER: &str = "Unregister";
    pub const ENABLE: &str = "Enable";
    pub const DISABLE: &str = "Disable";
    pub const DEV: &str = "Dev";
    pub const ICONS: &str = "Icons";
    pub const AUTOSTART: &str = "Startup";
    pub const RESET: &str = "Reset";
    pub const APPLY: &str = "Apply";
    pub const PULL: &str = "Pull";
    pub const PULL_JS: &str = "JS";
    pub const PULL_CSS: &str = "CSS";
    pub const PULL_CONFIG: &str = "Config";
    pub const CONFIG: &str = "Config";
    pub const THEME: &str = "Theme";
    pub const THEME_SYSTEM: &str = "System";
    pub const THEME_LIGHT: &str = "Light";
    pub const THEME_DARK: &str = "Dark";
    pub const ABOUT: &str = "About";
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

/// Whether any remote sync URL is configured (i.e. the Pull submenu applies).
pub fn has_remote() -> bool {
    config::remote_js_url().is_some()
        || config::remote_css_url().is_some()
        || config::remote_config_url().is_some()
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

/// The style [`toggle_style`] would switch to.
pub fn next_style() -> MenuStyle {
    if is_win11() {
        MenuStyle::Classic
    } else {
        MenuStyle::Windows11
    }
}

/// Flip the context-menu style. Returns the style that is now active.
///
/// The tray shows a single entry for the Win11 / Classic pair, so it needs one
/// call that both decides and applies the switch. Mirrors the other `toggle_*`
/// helpers.
pub fn toggle_style() -> Result<MenuStyle, String> {
    set_style(next_style())
}

/// Label for the merged style entry.
///
/// Names the style clicking *selects*, not the active one: a menu entry reads as
/// the action it performs, so an entry labelled "Classic" means "switch to
/// Classic". Which style is currently active is therefore readable from the
/// entry without a checkmark.
pub fn style_entry_label() -> &'static str {
    match next_style() {
        MenuStyle::Windows11 => text::WIN11,
        MenuStyle::Classic => text::CLASSIC,
    }
}

/// The blocking state [`toggle_blocking`] would switch to.
pub fn next_blocking() -> bool {
    !crate::ui::is_blocking_enabled()
}

/// Flip native context-menu blocking. Returns the new state.
pub fn toggle_blocking() -> Result<bool, String> {
    let next = next_blocking();
    set_blocking(next).map(|_| next)
}

/// Label for the merged blocking entry. Names the action, like
/// [`style_entry_label`].
pub fn blocking_entry_label() -> &'static str {
    if next_blocking() {
        text::ENABLE
    } else {
        text::DISABLE
    }
}

/// Register or unregister the shell extension. Returns the resulting status.
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

/// The registration state [`toggle_registered`] would switch to.
pub fn next_registered() -> bool {
    !register_status()
}

/// Register or unregister the shell extension. Returns the new state.
pub fn toggle_registered() -> bool {
    set_registered(next_registered())
}

/// Label for the merged registration entry. Names the action, like
/// [`style_entry_label`].
pub fn register_entry_label() -> &'static str {
    if next_registered() {
        text::REGISTER
    } else {
        text::UNREGISTER
    }
}

/// Enable or disable native context-menu blocking.
///
/// Returns a human-readable confirmation on success, or the error message.
/// The shared blocking cache is updated by
/// [`crate::ui::enable_blocking`]/[`crate::ui::disable_blocking`], so the tray
/// checkmarks and the event monitor agree without extra bookkeeping.
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

/// Flip the icon-ribbon preference. Returns the new value.
pub fn toggle_icons() -> bool {
    let value = !config::is_icons();
    config::set_icons(value);
    value
}

/// Flip dev mode. Returns the new value.
pub fn toggle_dev() -> bool {
    let value = !config::is_dev();
    config::set_dev(value);
    value
}

/// Set the theme. Returns the theme that is now active.
pub fn set_theme(theme: config::Theme) -> config::Theme {
    config::set_theme(theme);
    theme
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

/// Flip the "launch at startup" registration. Returns the new value.
pub fn toggle_autostart() -> Result<bool, String> {
    set_autostart(!registry::is_autostart_enabled())
}

/// Which remote file to download.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullFile {
    Js,
    Css,
    Config,
}

impl PullFile {
    /// Parse the suffix used by the tray ids / frontend commands.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "js" => Some(Self::Js),
            "css" => Some(Self::Css),
            "config" => Some(Self::Config),
            _ => None,
        }
    }

    /// File name this pull writes next to the executable.
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Js => "rcm.js",
            Self::Css => "style.css",
            Self::Config => "rcm.config.json",
        }
    }

    /// Configured remote URL, or an error if none is set.
    fn remote_url(self) -> Result<String, String> {
        let (url, what) = match self {
            Self::Js => (config::remote_js_url(), "rcm.js"),
            Self::Css => (config::remote_css_url(), "style.css"),
            Self::Config => (config::remote_config_url(), "rcm.config.json"),
        };
        url.ok_or_else(|| format!("No remote URL configured for {what}"))
    }

    fn download(self, url: &str) -> Result<String, String> {
        match self {
            Self::Js => menu::download_menu(url),
            Self::Css => menu::download_style(url),
            Self::Config => menu::download_config(url),
        }
    }
}

/// Result of a successful [`pull`].
#[derive(Debug, Clone)]
pub struct PullOutcome {
    /// Which file was pulled.
    pub file: PullFile,
    /// Absolute path it was written to.
    pub path: String,
}

impl PullOutcome {
    /// Contents of `style.css`, for frontends that push CSS to open windows.
    /// `None` for every other file.
    pub fn style_css(&self) -> Option<String> {
        (self.file == PullFile::Css)
            .then(|| std::fs::read_to_string(&self.path).ok())
            .flatten()
    }
}

/// Download one of the remote files. Logs progress and failure.
pub fn pull(file: PullFile) -> Result<PullOutcome, String> {
    let url = file.remote_url()?;
    log::info("Pull", &format!("pulling {} from {url}", file.file_name()));

    let path = file.download(&url).map_err(|e| {
        log::error("Pull", &format!("{} failed: {e}", file.file_name()));
        e
    })?;

    log::info("Pull", &format!("{} saved to {path}", file.file_name()));
    Ok(PullOutcome { file, path })
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
