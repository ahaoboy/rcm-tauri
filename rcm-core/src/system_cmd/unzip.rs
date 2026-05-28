//! `@unzip` — Extract one or more archives.
//!
//! Each archive is extracted into a subdirectory named after the
//! archive's stem (e.g. `foo.zip` → `foo/`).  If the directory
//! already exists, a collision-safe name is chosen (`foo (2)/`, …).

use crate::types::CommandPayload;
use super::{SystemCmdResult, unique_path};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let archives: Vec<String> = cmd.args.iter().cloned().collect();

    if archives.is_empty() {
        return SystemCmdResult {
            success: false,
            message: "No archives specified".into(),
        };
    }

    let base_dir = if cmd.cwd.is_empty() { "." } else { &cmd.cwd };
    let mut extracted = Vec::new();

    for archive in &archives {
        let fmt = match easy_archive::Fmt::guess(archive) {
            Some(f) => f,
            None => {
                return SystemCmdResult {
                    success: false,
                    message: format!("Unsupported archive format: {archive}"),
                };
            }
        };

        let stem = std::path::Path::new(archive)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("extracted");
        // Handle double extensions like .tar.gz
        let stem = stem.trim_end_matches(".tar");
        let dest = unique_path(&std::path::Path::new(base_dir).join(stem));

        easy_archive::cli::handle_decompression(archive, &dest.to_string_lossy(), fmt);
        extracted.push(dest.to_string_lossy().into_owned());
    }

    SystemCmdResult {
        success: true,
        message: format!("Extracted {} archive(s): {}", archives.len(), extracted.join(", ")),
    }
}
