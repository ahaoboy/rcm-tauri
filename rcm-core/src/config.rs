//! Persistent runtime configuration stored as `rcm.config.json`
//! next to the executable.
//!
//! On startup the file is read; if it doesn't exist it is created
//! with defaults.  Tray toggles persist immediately.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════
// Data
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConfigFile {
    /// Dev mode flag
    #[serde(default)]
    dev: bool,
    /// Show icon ribbon at top of menu
    #[serde(default)]
    icons: bool,
    /// Event filter rules.
    /// - Missing field → use built-in defaults
    /// - Empty array `[]` → no filtering (allow all events)
    /// - Non-empty → use specified rules
    #[serde(default, skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<FilterRule>>,
    /// Remote URL for menu JS sync (empty = disabled)
    #[serde(default)]
    url: String,
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
        // OpenWith dialog
        FilterRule {
            class: r"^Chrome_WidgetWin_".into(),
            file: String::new(),
            flags: Some(16),
            reason: "OpenWith dialog (Chrome_WidgetWin_, flags=16)".into(),
        },
        // UWP system UI (taskbar jump lists, etc.) — all events from
        // Windows.UI.Core.CoreWindow are spurious; flags vary
        // (observed: 2048=CMF_OPTIMIZEFORINVOKE, 32770=CMF_VERBSONLY|0x8000)
        // so we match any flags by leaving it None.
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
        "config: dev={} icons={} filters={} remote={} ({})",
        is_dev(),
        is_icons(),
        filters().len(),
        remote_url().as_deref().unwrap_or("(none)"),
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

/// Return the remote menu sync URL, or `None` if not configured.
pub fn remote_url() -> Option<String> {
    let url = &read_config().url;
    if url.is_empty() {
        None
    } else {
        Some(url.clone())
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

pub fn set_remote_url(url: String) {
    update_config(|cfg| cfg.url = url);
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
