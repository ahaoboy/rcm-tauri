//! `@pin-to-start` / `@unpin-from-start` — Pin or unpin a file to the
//! Windows Start Menu by creating/removing a shortcut in
//! `%APPDATA%\Microsoft\Windows\Start Menu\Programs`.
//!
//! Only `.exe` and `.lnk` files are supported.

use super::SystemCmdResult;
use crate::types::CommandPayload;

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

    let target = std::path::Path::new(path);
    let file_stem = match target.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => {
            return SystemCmdResult {
                success: false,
                message: format!("Cannot extract file name from '{path}'"),
            };
        }
    };

    let start_menu = match std::env::var("APPDATA") {
        Ok(appdata) => format!(
            "{}\\Microsoft\\Windows\\Start Menu\\Programs\\{file_stem}.lnk",
            appdata
        ),
        Err(e) => {
            return SystemCmdResult {
                success: false,
                message: format!("Cannot get APPDATA: {e}"),
            };
        }
    };

    crate::log::info(
        "Rust::pin_to_start",
        &format!("pinning '{path}' → '{start_menu}'"),
    );

    // Use WScript.Shell COM to create a proper .lnk shortcut.
    let script = format!(
        r#"$ws=New-Object -ComObject WScript.Shell;$s=$ws.CreateShortcut('{start_menu}');$s.TargetPath='{path}';$s.Save()"#
    );

    match crate::sys_cmd("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
    {
        Ok(output) if output.status.success() => {
            crate::log::info("Rust::pin_to_start", "shortcut created OK");
            SystemCmdResult {
                success: true,
                message: format!("Pinned to Start: {path}"),
            }
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let msg = if err.is_empty() {
                "powershell exited non-zero".into()
            } else {
                err
            };
            crate::log::error("Rust::pin_to_start", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
        Err(e) => {
            let msg = format!("failed to spawn powershell: {e}");
            crate::log::error("Rust::pin_to_start", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}

/// Run `@unpin-from-start` — remove the shortcut from the Start Menu
/// Programs folder.
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

    let target = std::path::Path::new(path);
    let file_stem = match target.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => {
            return SystemCmdResult {
                success: false,
                message: format!("Cannot extract file name from '{path}'"),
            };
        }
    };

    let start_menu_lnk = match std::env::var("APPDATA") {
        Ok(appdata) => format!(
            "{}\\Microsoft\\Windows\\Start Menu\\Programs\\{file_stem}.lnk",
            appdata
        ),
        Err(e) => {
            return SystemCmdResult {
                success: false,
                message: format!("Cannot get APPDATA: {e}"),
            };
        }
    };

    crate::log::info(
        "Rust::unpin_from_start",
        &format!("unpinning '{path}' — removing '{start_menu_lnk}'"),
    );

    if !std::path::Path::new(&start_menu_lnk).exists() {
        return SystemCmdResult {
            success: true,
            message: "Already not pinned".into(),
        };
    }

    match std::fs::remove_file(&start_menu_lnk) {
        Ok(()) => {
            crate::log::info("Rust::unpin_from_start", "shortcut removed OK");
            SystemCmdResult {
                success: true,
                message: format!("Unpinned from Start: {path}"),
            }
        }
        Err(e) => {
            let msg = format!("Failed to remove '{start_menu_lnk}': {e}");
            crate::log::error("Rust::unpin_from_start", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}

/// Check whether a file is pinned to Start.
///
/// Two strategies:
/// 1. If the file itself lives inside either the per-user or all-users
///    Start Menu Programs folder → already pinned (this covers `.lnk`
///    files already placed there).
/// 2. Otherwise check whether a `.lnk` with the same file stem exists
///    in either Start Menu Programs folder (covers `.exe` files).
pub fn is_pinned_to_start(path: &str) -> bool {
    let p = std::path::Path::new(path);

    // Candidate Start Menu directories
    let start_dirs: Vec<String> = [
        std::env::var("APPDATA")
            .map(|d| format!("{d}\\Microsoft\\Windows\\Start Menu\\Programs")),
        std::env::var("ProgramData")
            .map(|d| format!("{d}\\Microsoft\\Windows\\Start Menu\\Programs")),
    ]
    .into_iter()
    .flatten()
    .collect();

    // Strategy 1: the file is *inside* a Start Menu folder
    for dir in &start_dirs {
        if p.starts_with(dir) {
            return true;
        }
    }

    // Strategy 2: a .lnk named after this file exists in a Start Menu folder
    let file_stem = match p.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return false,
    };

    for dir in &start_dirs {
        let candidate = std::path::Path::new(dir).join(format!("{file_stem}.lnk"));
        if candidate.exists() {
            return true;
        }
    }

    false
}

/// List all items pinned to the Start Menu.
///
/// Enumerates `.lnk` files in both per-user and all-users Start Menu
/// Programs folders, returning their **file stems** (name without `.lnk`).
/// The frontend can match these against `InvokeProps.files[].path`.
pub fn list_pinned_to_start() -> Vec<String> {
    let start_dirs: Vec<String> = [
        std::env::var("APPDATA")
            .map(|d| format!("{d}\\Microsoft\\Windows\\Start Menu\\Programs")),
        std::env::var("ProgramData")
            .map(|d| format!("{d}\\Microsoft\\Windows\\Start Menu\\Programs")),
    ]
    .into_iter()
    .flatten()
    .collect();

    let mut stems: Vec<String> = Vec::new();
    for dir in &start_dirs {
        let path = std::path::Path::new(dir);
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("lnk") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        stems.push(stem.to_owned());
                    }
                }
            }
        }
    }
    stems
}
