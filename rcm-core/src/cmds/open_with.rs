//! `@open-with` — Open the Windows "Open With → Choose another app" dialog.
//!
//! Spawns PowerShell in the background to invoke the `openas` shell verb via
//! `Shell.Application` COM — the exact equivalent of right-click → "Open with"
//! → "Choose another app" in Windows Explorer.
//!
//! Uses `spawn` (fire-and-forget) rather than `output` to avoid blocking the
//! Rust async runtime and to prevent the dialog activation from being
//! misinterpreted as a new right-click event.

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@open-with` — show the "Choose another app" dialog for a file.
pub struct OpenWith;

/// Arguments for [`OpenWith`].
struct Args {
    /// File to open the dialog for.
    path: String,
}

impl OpenWith {
    /// Extract the file path.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            path: CmdArgs::of(payload)
                .required(0, "path", "a file path")?
                .to_owned(),
        })
    }

    /// Launch the "openas" verb through PowerShell.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        // Escape single quotes for PowerShell string interpolation.
        let escaped = path.replace('\'', "''");
        let script = format!(
            "$f=gi -LiteralPath '{escaped}';\
             (New-Object -ComObject Shell.Application).Namespace($f.DirectoryName).ParseName($f.Name).InvokeVerb('openas')"
        );

        crate::sys_cmd("powershell")
            .args(["-NoProfile", "-Command", &script])
            .spawn()
            .map_err(|e| CmdError::failed(format!("OpenWith failed to launch: {e}")))?;

        Ok(format!("OpenWith dialog launched for: {path}"))
    }
}

impl Command for OpenWith {
    fn id(&self) -> &'static str {
        "@open-with"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
