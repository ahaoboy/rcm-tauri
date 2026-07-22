//! Persistent runtime configuration stored as `rcm.config.json`
//! next to the executable.
//!
//! On startup the file is read; if it doesn't exist it is created
//! with defaults.  Tray toggles persist immediately.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(&self) -> &str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Data
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    /// Dev mode flag
    #[serde(default)]
    dev: bool,
    /// Show icon ribbon at top of menu
    #[serde(default)]
    icons: bool,
    /// Menu theme: system (follow OS), light, or dark
    #[serde(default)]
    theme: Theme,
    /// Event filter rules.
    /// - Missing field → use built-in defaults
    /// - Empty array `[]` → no filtering (allow all events)
    /// - Non-empty → use specified rules
    #[serde(default, skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<FilterRule>>,
    /// Remote URL for menu JS sync (empty = disabled)
    #[serde(default = "default_js_url")]
    js_url: String,
    /// Remote URL for style CSS sync (empty = disabled)
    #[serde(default = "default_css_url")]
    css_url: String,
    /// Remote URL for config JSON sync (empty = disabled)
    #[serde(default = "default_config_url")]
    config_url: String,
}

fn default_js_url() -> String {
    "https://github.com/ahaoboy/rcm-tauri/releases/latest/download/rcm.js".into()
}
fn default_css_url() -> String {
    "https://github.com/ahaoboy/rcm-tauri/releases/latest/download/style.css".into()
}
fn default_config_url() -> String {
    "".into()
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            dev: false,
            icons: false,
            theme: Theme::default(),
            filters: None,
            js_url: default_js_url(),
            css_url: default_css_url(),
            config_url: default_config_url(),
        }
    }
}

/// A single filter rule for ignoring context-menu events.
///
/// A rule matches when **all** of its non-empty / `Some` fields match.
/// - `class_re` — regex against `event.class` (empty → match all)
/// - `file_eq`  — exact match against any entry in `event.files` (empty → match all)
/// - `flags_eq` — exact match against `event.event.flags()` (`None` → match all)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// Regex pattern for window class.
    #[serde(default)]
    pub class: String,
    /// Exact file path to match (compared with `==` against each file).
    #[serde(default)]
    pub file: String,
    /// Exact `flags()` value to match.
    #[serde(default)]
    pub flags: Option<u32>,
    /// Human-readable reason logged when the filter triggers.
    #[serde(default)]
    pub reason: String,
}

fn default_filters() -> Vec<FilterRule> {
    vec![
        // RCM's own transparent overlay windows — prevent self-trigger loops
        FilterRule {
            class: r"^Tauri Window$".into(),
            file: String::new(),
            flags: None,
            reason: "RCM self-window (Tauri Window)".into(),
        },
        // OpenWith dialog
        FilterRule {
            class: r"^Chrome_WidgetWin_".into(),
            file: String::new(),
            flags: Some(16),
            reason: "OpenWith dialog (Chrome_WidgetWin_, flags=16)".into(),
        },
        // Generic Windows dialogs (#32770) — installer completion,
        // message boxes, file-open dialogs, etc.
        // https://learn.microsoft.com/en-us/windows/win32/winauto/dialog-box
        FilterRule {
            class: r"^#32770$".into(),
            file: String::new(),
// ComfyUI installer completion dialog uses flags=2050 (silent/static), whereas OBS file picker uses 132128 (visible/refreshing).
            flags: Some(2050),
            reason: "Windows dialog (#32770)".into(),
        },
        // Explorer programmatic context menu (flags=16 = CMF_CANRENAME).
        // Fired as a side effect when rcm executes shell verbs (e.g. pintohome,
        // pin-to-start, etc.) — not a real user right-click.
        FilterRule {
            class: r"^CabinetWClass$".into(),
            file: String::new(),
            flags: Some(16),
            reason: "Explorer programmatic menu (CabinetWClass, flags=16)".into(),
        },
        // UWP system UI (taskbar jump lists, etc.)
        FilterRule {
            class: r"^Windows\.UI\.Core\.CoreWindow$".into(),
            file: String::new(),
            flags: None,
            reason: "CoreWindow (taskbar jump list / UWP system UI)".into(),
        },
    ]
}

impl FilterRule {
    /// Check whether this rule matches a context-menu event.
    /// Returns `true` when **all** non-empty / `Some` fields match.
    pub fn matches(&self, event: &rcm_com::ContextMenuInfo) -> bool {
        // class_re — regex match (skip if empty)
        if !self.class.is_empty() {
            match regex::Regex::new(&self.class) {
                Ok(re) => {
                    if !re.is_match(&event.class) {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }

        // file_eq — exact match against any file in the list (skip if empty)
        if !self.file.is_empty() && !event.files.iter().any(|f| f == &self.file) {
            return false;
        }

        // flags_eq — exact match against event flags (skip if None)
        if let Some(f) = self.flags
            && event.event.flags() != f
        {
            return false;
        }

        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Init — called once at startup
// ═══════════════════════════════════════════════════════════════════════════

/// Ensure the config file exists (creating defaults if missing),
/// then log the current configuration.
pub fn init() {
    let path = config_path();

    if std::fs::read_to_string(&path).is_err() {
        save_inner(&path, &ConfigFile::default());
    }

    println!(
        "config: dev={} icons={} theme={:?} filters={} js_url={} css_url={} cfg_url={} ({})",
        is_dev(),
        is_icons(),
        theme(),
        filters().len(),
        remote_js_url().as_deref().unwrap_or("(none)"),
        remote_css_url().as_deref().unwrap_or("(none)"),
        remote_config_url().as_deref().unwrap_or("(none)"),
        path.display(),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Getters — always read from file so manual edits take effect immediately
// ═══════════════════════════════════════════════════════════════════════════

pub fn is_dev() -> bool {
    read_config().dev
}

pub fn is_icons() -> bool {
    read_config().icons
}

/// Return the current list of event filter rules.
/// - `None` in config → built-in defaults.
/// - `[]` in config → empty (allow all events).
pub fn filters() -> Vec<FilterRule> {
    read_config().filters.unwrap_or_else(default_filters)
}

pub fn theme() -> Theme {
    read_config().theme
}

/// Return the remote menu sync URL, or `None` if not configured.
pub fn remote_js_url() -> Option<String> {
    url_or_none(&read_config().js_url)
}
pub fn remote_css_url() -> Option<String> {
    url_or_none(&read_config().css_url)
}
pub fn remote_config_url() -> Option<String> {
    url_or_none(&read_config().config_url)
}

fn url_or_none(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Setters — read-modify-write the config file
// ═══════════════════════════════════════════════════════════════════════════

pub fn set_dev(dev: bool) {
    update_config(|cfg| cfg.dev = dev);
}

pub fn set_icons(icons: bool) {
    update_config(|cfg| cfg.icons = icons);
}

pub fn set_remote_js_url(url: String) {
    update_config(|cfg| cfg.js_url = url);
}
pub fn set_remote_css_url(url: String) {
    update_config(|cfg| cfg.css_url = url);
}
pub fn set_remote_config_url(url: String) {
    update_config(|cfg| cfg.config_url = url);
}

pub fn set_theme(theme: Theme) {
    update_config(|cfg| cfg.theme = theme);
}

/// Reset all config and menu files to embedded defaults.
pub fn reset() {
    save_inner(&config_path(), &ConfigFile::default());
    crate::menu::write_menu_defaults();
    println!("config: reset to defaults");
}

// ═══════════════════════════════════════════════════════════════════════════
// Internal
// ═══════════════════════════════════════════════════════════════════════════

fn config_path() -> PathBuf {
    crate::exe_dir().join("rcm.config.json")
}

/// Read and parse the config file, falling back to defaults on any error.
fn read_config() -> ConfigFile {
    let path = config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// Read the config file, apply `f`, and write it back.
fn update_config(f: impl FnOnce(&mut ConfigFile)) {
    let path = config_path();
    let mut cfg = read_config();
    f(&mut cfg);
    save_inner(&path, &cfg);
}

fn save_inner(path: &std::path::Path, cfg: &ConfigFile) {
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}
