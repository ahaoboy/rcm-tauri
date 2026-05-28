//! `@copy-path` — Copy file path(s) to clipboard (slash-separated).
//! Falls back to the current directory path when no files are selected.

use crate::types::CommandPayload;
use super::SystemCmdResult;
use clipboard_rs::{Clipboard, ClipboardContext};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    // Collect file paths or fall back to cwd
    let mut paths: Vec<String> = cmd.args.iter().map(|s| s.as_str().to_owned()).collect();

    if paths.is_empty() {
        // No file args → use cwd as fallback
        if cmd.cwd.is_empty() {
            return SystemCmdResult { success: false, message: "No path to copy".into() };
        }
        paths.push(cmd.cwd.clone());
    }

    // Convert backslashes to forward slashes
    let text = paths
        .iter()
        .map(|p| p.replace('\\', "/"))
        .collect::<Vec<_>>()
        .join("\n");

    let ctx = match ClipboardContext::new() {
        Ok(c) => c,
        Err(e) => return SystemCmdResult {
            success: false,
            message: format!("Failed to open clipboard: {e}"),
        },
    };

    match ctx.set_text(text) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Copied {} path(s) to clipboard", paths.len()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Failed to set clipboard: {e}"),
        },
    }
}
