//! `@new-folder` — Create a new folder.

use crate::rcm::CommandPayload;
use super::{SystemCmdResult, unique_path};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let path = cmd.args.first().cloned().unwrap_or_default();
    let path = unique_path(std::path::Path::new(&path));

    match std::fs::create_dir(&path) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Created: {}", path.display()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Create folder failed: {e}"),
        },
    }
}
