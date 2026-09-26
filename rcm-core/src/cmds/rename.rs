//! `@rename` — Rename a file or folder with collision avoidance.

use std::path::Path;

use super::{CmdArgs, CmdError, Command, unique_path};
use crate::types::CommandPayload;

/// `@rename` — rename a file or folder to `name` inside its parent directory.
pub struct Rename;

/// Arguments for [`Rename`].
struct Args {
    /// File or folder to rename.
    source: std::path::PathBuf,
    /// New name for `source`.
    name: String,
}

impl Rename {
    /// Extract the source path and the new name.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let args = CmdArgs::of(payload);
        Ok(Args {
            source: args.path(0, "source", "a file or folder path")?,
            name: args.required(1, "name", "the new name")?.to_owned(),
        })
    }

    /// Rename `source` within its parent directory.
    fn execute(args: Args) -> Result<String, CmdError> {
        let parent = args.source.parent().unwrap_or_else(|| Path::new("."));
        let dest = unique_path(&parent.join(&args.name));

        std::fs::rename(&args.source, &dest)
            .map_err(|e| CmdError::failed(format!("Rename failed: {e}")))?;

        Ok(format!("Renamed to: {}", dest.display()))
    }
}

impl Command for Rename {
    fn id(&self) -> &'static str {
        "@rename"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
