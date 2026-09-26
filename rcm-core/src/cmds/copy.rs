//! `@copy` — Copy selected file(s) to the system clipboard as file-drop data.

use clipboard_rs::{Clipboard, ClipboardContext};

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@copy` — place the given files on the clipboard as file-drop data.
pub struct CopyFiles;

/// Arguments for [`CopyFiles`].
struct Args {
    /// Files to place on the clipboard.
    paths: Vec<String>,
}

impl CopyFiles {
    /// Extract the files to copy.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            paths: CmdArgs::of(payload)
                .all("path", "one or more file paths")?
                .to_vec(),
        })
    }

    /// Put the files on the clipboard.
    fn execute(args: Args) -> Result<String, CmdError> {
        let count = args.paths.len();
        let ctx = ClipboardContext::new()
            .map_err(|e| CmdError::failed(format!("Failed to open clipboard: {e}")))?;
        ctx.set_files(args.paths)
            .map_err(|e| CmdError::failed(format!("Failed to copy to clipboard: {e}")))?;
        Ok(format!("Copied {count} item(s) to clipboard"))
    }
}

impl Command for CopyFiles {
    fn id(&self) -> &'static str {
        "@copy"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
