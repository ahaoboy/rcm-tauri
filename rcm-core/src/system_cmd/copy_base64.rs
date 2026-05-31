//! `@copy-base64` — Copy selected file(s) content as base64 to clipboard.
//! Each file is encoded separately and joined with newlines.
//! Falls back to encoding the current directory path when no files are selected.

use crate::types::CommandPayload;
use super::SystemCmdResult;
use clipboard_rs::{Clipboard, ClipboardContext};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::fs;
use std::path::Path;

/// Maximum file size to read (10 MB). Larger files are skipped with a warning.
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let mut encoded: Vec<String> = Vec::new();
    let mut skipped: usize = 0;

    let paths: Vec<&str> = if cmd.args.is_empty() {
        if cmd.cwd.is_empty() {
            return SystemCmdResult { success: false, message: "Nothing to copy".into() };
        }
        vec![cmd.cwd.as_str()]
    } else {
        cmd.args.iter().map(|s| s.as_str()).collect()
    };

    for path_str in &paths {
        let path = Path::new(path_str);

        // Skip directories — can't base64-encode a directory
        if path.is_dir() {
            skipped += 1;
            continue;
        }

        // Check file size before reading
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_e) => {
                skipped += 1;
                continue;
            }
        };

        if metadata.len() > MAX_FILE_SIZE {
            skipped += 1;
            continue;
        }

        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        encoded.push(BASE64.encode(&bytes));
    }

    if encoded.is_empty() {
        return SystemCmdResult {
            success: false,
            message: if skipped > 0 {
                format!("No files encoded ({skipped} skipped)")
            } else {
                "No files to encode".into()
            },
        };
    }

    let text = encoded.join("\n");

    let ctx = match ClipboardContext::new() {
        Ok(c) => c,
        Err(e) => return SystemCmdResult {
            success: false,
            message: format!("Failed to open clipboard: {e}"),
        },
    };

    let mut msg = format!("Copied {} file(s) as base64", encoded.len());
    if skipped > 0 {
        msg.push_str(&format!(" ({skipped} skipped)"));
    }

    match ctx.set_text(text) {
        Ok(()) => SystemCmdResult { success: true, message: msg },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Failed to set clipboard: {e}"),
        },
    }
}
