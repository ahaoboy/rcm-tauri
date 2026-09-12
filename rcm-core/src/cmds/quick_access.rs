//! `@add-to-quick-access` / `@remove-from-quick-access` — Add or remove
//! a file/folder from the Windows Quick Access pane in File Explorer.
//!
//! Uses the `quick-access` library (direct COM calls via `IShellItem` /
//! `IContextMenu`) instead of PowerShell.

use super::SystemCmdResult;
use crate::types::CommandPayload;

/// Run `@add-to-quick-access` — pin a file/folder to Quick Access.
pub fn run_add(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No path specified".into(),
            };
        }
    };

    crate::log::info(
        "Rust::add_to_quick_access",
        &format!("adding '{path}' to Quick Access"),
    );

    match quick_access::add(path) {
        Ok(()) => {
            crate::log::info("Rust::add_to_quick_access", "pintohome OK");
            SystemCmdResult {
                success: true,
                message: format!("Added to Quick Access: {path}"),
            }
        }
        Err(e) => {
            let msg = e.to_string();
            crate::log::error("Rust::add_to_quick_access", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}

/// Run `@remove-from-quick-access` — unpin a file/folder from Quick Access.
pub fn run_remove(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No path specified".into(),
            };
        }
    };

    crate::log::info(
        "Rust::remove_from_quick_access",
        &format!("removing '{path}' from Quick Access"),
    );

    match quick_access::remove(path) {
        Ok(()) => {
            crate::log::info("Rust::remove_from_quick_access", "unpinfromhome OK");
            SystemCmdResult {
                success: true,
                message: format!("Removed from Quick Access: {path}"),
            }
        }
        Err(e) => {
            let msg = e.to_string();
            crate::log::error("Rust::remove_from_quick_access", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}

/// List all paths currently pinned to Quick Access.
pub fn list_quick_access() -> Vec<String> {
    match quick_access::list() {
        Ok(entries) => entries
            .into_iter()
            .map(|e| e.path.to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    }
}
