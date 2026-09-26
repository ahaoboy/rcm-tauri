//! `@paste-files` — Paste files from clipboard to a target directory.
//! Uses clipboard-rs for cross-platform file-list retrieval.
//! Mimics Windows Explorer: auto-renames on collision → "name (2).ext", …

use std::fs;
use std::path::Path;

use clipboard_rs::{Clipboard, ClipboardContext};

use super::{CmdError, Command, unique_path};
use crate::types::CommandPayload;

/// `@paste-files` — copy the clipboard's files into `cwd`.
pub struct PasteFiles;

/// Arguments for [`PasteFiles`].
struct Args {
    /// Destination directory.
    dest: String,
}

impl PasteFiles {
    /// Extract the destination directory.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        if payload.cwd.is_empty() {
            return Err(CmdError::missing("cwd", "a destination directory"));
        }
        Ok(Args {
            dest: payload.cwd.clone(),
        })
    }

    /// Copy every clipboard file into `dest`.
    fn execute(args: Args) -> Result<String, CmdError> {
        let ctx = ClipboardContext::new()
            .map_err(|e| CmdError::failed(format!("Failed to open clipboard: {e}")))?;
        let files = ctx
            .get_files()
            .map_err(|_| CmdError::failed("No files in clipboard"))?;

        let dest = Path::new(&args.dest);
        let mut copied = 0usize;
        let mut errors = 0usize;

        for path in &files {
            let src = Path::new(path);
            let file_name = src.file_name().unwrap_or_default();
            // Windows-style auto-rename on collision: cookies.txt → cookies (2).txt
            let dst = unique_path(&dest.join(file_name));

            let result = if src.is_dir() {
                copy_dir_recursive(src, &dst)
            } else {
                fs::copy(src, &dst).map(|_| ())
            };

            match result {
                Ok(()) => copied += 1,
                Err(_) => errors += 1,
            }
        }

        let mut message = format!("Pasted {copied} file(s)");
        if errors > 0 {
            message.push_str(&format!(" ({errors} failed)"));
        }

        if copied == 0 {
            return Err(CmdError::failed(message));
        }
        Ok(message)
    }
}

impl Command for PasteFiles {
    fn id(&self) -> &'static str {
        "@paste-files"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
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
