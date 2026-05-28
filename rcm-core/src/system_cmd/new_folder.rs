//! `@new-folder` — Create a new folder.
//!
//! The base name comes from the first argument (defaults to
//! `New folder`).  The final path is resolved relative to
//! `cmd.cwd` with collision avoidance.

use crate::types::CommandPayload;
use super::{SystemCmdResult, unique_path};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let name = cmd.args.first().map(|s| s.as_str()).unwrap_or("New folder");
    let dir = if cmd.cwd.is_empty() { "." } else { &cmd.cwd };
    let path = unique_path(&std::path::Path::new(dir).join(name));

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
