//! `@pin-to-start` / `@unpin-from-start` — Pin or unpin a file to the
//! Windows Start Menu by creating/removing a shortcut in
//! `%APPDATA%\Microsoft\Windows\Start Menu\Programs`.
//!
//! Uses the [`startmenu`] crate for file-level API (no external shell).

use super::SystemCmdResult;
use crate::types::CommandPayload;
use startmenu::{self, Scope};
use std::path::Path;

/// Run `@pin-to-start` — create a shortcut in the Start Menu Programs folder.
pub fn run_pin(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No file specified".into(),
            };
        }
    };

    let target = Path::new(path);
    let name = match target.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => {
            return SystemCmdResult {
                success: false,
                message: format!("Cannot extract file name from '{path}'"),
            };
        }
    };

    crate::log::info(
        "Rust::pin_to_start",
        &format!("pinning '{path}' as '{name}'"),
    );

    match startmenu::add(Scope::User, name, target, None) {
        Ok(lnk_path) => {
            crate::log::info("Rust::pin_to_start", "shortcut created OK");
            SystemCmdResult {
                success: true,
                message: format!("Pinned to Start: {}", lnk_path.display()),
            }
        }
        Err(e) => {
            crate::log::error("Rust::pin_to_start", &e.to_string());
            SystemCmdResult {
                success: false,
                message: e.to_string(),
            }
        }
    }
}

/// Run `@unpin-from-start` — remove the shortcut from the Start Menu.
pub fn run_unpin(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No file specified".into(),
            };
        }
    };

    crate::log::info(
        "Rust::unpin_from_start",
        &format!("unpinning '{path}'"),
    );

    match startmenu::remove(Path::new(path)) {
        Ok(removed) if removed.is_empty() => {
            crate::log::info("Rust::unpin_from_start", "no matching shortcut found");
            SystemCmdResult {
                success: true,
                message: "Already not pinned".into(),
            }
        }
        Ok(removed) => {
            crate::log::info("Rust::unpin_from_start", "shortcut removed OK");
            SystemCmdResult {
                success: true,
                message: format!(
                    "Unpinned from Start: {}",
                    removed.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
                ),
            }
        }
        Err(e) => {
            crate::log::error("Rust::unpin_from_start", &e.to_string());
            SystemCmdResult {
                success: false,
                message: e.to_string(),
            }
        }
    }
}

/// Check whether a file is pinned to Start.
pub fn is_pinned_to_start(path: &str) -> bool {
    startmenu::exists(Path::new(path)).ok().flatten().is_some()
}

/// List all items pinned to the Start Menu as [`crate::types::Entry`]s
/// (user + machine scopes, with resolved args and target).
pub fn list_pinned_to_start() -> Vec<crate::types::Entry> {
    startmenu::list()
        .unwrap_or_default()
        .into_iter()
        .map(|lnk| crate::types::Entry {
            path: lnk.path.to_string_lossy().into_owned(),
            args: lnk.args.clone(),
            target: lnk.target().ok().flatten(),
        })
        .collect()
}
