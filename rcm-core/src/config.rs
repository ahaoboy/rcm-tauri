//! Persistent runtime configuration stored as `rcm.config.json`
//! next to the executable.
//!
//! On startup the file is read; if it doesn't exist it is created
//! with defaults.  Tray toggles update the in-memory state and
//! persist immediately.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

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
    /// Event filter rules
    #[serde(default = "default_filters")]
    filters: Vec<FilterRule>,
    /// Remote URL for menu JS updates (empty = disabled)
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
        if !self.file.is_empty()
            && !event.files.iter().any(|f| f == &self.file) {
                return false;
            }

        // flags_eq — exact match against event flags (skip if None)
        if let Some(f) = self.flags
            && event.event.flags() != f {
                return false;
            }

        true
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            dev: false,
            icons: false,
            filters: default_filters(),
            url: String::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// In-memory state (fast atomic reads)
// ═══════════════════════════════════════════════════════════════════════════

static DEV_MODE: AtomicBool = AtomicBool::new(false);
static SHOW_ICONS: AtomicBool = AtomicBool::new(false);
static FILTERS: OnceLock<Vec<FilterRule>> = OnceLock::new();
static REMOTE_URL: Mutex<String> = Mutex::new(String::new());

// ═══════════════════════════════════════════════════════════════════════════
// Init — called once at startup
// ═══════════════════════════════════════════════════════════════════════════

/// Load config from `<exe_dir>/rcm.config.json`, or create it with defaults.
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

    DEV_MODE.store(cfg.dev, Ordering::Relaxed);
    SHOW_ICONS.store(cfg.icons, Ordering::Relaxed);
    let _ = FILTERS.set(cfg.filters);
    *REMOTE_URL.lock().unwrap() = cfg.url;

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
// Getters
// ═══════════════════════════════════════════════════════════════════════════

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

/// Return the remote menu update URL, or `None` if not configured.
pub fn remote_url() -> Option<String> {
    let url = REMOTE_URL.lock().unwrap();
    if url.is_empty() { None } else { Some(url.clone()) }
}

// ═══════════════════════════════════════════════════════════════════════════
// Setters — update memory + persist to disk
// ═══════════════════════════════════════════════════════════════════════════

pub fn set_dev(dev: bool) {
    DEV_MODE.store(dev, Ordering::Relaxed);
    save();
}

pub fn set_icons(icons: bool) {
    SHOW_ICONS.store(icons, Ordering::Relaxed);
    save();
}

pub fn set_remote_url(url: String) {
    *REMOTE_URL.lock().unwrap() = url;
    save();
}

/// Reset all config and menu files to embedded defaults.
pub fn reset() {
    DEV_MODE.store(false, Ordering::Relaxed);
    SHOW_ICONS.store(false, Ordering::Relaxed);
    *REMOTE_URL.lock().unwrap() = String::new();
    save();
    crate::menu_defaults::write_menu_defaults();
    println!("config: reset to defaults");
}

// ═══════════════════════════════════════════════════════════════════════════
// Internal
// ═══════════════════════════════════════════════════════════════════════════

fn config_path() -> PathBuf {
    let mut p = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    p.set_file_name("rcm.config.json");
    p
}

fn save() {
    let cfg = ConfigFile {
        dev: DEV_MODE.load(Ordering::Relaxed),
        icons: SHOW_ICONS.load(Ordering::Relaxed),
        filters: filters().to_vec(),
        url: REMOTE_URL.lock().unwrap().clone(),
    };
    save_inner(&config_path(), &cfg);
}

fn save_inner(path: &std::path::Path, cfg: &ConfigFile) {
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}
