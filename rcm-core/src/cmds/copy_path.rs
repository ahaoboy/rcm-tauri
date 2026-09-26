//! `@copy-path` — Copy file path(s) to clipboard (slash-separated).
//! Falls back to the current directory path when no files are selected.

use clipboard_rs::{Clipboard, ClipboardContext};

use super::{CmdError, Command};
use crate::types::CommandPayload;

/// `@copy-path` — copy the selected paths (or the cwd) as forward-slash text.
pub struct CopyPath;

/// Arguments for [`CopyPath`].
struct Args {
    /// Paths to copy; falls back to `cwd` when nothing is selected.
    paths: Vec<String>,
}

impl CopyPath {
    /// Resolve the path list, defaulting to `cwd`.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let paths = if payload.args.is_empty() {
            if payload.cwd.is_empty() {
                return Err(CmdError::missing("path", "one or more paths"));
            }
            vec![payload.cwd.clone()]
        } else {
            payload.args.clone()
        };
        Ok(Args { paths })
    }

    /// Put the forward-slash paths on the clipboard.
    fn execute(args: Args) -> Result<String, CmdError> {
        let text = args
            .paths
            .iter()
            .map(|path| path.replace('\\', "/"))
            .collect::<Vec<_>>()
            .join("\n");

        let ctx = ClipboardContext::new()
            .map_err(|e| CmdError::failed(format!("Failed to open clipboard: {e}")))?;
        ctx.set_text(text)
            .map_err(|e| CmdError::failed(format!("Failed to set clipboard: {e}")))?;
        Ok(format!("Copied {} path(s) to clipboard", args.paths.len()))
    }
}

impl Command for CopyPath {
    fn id(&self) -> &'static str {
        "@copy-path"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
