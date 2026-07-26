//! `@open-file-location` — Open the containing folder in Explorer and
//! select the file. For shortcut (.lnk) files, resolves the target first.

use super::SystemCmdResult;
use crate::types::CommandPayload;
use std::path::Path;

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

    let mut cmd = crate::sys_cmd("explorer");

    if is_dir {
        cmd.arg(&target);
    } else {
        cmd.arg("/select,").arg(&target);
    }

    match cmd.spawn() {
        Ok(_) => {
            crate::log::info("Rust::open_file_location", "explorer launched OK");
            SystemCmdResult {
                success: true,
                message: format!("Opened location for: {target}"),
            }
        }
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Failed to launch explorer: {e}"),
        },
    }
}

/// Resolve a Windows shortcut (.lnk) to its target path using `lnk_com`.
fn resolve_shortcut(lnk_path: &str) -> Result<String, String> {
    let link = lnk_com::resolve(Path::new(lnk_path)).map_err(|e| e.to_string())?;
    link.link_target()
        .ok_or_else(|| "shortcut has no target".into())
}
