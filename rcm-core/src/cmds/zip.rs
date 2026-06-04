//! `@zip` — Create an archive.
//!
//! If source files are provided, they are archived directly.
//! If no sources are given (background click), the entire current
//! directory is archived.  The output name defaults to the directory
//! name with a `.zip` extension, with collision avoidance.

use super::{SystemCmdResult, unique_path};
use crate::types::CommandPayload;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let sources: Vec<String> = cmd.args.to_vec();

    // Determine sources and archive name
    let (final_sources, archive) = if sources.is_empty() {
        // Background click — archive the entire current directory
        let dir = if cmd.cwd.is_empty() { "." } else { &cmd.cwd };
        let dir_path = std::path::Path::new(dir);
        let dir_name = dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive");
        let archive_path = unique_path(&dir_path.join(format!("{dir_name}.zip")));
        (
            vec![dir.to_string()],
            archive_path.to_string_lossy().into_owned(),
        )
    } else {
        // Files selected — use first file's name for the archive
        let first = std::path::Path::new(&sources[0]);
        let stem = first
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("archive");
        let parent = first.parent().unwrap_or(std::path::Path::new("."));
        let archive_path = unique_path(&parent.join(format!("{stem}.zip")));
        (sources, archive_path.to_string_lossy().into_owned())
    };

    let fmt = match easy_archive::Fmt::guess(&archive) {
        Some(f) => f,
        None => {
            return SystemCmdResult {
                success: false,
                message: format!("Unsupported archive format: {archive}"),
            };
        }
    };

    easy_archive::cli::handle_compression(&final_sources, &archive, fmt);

    SystemCmdResult {
        success: true,
        message: format!("Created: {archive}"),
    }
}
