//! `@copy-name` — Copy file name(s) to clipboard (newline-separated).
//! Falls back to the current directory name when no files are selected.

use std::path::Path;

use clipboard_rs::{Clipboard, ClipboardContext};

use super::{CmdError, Command};
use crate::types::CommandPayload;

/// `@copy-name` — copy the base name of each selected path.
pub struct CopyName;

/// Arguments for [`CopyName`].
struct Args {
    /// Base names to copy.
    names: Vec<String>,
}

impl CopyName {
    /// Resolve the base-name list, defaulting to the `cwd` name.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let names = if payload.args.is_empty() {
            if payload.cwd.is_empty() {
                return Err(CmdError::missing("path", "one or more paths"));
            }
            vec![file_name(&payload.cwd)]
        } else {
            payload.args.iter().map(|path| file_name(path)).collect()
        };
        Ok(Args { names })
    }

    /// Put the names on the clipboard, one per line.
    fn execute(args: Args) -> Result<String, CmdError> {
        let ctx = ClipboardContext::new()
            .map_err(|e| CmdError::failed(format!("Failed to open clipboard: {e}")))?;
        ctx.set_text(args.names.join("\n"))
            .map_err(|e| CmdError::failed(format!("Failed to set clipboard: {e}")))?;
        Ok(format!("Copied {} name(s) to clipboard", args.names.len()))
    }
}

impl Command for CopyName {
    fn id(&self) -> &'static str {
        "@copy-name"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// Last path component, or the whole string when it has none.
fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_owned())
}
