//! `@copy-target` — Resolve a .lnk shortcut and copy its target path to
//! the clipboard.
//!
//! Only works for single `.lnk` file selections.

use super::SystemCmdResult;
use crate::types::CommandPayload;
use clipboard_rs::{Clipboard, ClipboardContext};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No .lnk file specified".into(),
            };
        }
    };

    let lnk_path = std::path::Path::new(path);
    let target = match lnk_com::resolve(lnk_path) {
        Ok(link) => match link.link_target() {
            Some(t) => t.to_string(),
            None => {
                return SystemCmdResult {
                    success: false,
                    message: format!("'{path}' has no target"),
                };
            }
        },
        Err(e) => {
            return SystemCmdResult {
                success: false,
                message: format!("Failed to resolve '{path}': {e}"),
            };
        }
    };

    crate::log::info(
        "Rust::copy_target",
        &format!("'{path}' → '{target}'"),
    );

    let ctx = match ClipboardContext::new() {
        Ok(c) => c,
        Err(e) => {
            return SystemCmdResult {
                success: false,
                message: format!("Failed to open clipboard: {e}"),
            };
        }
    };

    // Convert backslashes to forward slashes (consistent with @copy-path)
    let target = target.replace('\\', "/");

    match ctx.set_text(target.clone()) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Copied target: {target}"),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Failed to set clipboard: {e}"),
        },
    }
}
