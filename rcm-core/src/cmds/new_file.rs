//! `@new-file` — Create a new empty file.
//!
//! If the first argument starts with `.` (e.g. `.txt`), it is treated as a
//! file extension and the base name defaults to `New File`. Otherwise the
//! argument is the full file name. The final path is resolved relative to
//! `cwd` with collision avoidance.

use std::path::{Path, PathBuf};

use super::{CmdArgs, CmdError, Command, cwd_dir, unique_path};
use crate::types::CommandPayload;

/// `@new-file` — create an empty file in `cwd`.
pub struct NewFile;

/// Arguments for [`NewFile`].
struct Args {
    /// Directory the file is created in.
    dir: PathBuf,
    /// Resolved file name (extension included).
    filename: String,
}

impl NewFile {
    /// Resolve the target name and directory.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let base = CmdArgs::of(payload).optional(0).unwrap_or("New File");

        let (name, ext) = match base.strip_prefix('.') {
            Some(ext) => ("New File", ext),
            None => {
                let path = Path::new(base);
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("New File");
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                (name, ext)
            }
        };

        let filename = if ext.is_empty() {
            name.to_string()
        } else {
            format!("{name}.{ext}")
        };

        Ok(Args {
            dir: cwd_dir(&payload.cwd),
            filename,
        })
    }

    /// Create the file with collision avoidance.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = unique_path(&args.dir.join(&args.filename));
        std::fs::File::create(&path)
            .map_err(|e| CmdError::failed(format!("Create file failed: {e}")))?;
        Ok(format!("Created: {}", path.display()))
    }
}

impl Command for NewFile {
    fn id(&self) -> &'static str {
        "@new-file"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
