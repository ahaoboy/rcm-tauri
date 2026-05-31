//! `@copy-name` — Copy file name(s) to clipboard (newline-separated).
//! Falls back to the current directory name when no files are selected.

use crate::types::CommandPayload;
use super::SystemCmdResult;
use clipboard_rs::{Clipboard, ClipboardContext};
use std::path::Path;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let mut names: Vec<String> = Vec::new();

    if cmd.args.is_empty() {
        // No file args → use the last component of cwd
        if cmd.cwd.is_empty() {
            return SystemCmdResult { success: false, message: "No name to copy".into() };
        }
        let name = Path::new(&cmd.cwd)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| cmd.cwd.clone());
        names.push(name);
    } else {
        for path in &cmd.args {
            let name = Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            names.push(name);
        }
    }

    let text = names.join("\n");

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
            message: format!("Copied {} name(s) to clipboard", names.len()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Failed to set clipboard: {e}"),
        },
    }
}
