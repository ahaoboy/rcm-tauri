//! `@trash` — Move file(s) to the recycle bin using the `trash` crate.

use crate::types::CommandPayload;
use super::SystemCmdResult;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let paths: Vec<&str> = cmd.args.iter().map(|s| s.as_str()).collect();
    if paths.is_empty() {
        return SystemCmdResult { success: false, message: "No files specified".into() };
    }

    match trash::delete_all(&paths) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Moved {} item(s) to recycle bin", paths.len()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Failed to move to recycle bin: {e}"),
        },
    }
}
