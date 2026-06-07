//! Persistent runtime configuration stored as `rmc.config.json`
//! next to the executable.
//!
//! On startup the file is read; if it doesn't exist it is created
//! with defaults.  Tray toggles update the in-memory state and
//! persist immediately.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

// ═══════════════════════════════════════════════════════════════════════════
// Data
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    /// `"lite"` or `"full"`
    #[serde(default = "default_menu")]
    menu: String,
    /// Dev mode flag
    #[serde(default)]
    dev: bool,
    /// Show icon ribbon at top of menu
    #[serde(default)]
    icons: bool,
    /// Event filter rules
    #[serde(default = "default_filters")]
    filters: Vec<FilterRule>,
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
        // Windows Terminal / VS Code CoreWindow spurious events
        FilterRule {
            class: r"^Windows\.UI\.Core\.CoreWindow$".into(),
            file: String::new(),
            flags: Some(2048),
            reason: "CoreWindow spurious Menu (flags=2048)".into(),
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
        if !self.file.is_empty() {
            if !event.files.iter().any(|f| f == &self.file) {
                return false;
            }
        }

        // flags_eq — exact match against event flags (skip if None)
        if let Some(f) = self.flags {
            if event.event.flags() != f {
                return false;
            }
        }

        true
    }
}

fn default_menu() -> String {
    "lite".into()
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            menu: default_menu(),
            dev: false,
            icons: false,
            filters: default_filters(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// In-memory state (fast atomic reads)
// ═══════════════════════════════════════════════════════════════════════════

static IS_LITE: AtomicBool = AtomicBool::new(true);
static DEV_MODE: AtomicBool = AtomicBool::new(false);
static SHOW_ICONS: AtomicBool = AtomicBool::new(false);
static FILTERS: OnceLock<Vec<FilterRule>> = OnceLock::new();

// ═══════════════════════════════════════════════════════════════════════════
// Init — called once at startup
// ═══════════════════════════════════════════════════════════════════════════

/// Load config from `<exe_dir>/rmc.config.json`, or create it with defaults.
pub fn init() {
    let path = config_path();

    let cfg: ConfigFile = match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => {
            let default = ConfigFile::default();
            save_inner(&path, &default);
            default
        }
    };

    IS_LITE.store(cfg.menu == "lite", Ordering::Relaxed);
    DEV_MODE.store(cfg.dev, Ordering::Relaxed);
    SHOW_ICONS.store(cfg.icons, Ordering::Relaxed);
    let _ = FILTERS.set(cfg.filters);

    println!(
        "config: menu={} dev={} icons={} filters={} ({})",
        if is_lite() { "lite" } else { "full" },
        is_dev(),
        is_icons(),
        filters().len(),
        path.display(),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Getters
// ═══════════════════════════════════════════════════════════════════════════

pub fn is_lite() -> bool {
    IS_LITE.load(Ordering::Relaxed)
}

pub fn is_dev() -> bool {
    DEV_MODE.load(Ordering::Relaxed)
}

pub fn is_icons() -> bool {
    SHOW_ICONS.load(Ordering::Relaxed)
}

/// Return the current list of event filter rules.
pub fn filters() -> &'static [FilterRule] {
    FILTERS.get().map(|v| v.as_slice()).unwrap_or(&[])
}

// ═══════════════════════════════════════════════════════════════════════════
// Setters — update memory + persist to disk
// ═══════════════════════════════════════════════════════════════════════════

pub fn set_lite(lite: bool) {
    IS_LITE.store(lite, Ordering::Relaxed);
    save();
}

pub fn set_dev(dev: bool) {
    DEV_MODE.store(dev, Ordering::Relaxed);
    save();
}

pub fn set_icons(icons: bool) {
    SHOW_ICONS.store(icons, Ordering::Relaxed);
    save();
}

/// Reset all config and menu files to embedded defaults.
pub fn reset() {
    IS_LITE.store(true, Ordering::Relaxed);
    DEV_MODE.store(false, Ordering::Relaxed);
    SHOW_ICONS.store(false, Ordering::Relaxed);
    save();
    crate::menu_defaults::write_menu_defaults();
    println!("config: reset to defaults");
}

// ═══════════════════════════════════════════════════════════════════════════
// Internal
// ═══════════════════════════════════════════════════════════════════════════

fn config_path() -> PathBuf {
    let mut p = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    p.set_file_name("rmc.config.json");
    p
}

fn save() {
    let cfg = ConfigFile {
        menu: if IS_LITE.load(Ordering::Relaxed) {
            "lite".into()
        } else {
            "full".into()
        },
        dev: DEV_MODE.load(Ordering::Relaxed),
        icons: SHOW_ICONS.load(Ordering::Relaxed),
        filters: filters().to_vec(),
    };
    save_inner(&config_path(), &cfg);
}

fn save_inner(path: &std::path::Path, cfg: &ConfigFile) {
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}
