//! `@paste-files` — Paste files from clipboard to a target directory.
//! Uses clipboard-rs for cross-platform file-list retrieval.
//! Mimics Windows Explorer: auto-renames on collision → "name (2).ext", …

use super::{SystemCmdResult, unique_path};
use crate::types::CommandPayload;
use clipboard_rs::{Clipboard, ClipboardContext};
use std::fs;
use std::path::Path;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    if cmd.cwd.is_empty() {
        return SystemCmdResult {
            success: false,
            message: "No destination directory".into(),
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

    let files = match ctx.get_files() {
        Ok(f) => f,
        Err(_) => {
            return SystemCmdResult {
                success: false,
                message: "No files in clipboard".into(),
            };
        }
    };

    let mut copied: usize = 0;
    let mut errors: usize = 0;
    let dest = Path::new(&cmd.cwd);

    for path_str in &files {
        let src = Path::new(path_str);
        let file_name = src.file_name().unwrap_or_default();
        // Windows-style auto-rename on collision: cookies.txt → cookies (2).txt
        let dst = unique_path(&dest.join(file_name));

        if src.is_dir() {
            match copy_dir_recursive(src, &dst) {
                Ok(()) => copied += 1,
                Err(_) => errors += 1,
            }
        } else {
            match fs::copy(src, &dst) {
                Ok(_) => copied += 1,
                Err(_) => errors += 1,
            }
        }
    }

    let mut msg = format!("Pasted {copied} file(s)");
    if errors > 0 {
        msg.push_str(&format!(" ({errors} failed)"));
    }

    SystemCmdResult {
        success: copied > 0,
        message: msg,
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
