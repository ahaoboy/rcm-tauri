//! `@copy` — Copy selected file(s) to the system clipboard as file-drop data.

use super::SystemCmdResult;
use crate::types::CommandPayload;
use clipboard_rs::{Clipboard, ClipboardContext};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let paths: Vec<&str> = cmd.args.iter().map(|s| s.as_str()).collect();
    if paths.is_empty() {
        return SystemCmdResult {
            success: false,
            message: "No files specified".into(),
        };
    }

    let ctx = match ClipboardContext::new() {
        Ok(c) => c,
        Err(e) => {
            return SystemCmdResult {
                success: false,
                message: format!("Failed to open clipboard: {e}"),
            };
        }
    };

    match ctx.set_files(paths.iter().map(|s| s.to_string()).collect()) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Copied {} item(s) to clipboard", paths.len()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Failed to copy to clipboard: {e}"),
        },
    }
}
