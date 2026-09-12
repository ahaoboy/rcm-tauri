//! `@unzip` — Extract one or more archives.
//!
//! Each archive is extracted into a subdirectory named after the
//! archive's stem (e.g. `foo.zip` → `foo/`).  If the directory
//! already exists, a collision-safe name is chosen (`foo (2)/`, …).

use std::path::Path;

use easy_archive::Fmt;

use super::{SystemCmdResult, unique_path};
use crate::types::CommandPayload;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let archives: Vec<String> = cmd.args.to_vec();

    if archives.is_empty() {
        return SystemCmdResult {
            success: false,
            message: "No archives specified".into(),
        };
    }

    let base_dir = if cmd.cwd.is_empty() { "." } else { &cmd.cwd };
    let mut extracted = Vec::new();

    for archive in &archives {
        let Some(fmt) = Fmt::guess(archive) else {
            return SystemCmdResult {
                success: false,
                message: format!("Unsupported archive format: {archive}"),
            };
        };

        let stem = Path::new(archive)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("extracted");
        // Handle double extensions like .tar.gz
        let stem = stem.trim_end_matches(".tar");
        let dest = unique_path(&Path::new(base_dir).join(stem));

        if let Err(e) = easy_archive::cli::handle_decompression(
            archive,
            &dest.to_string_lossy(),
            fmt,
        ) {
            return SystemCmdResult {
                success: false,
                message: format!("Failed to extract '{archive}': {e}"),
            };
        }
        extracted.push(dest.to_string_lossy().into_owned());
    }

    SystemCmdResult {
        success: true,
        message: format!(
            "Extracted {} archive(s): {}",
            archives.len(),
            extracted.join(", ")
        ),
    }
}
