//! `@unzip` — Extract one or more archives.
//!
//! Each archive is extracted into a subdirectory named after the
//! archive's stem (e.g. `foo.zip` → `foo/`).  If the directory
//! already exists, a collision-safe name is chosen (`foo (2)/`, …).

use std::path::{Path, PathBuf};

use easy_archive::Fmt;

use super::{CmdArgs, CmdError, Command, cwd_dir, unique_path};
use crate::types::CommandPayload;

/// `@unzip` — extract archives next to `cwd`.
pub struct Unzip;

/// Arguments for [`Unzip`].
struct Args {
    /// Archives to extract.
    archives: Vec<String>,
    /// Directory the archives are extracted into.
    base_dir: PathBuf,
}

impl Unzip {
    /// Extract the archive list and base directory.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            archives: CmdArgs::of(payload)
                .all("archive", "one or more archive paths")?
                .to_vec(),
            base_dir: cwd_dir(&payload.cwd),
        })
    }

    /// Extract every archive into its own subdirectory.
    fn execute(args: Args) -> Result<String, CmdError> {
        let mut extracted = Vec::with_capacity(args.archives.len());

        for archive in &args.archives {
            let fmt = Fmt::guess(archive).ok_or_else(|| {
                CmdError::invalid("archive", archive.as_str(), "a supported archive format")
            })?;

            let stem = Path::new(archive)
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("extracted");
            // Handle double extensions like .tar.gz
            let stem = stem.trim_end_matches(".tar");
            let dest = unique_path(&args.base_dir.join(stem));

            easy_archive::cli::handle_decompression(archive, &dest.to_string_lossy(), fmt)
                .map_err(|e| CmdError::failed(format!("Failed to extract '{archive}': {e}")))?;

            extracted.push(dest.to_string_lossy().into_owned());
        }

        Ok(format!(
            "Extracted {} archive(s): {}",
            args.archives.len(),
            extracted.join(", ")
        ))
    }
}

impl Command for Unzip {
    fn id(&self) -> &'static str {
        "@unzip"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
