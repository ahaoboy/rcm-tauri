//! `@trash` — Move file(s) to the recycle bin using the `trash` crate.

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@trash` — send the given files/folders to the recycle bin.
pub struct Trash;

/// Arguments for [`Trash`].
struct Args {
    /// Paths to send to the recycle bin.
    paths: Vec<String>,
}

impl Trash {
    /// Extract the paths to trash.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            paths: CmdArgs::of(payload)
                .all("path", "one or more file or folder paths")?
                .to_vec(),
        })
    }

    /// Move every path to the recycle bin.
    fn execute(args: Args) -> Result<String, CmdError> {
        let refs: Vec<&str> = args.paths.iter().map(String::as_str).collect();
        trash::delete_all(&refs)
            .map_err(|e| CmdError::failed(format!("Failed to move to recycle bin: {e}")))?;
        Ok(format!("Moved {} item(s) to recycle bin", args.paths.len()))
    }
}

impl Command for Trash {
    fn id(&self) -> &'static str {
        "@trash"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
