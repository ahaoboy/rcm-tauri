//! `@eject` — Eject a removable drive.
//!
//! Uses PowerShell + Shell.Application COM to invoke the "Eject" shell verb,
//! exactly replicating the original right-click → Eject behavior.

use super::{CmdArgs, CmdError, Command, powershell_error};
use crate::types::CommandPayload;

/// `@eject` — eject the removable drive at `path`.
pub struct Eject;

/// Arguments for [`Eject`].
struct Args {
    /// Drive to eject.
    path: String,
}

impl Eject {
    /// Extract the drive path.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            path: CmdArgs::of(payload)
                .required(0, "drive", "a drive path (e.g. E:\\)")?
                .to_owned(),
        })
    }

    /// Invoke the "Eject" verb on the drive.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info("Rust::eject", &format!("ejecting drive '{path}'"));

        // Shell.Application → Namespace(17) = "This PC" → ParseName finds the
        // drive → InvokeVerb("Eject") fires the same handler as the Win11
        // right-click menu. Single quotes are used so any double quotes or `$`
        // in the path cannot break out of the PowerShell literal.
        let escaped = path.replace('\'', "''");
        let script = format!(
            r#"(New-Object -ComObject Shell.Application).Namespace(17).ParseName('{escaped}').InvokeVerb("Eject")"#
        );

        match crate::sys_cmd("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
        {
            Ok(output) if output.status.success() => {
                crate::log::info("Rust::eject", "powershell InvokeVerb Eject OK");
                Ok("Eject initiated".into())
            }
            Ok(output) => {
                let message = powershell_error(&output.stderr, "powershell exited non-zero");
                crate::log::error("Rust::eject", &message);
                Err(CmdError::failed(message))
            }
            Err(e) => {
                let message = format!("failed to spawn powershell: {e}");
                crate::log::error("Rust::eject", &message);
                Err(CmdError::failed(message))
            }
        }
    }
}

impl Command for Eject {
    fn id(&self) -> &'static str {
        "@eject"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
