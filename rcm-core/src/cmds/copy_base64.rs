//! `@copy-base64` — Copy selected file(s) content as base64 to clipboard.
//! Each file is encoded separately and joined with newlines.
//! Falls back to encoding the current directory path when no files are selected.

use std::fs;
use std::path::Path;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use clipboard_rs::{Clipboard, ClipboardContext};

use super::{CmdError, Command};
use crate::types::CommandPayload;

/// Maximum file size to read (10 MB). Larger files are skipped with a warning.
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// `@copy-base64` — encode the selected files' contents as base64 text.
pub struct CopyBase64;

/// Arguments for [`CopyBase64`].
struct Args {
    /// Files to encode; falls back to `cwd` when nothing is selected.
    paths: Vec<String>,
}

impl CopyBase64 {
    /// Resolve the file list, defaulting to `cwd`.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let paths = if payload.args.is_empty() {
            if payload.cwd.is_empty() {
                return Err(CmdError::missing("path", "one or more file paths"));
            }
            vec![payload.cwd.clone()]
        } else {
            payload.args.clone()
        };
        Ok(Args { paths })
    }

    /// Encode each file and put the result on the clipboard.
    fn execute(args: Args) -> Result<String, CmdError> {
        let mut encoded: Vec<String> = Vec::with_capacity(args.paths.len());
        let mut skipped = 0usize;

        for path in &args.paths {
            match encode_file(Path::new(path)) {
                Some(text) => encoded.push(text),
                None => skipped += 1,
            }
        }

        if encoded.is_empty() {
            return Err(CmdError::failed(if skipped > 0 {
                format!("No files encoded ({skipped} skipped)")
            } else {
                "No files to encode".into()
            }));
        }

        let mut message = format!("Copied {} file(s) as base64", encoded.len());
        if skipped > 0 {
            message.push_str(&format!(" ({skipped} skipped)"));
        }

        let ctx = ClipboardContext::new()
            .map_err(|e| CmdError::failed(format!("Failed to open clipboard: {e}")))?;
        ctx.set_text(encoded.join("\n"))
            .map_err(|e| CmdError::failed(format!("Failed to set clipboard: {e}")))?;
        Ok(message)
    }
}

impl Command for CopyBase64 {
    fn id(&self) -> &'static str {
        "@copy-base64"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// Base64-encode a file, or `None` when it is a directory, too large or unreadable.
fn encode_file(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.is_dir() || metadata.len() > MAX_FILE_SIZE {
        return None;
    }
    fs::read(path).ok().map(|bytes| BASE64.encode(bytes))
}
