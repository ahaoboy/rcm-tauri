//! `@copy-path` — Copy selected file path(s) to the clipboard.

use crate::rcm::CommandPayload;
use super::{SystemCmdResult, powershell};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let text = cmd.args.join("\n");
    if text.is_empty() {
        return SystemCmdResult { success: false, message: "No paths to copy".into() };
    }

    let script = format!("Set-Clipboard -Value '{text}'");
    match powershell(&script) {
        Ok(_) => SystemCmdResult { success: true, message: "Paths copied to clipboard".into() },
        Err(e) => SystemCmdResult { success: false, message: e },
    }
}
