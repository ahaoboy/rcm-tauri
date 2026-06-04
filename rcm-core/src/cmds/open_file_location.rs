//! `@open-file-location` — Open the containing folder in Explorer and
//! select the file. For shortcut (.lnk) files, resolves the target first
//! and opens that location instead.

use super::{SystemCmdResult, powershell};
use crate::types::CommandPayload;
use std::os::windows::process::CommandExt;
use std::path::Path;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
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
        "Rust::open_file_location",
        &format!("opening location for '{path}'"),
    );

    // Resolve shortcut target if it's a .lnk file
    let target = if path.to_lowercase().ends_with(".lnk") {
        match resolve_shortcut(path) {
            Ok(t) => {
                crate::log::info(
                    "Rust::open_file_location",
                    &format!("resolved shortcut '{path}' -> '{t}'"),
                );
                t
            }
            Err(e) => {
                crate::log::info(
                    "Rust::open_file_location",
                    &format!("shortcut resolve failed: {e}, falling back to .lnk itself"),
                );
                path.to_string()
            }
        }
    } else {
        path.to_string()
    };

    // If the target is a directory, open it directly. Otherwise use /select
    // to highlight the file in its parent folder.
    let is_dir = Path::new(&target).is_dir();
    let explorer_arg = if is_dir {
        target.clone()
    } else {
        format!("/select,{target}")
    };
    let msg = format!("Opened location: {explorer_arg}");

    match std::process::Command::new("explorer")
        .arg(&explorer_arg)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
    {
        Ok(_) => {
            crate::log::info("Rust::open_file_location", "explorer launched OK");
            SystemCmdResult {
                success: true,
                message: msg,
            }
        }
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Failed to launch explorer: {e}"),
        },
    }
}

/// Resolve a Windows shortcut (.lnk) to its target path using WScript.Shell.
fn resolve_shortcut(lnk_path: &str) -> Result<String, String> {
    let script = format!(
        "$w=New-Object -ComObject WScript.Shell;$w.CreateShortcut('{}').TargetPath",
        lnk_path.replace('\'', "''")
    );
    powershell(&script)
}
