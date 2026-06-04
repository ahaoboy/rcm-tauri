//! `@rename` — Rename a file or folder with collision avoidance.

use super::{SystemCmdResult, unique_path};
use crate::types::CommandPayload;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let src = cmd.args.first().cloned().unwrap_or_default();
    let new_name = cmd.args.get(1).cloned().unwrap_or_default();

    let src_path = std::path::Path::new(&src);
    let parent = src_path.parent().unwrap_or(std::path::Path::new("."));
    let dest = parent.join(&new_name);

    let dest = unique_path(&dest);

    match std::fs::rename(&src, &dest) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Renamed to: {}", dest.display()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Rename failed: {e}"),
        },
    }
}
