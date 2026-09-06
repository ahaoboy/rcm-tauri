//! `@zip` — Create an archive in a requested format.
//!
//! First argument is the required output format extension (`.zip`,
//! `.tar.gz`, `.7z`, …); remaining args are source files. If no sources are
//! given (background click), the entire current directory is archived. The
//! output name defaults to the first source's stem / directory name with the
//! format extension and collision avoidance.

use super::{SystemCmdResult, unique_path};
use crate::types::CommandPayload;
use easy_archive::Fmt;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let args: Vec<String> = cmd.args.to_vec();

    // Required first arg: format extension — ".zip" | ".tar.gz" | ".7z" | …
    let Some(first) = args.first() else {
        return SystemCmdResult {
            success: false,
            message: "Missing format argument (e.g. \".zip\", \".tar.gz\")".into(),
        };
    };
    let Some(fmt) = Fmt::guess(first) else {
        return SystemCmdResult {
            success: false,
            message: format!("Unknown archive format: '{first}'"),
        };
    };
    let ext = first.clone();
    let sources = args[1..].to_vec();

    // Determine sources and archive name
    let (final_sources, archive) = if sources.is_empty() {
        // Background click — archive the entire current directory
        let dir = if cmd.cwd.is_empty() { "." } else { &cmd.cwd };
        let dir_path = std::path::Path::new(dir);
        let dir_name = dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive");
        let archive_path = unique_path(&dir_path.join(format!("{dir_name}{ext}")));
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
        let archive_path = unique_path(&parent.join(format!("{stem}{ext}")));
        (sources, archive_path.to_string_lossy().into_owned())
    };

    easy_archive::cli::handle_compression(&final_sources, &archive, fmt);

    SystemCmdResult {
        success: true,
        message: format!("Created: {archive}"),
    }
}
