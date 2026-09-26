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

use super::{CmdArgs, CmdError, Command, powershell_error};
use crate::types::CommandPayload;

/// Expected form of the `drive` argument, reused by both error sites.
const DRIVE_EXPECTED: &str = "a drive letter (e.g. C:)";

/// `@format` — open the format dialog for the drive at `path`.
pub struct Format;

/// Arguments for [`Format`].
struct Args {
    /// Original drive path, for logging.
    path: String,
    /// Zero-based drive index for `SHFormatDrive`.
    drive: u32,
}

impl Format {
    /// Extract the drive path and index.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let path = CmdArgs::of(payload).required(0, "drive", DRIVE_EXPECTED)?;
        let drive = drive_index(path)
            .ok_or_else(|| CmdError::invalid("drive", path, DRIVE_EXPECTED))?;
        Ok(Args {
            path: path.to_owned(),
            drive,
        })
    }

    /// Invoke `SHFormatDrive` through PowerShell.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        let drive = args.drive;
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
                Ok("Format dialog opened".into())
            }
            Ok(output) => {
                let message = powershell_error(&output.stderr, "powershell exited non-zero");
                crate::log::error("Rust::format", &message);
                Err(CmdError::failed(message))
            }
            Err(e) => {
                let message = format!("failed to spawn powershell: {e}");
                crate::log::error("Rust::format", &message);
                Err(CmdError::failed(message))
            }
        }
    }
}

impl Command for Format {
    fn id(&self) -> &'static str {
        "@format"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// Convert a drive path like `"C:\\"` or `"C:"` to its `SHFormatDrive` index:
/// `A:` = 0, `B:` = 1, `C:` = 2, …
fn drive_index(path: &str) -> Option<u32> {
    let letter = path.trim_start().chars().next()?;
    letter
        .is_ascii_alphabetic()
        .then(|| letter.to_ascii_uppercase() as u32 - 'A' as u32)
}
