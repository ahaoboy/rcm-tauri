//! `@eject` — Eject a removable drive.
//!
//! Uses PowerShell + Shell.Application COM to invoke the "Eject" shell verb,
//! exactly replicating the original right-click → Eject behavior.

use super::SystemCmdResult;
use crate::types::CommandPayload;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No drive specified".into(),
            };
        }
    };

    crate::log::info("Rust::eject", &format!("ejecting drive '{path}'"));

    // Shell.Application → Namespace(17) = "This PC" → ParseName finds the
    // drive → InvokeVerb("Eject") fires the same handler as the Win11
    // right-click menu.
    let script = format!(
        r#"(New-Object -ComObject Shell.Application).Namespace(17).ParseName("{path}").InvokeVerb("Eject")"#
    );

    match crate::sys_cmd("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
    {
        Ok(output) if output.status.success() => {
            crate::log::info("Rust::eject", "powershell InvokeVerb Eject OK");
            SystemCmdResult {
                success: true,
                message: "Eject initiated".into(),
            }
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let msg = if err.is_empty() {
                "powershell exited non-zero".into()
            } else {
                err
            };
            crate::log::error("Rust::eject", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
        Err(e) => {
            let msg = format!("failed to spawn powershell: {e}");
            crate::log::error("Rust::eject", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}
