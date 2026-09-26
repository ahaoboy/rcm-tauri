//! `@add-to-quick-access` / `@remove-from-quick-access` — Add or remove
//! a file/folder from the Windows Quick Access pane in File Explorer.
//!
//! Uses the `quick-access` library (direct COM calls via `IShellItem` /
//! `IContextMenu`) instead of PowerShell.

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@add-to-quick-access` — pin a file/folder to Quick Access.
pub struct AddToQuickAccess;

/// Arguments for [`AddToQuickAccess`].
struct Args {
    /// Path to pin.
    path: String,
}

impl AddToQuickAccess {
    /// Extract the path to pin.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            path: CmdArgs::of(payload)
                .required(0, "path", "a file or folder path")?
                .to_owned(),
        })
    }

    /// Pin the path to Quick Access.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info(
            "Rust::add_to_quick_access",
            &format!("adding '{path}' to Quick Access"),
        );

        quick_access::add(path).map_err(|e| {
            let message = e.to_string();
            crate::log::error("Rust::add_to_quick_access", &message);
            CmdError::failed(message)
        })?;

        crate::log::info("Rust::add_to_quick_access", "pintohome OK");
        Ok(format!("Added to Quick Access: {path}"))
    }
}

impl Command for AddToQuickAccess {
    fn id(&self) -> &'static str {
        "@add-to-quick-access"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// `@remove-from-quick-access` — unpin a file/folder from Quick Access.
pub struct RemoveFromQuickAccess;

/// Arguments for [`RemoveFromQuickAccess`].
struct RemoveArgs {
    /// Path to unpin.
    path: String,
}

impl RemoveFromQuickAccess {
    /// Extract the path to unpin.
    fn args(payload: &CommandPayload) -> Result<RemoveArgs, CmdError> {
        Ok(RemoveArgs {
            path: CmdArgs::of(payload)
                .required(0, "path", "a file or folder path")?
                .to_owned(),
        })
    }

    /// Unpin the path from Quick Access.
    fn execute(args: RemoveArgs) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info(
            "Rust::remove_from_quick_access",
            &format!("removing '{path}' from Quick Access"),
        );

        quick_access::remove(path).map_err(|e| {
            let message = e.to_string();
            crate::log::error("Rust::remove_from_quick_access", &message);
            CmdError::failed(message)
        })?;

        crate::log::info("Rust::remove_from_quick_access", "unpinfromhome OK");
        Ok(format!("Removed from Quick Access: {path}"))
    }
}

impl Command for RemoveFromQuickAccess {
    fn id(&self) -> &'static str {
        "@remove-from-quick-access"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// List all paths currently pinned to Quick Access.
pub fn list_quick_access() -> Vec<String> {
    match quick_access::list() {
        Ok(entries) => entries
            .into_iter()
            .map(|e| e.path.to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    }
}
