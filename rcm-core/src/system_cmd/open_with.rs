//! `@open-with` — Open the Windows "Open With → Choose another app" dialog.
//!
//! Spawns PowerShell in the background to invoke the `openas` shell verb
//! via `Shell.Application` COM — the exact equivalent of right-click →
//! "Open with" → "Choose another app" in Windows Explorer.
//!
//! Uses `spawn` (fire-and-forget) rather than `output` to avoid blocking
//! the Rust async runtime and to prevent the dialog activation from
//! being misinterpreted as a new right-click event.

use super::SystemCmdResult;
use crate::types::CommandPayload;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let path = cmd.args.first().map(|s| s.as_str()).unwrap_or("");

    if path.is_empty() {
        return SystemCmdResult {
            success: false,
            message: "@open-with requires a file path argument".into(),
        };
    }

    // Escape single quotes for PowerShell string interpolation
    let escaped = path.replace('\'', "''");

    let script = format!(
        "$f=gi -LiteralPath '{escaped}';\
         (New-Object -ComObject Shell.Application).Namespace($f.DirectoryName).ParseName($f.Name).InvokeVerb('openas')"
    );

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .spawn()
    {
        Ok(_) => SystemCmdResult {
            success: true,
            message: format!("OpenWith dialog launched for: {path}"),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("OpenWith failed to launch: {e}"),
        },
    }
}
