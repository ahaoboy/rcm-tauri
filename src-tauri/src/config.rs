//! Persistent runtime configuration stored as `rmc.config.json`
//! next to the executable.
//!
//! On startup the file is read; if it doesn't exist it is created
//! with defaults.  Tray toggles update the in-memory state and
//! persist immediately.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

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
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// In-memory state (fast atomic reads)
// ═══════════════════════════════════════════════════════════════════════════

static IS_LITE: AtomicBool = AtomicBool::new(true);
static DEV_MODE: AtomicBool = AtomicBool::new(false);
static SHOW_ICONS: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════
// Init — called once at startup
// ═══════════════════════════════════════════════════════════════════════════

/// Load config from `<exe_dir>/rmc.config.json`, or create it with defaults.
pub fn init() {
    let path = config_path();

    let cfg: ConfigFile = match std::fs::read_to_string(&path) {
        Ok(text) => {
            serde_json::from_str(&text).unwrap_or_default()
        }
        Err(_) => {
            let default = ConfigFile::default();
            save_inner(&path, &default);
            default
        }
    };

    IS_LITE.store(cfg.menu == "lite", Ordering::Relaxed);
    DEV_MODE.store(cfg.dev, Ordering::Relaxed);
    SHOW_ICONS.store(cfg.icons, Ordering::Relaxed);

    println!(
        "config: menu={} dev={} icons={} ({})",
        if is_lite() { "lite" } else { "full" },
        is_dev(),
        is_icons(),
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
    crate::vm::write_menu_defaults();
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
        menu: if IS_LITE.load(Ordering::Relaxed) { "lite".into() } else { "full".into() },
        dev: DEV_MODE.load(Ordering::Relaxed),
        icons: SHOW_ICONS.load(Ordering::Relaxed),
    };
    save_inner(&config_path(), &cfg);
}

fn save_inner(path: &std::path::Path, cfg: &ConfigFile) {
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}
