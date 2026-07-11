//! `@format` — Open the Windows "Format" dialog for a drive.
//!
//! Uses PowerShell P/Invoke to call [`SHFormatDrive`](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shformatdrive)
//! from `shell32.dll` — the exact same API that Explorer invokes when you
//! click "Format…" in the drive context menu.
//!
//! # Parameters
//!
//! | Parameter | Value    | Meaning                      |
//! |-----------|----------|------------------------------|
//! | hwnd      | 0        | no parent window             |
//! | drive     | 0=A,2=C… | drive index (A: = 0)        |
//! | fmtID     | 0xFFFF   | SHFMT_ID_DEFAULT — all opts  |
//! | options   | 0        | SHFMT_OPT_DEFAULT            |

use super::SystemCmdResult;
use crate::types::CommandPayload;

/// Convert a drive path like `"C:\\"` or `"C:"` to `SHFormatDrive` drive index.
/// A: = 0, B: = 1, C: = 2, …
fn drive_index(path: &str) -> Option<u32> {
    let letter = path.trim_start().chars().next()?;
    if !letter.is_ascii_alphabetic() {
        return None;
    }
    Some(letter.to_ascii_uppercase() as u32 - 'A' as u32)
}

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

    let drive = match drive_index(path) {
        Some(d) => d,
        None => {
            return SystemCmdResult {
                success: false,
                message: format!("Could not parse drive letter from '{path}'"),
            };
        }
    };

    crate::log::info(
        "Rust::format",
        &format!("opening format dialog for '{path}' (drive index {drive})"),
    );

    // P/Invoke SHFormatDrive via powershell.exe Add-Type.
    // fmtID = 0xFFFF (SHFMT_ID_DEFAULT)  → show all formatting options.
    // options = 0 (SHFMT_OPT_DEFAULT)    → default behaviour.
    let script = format!(
        r#"$code='[DllImport("shell32.dll")]public static extern uint SHFormatDrive(IntPtr hwnd,uint drive,uint fmtID,uint options);';$t=Add-Type -MemberDefinition $code -Name 'Fmt' -Namespace 'Win32' -PassThru;$t::SHFormatDrive([IntPtr]::Zero,{drive},0xFFFF,0)|Out-Null"#
    );

    match crate::sys_cmd("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
    {
        Ok(output) if output.status.success() => {
            crate::log::info("Rust::format", "powershell SHFormatDrive OK");
            SystemCmdResult {
                success: true,
                message: "Format dialog opened".into(),
            }
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let msg = if err.is_empty() {
                "powershell exited non-zero".into()
            } else {
                err
            };
            crate::log::error("Rust::format", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
        Err(e) => {
            let msg = format!("failed to spawn powershell: {e}");
            crate::log::error("Rust::format", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}
