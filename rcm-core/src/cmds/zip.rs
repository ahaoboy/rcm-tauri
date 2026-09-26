//! `@zip` — Create an archive in a requested format.
//!
//! First argument is the required output format extension (`.zip`,
//! `.tar.gz`, `.7z`, …); remaining args are source files. If no sources are
//! given (background click), the entire current directory is archived. The
//! output name defaults to the first source's stem / directory name with the
//! format extension and collision avoidance.

use std::path::{Path, PathBuf};

use easy_archive::Fmt;

use super::{CmdArgs, CmdError, Command, cwd_dir, unique_path};
use crate::types::CommandPayload;

/// Expected form of the `format` argument, reused by both error sites.
const FORMAT_EXPECTED: &str = "an archive extension (e.g. .zip, .tar.gz, .7z)";

/// `@zip` — compress sources into a new archive.
pub struct Zip;

/// Arguments for [`Zip`].
struct Args {
    /// Output format extension, e.g. `.zip`.
    ext: String,
    /// Parsed archive format.
    fmt: Fmt,
    /// Files to compress; empty means the whole current directory.
    sources: Vec<String>,
    /// Directory used when no sources were selected.
    cwd: PathBuf,
}

impl Zip {
    /// Extract the format and source list.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let args = CmdArgs::of(payload);
        let ext = args.required(0, "format", FORMAT_EXPECTED)?;
        let fmt = Fmt::guess(ext)
            .ok_or_else(|| CmdError::invalid("format", ext, FORMAT_EXPECTED))?;
        Ok(Args {
            ext: ext.to_owned(),
            fmt,
            sources: args.tail(1).to_vec(),
            cwd: cwd_dir(&payload.cwd),
        })
    }

    /// Compress the sources into a new archive.
    fn execute(args: Args) -> Result<String, CmdError> {
        let ext = &args.ext;
        let (sources, archive) = if args.sources.is_empty() {
            // Background click — archive the entire current directory.
            let dir_name = args
                .cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("archive");
            let archive = unique_path(&args.cwd.join(format!("{dir_name}{ext}")));
            (
                vec![args.cwd.to_string_lossy().into_owned()],
                archive.to_string_lossy().into_owned(),
            )
        } else {
            // Files selected — name the archive after the first file.
            let first = Path::new(&args.sources[0]);
            let stem = first
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("archive");
            let parent = first.parent().unwrap_or_else(|| Path::new("."));
            let archive = unique_path(&parent.join(format!("{stem}{ext}")));
            (args.sources, archive.to_string_lossy().into_owned())
        };

        easy_archive::cli::handle_compression(&sources, &archive, args.fmt)
            .map_err(|e| CmdError::failed(format!("Failed to create '{archive}': {e}")))?;

        Ok(format!("Created: {archive}"))
    }
}

impl Command for Zip {
    fn id(&self) -> &'static str {
        "@zip"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
