//! `@new-folder` — Create a new folder.
//!
//! The base name comes from the first argument (defaults to `New folder`).
//! The final path is resolved relative to `cwd` with collision avoidance.

use std::path::PathBuf;

use super::{CmdArgs, CmdError, Command, cwd_dir, unique_path};
use crate::types::CommandPayload;

/// `@new-folder` — create a folder in `cwd`.
pub struct NewFolder;

/// Arguments for [`NewFolder`].
struct Args {
    /// Directory the folder is created in.
    dir: PathBuf,
    /// Name of the new folder.
    name: String,
}

impl NewFolder {
    /// Resolve the folder name and directory.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            dir: cwd_dir(&payload.cwd),
            name: CmdArgs::of(payload)
                .optional(0)
                .unwrap_or("New folder")
                .to_owned(),
        })
    }

    /// Create the folder with collision avoidance.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = unique_path(&args.dir.join(&args.name));
        std::fs::create_dir(&path)
            .map_err(|e| CmdError::failed(format!("Create folder failed: {e}")))?;
        Ok(format!("Created: {}", path.display()))
    }
}

impl Command for NewFolder {
    fn id(&self) -> &'static str {
        "@new-folder"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
