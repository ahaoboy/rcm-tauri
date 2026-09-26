//! `@copy-target` — Resolve a .lnk shortcut and copy its target path to
//! the clipboard.
//!
//! Only works for single `.lnk` file selections.

use std::path::Path;

use clipboard_rs::{Clipboard, ClipboardContext};

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@copy-target` — copy the resolved target of a single `.lnk` file.
pub struct CopyTarget;

/// Arguments for [`CopyTarget`].
struct Args {
    /// The `.lnk` shortcut to resolve.
    path: String,
}

impl CopyTarget {
    /// Extract the shortcut path.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            path: CmdArgs::of(payload)
                .required(0, "path", "a .lnk shortcut path")?
                .to_owned(),
        })
    }

    /// Resolve the shortcut and copy its target.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        let link = lnk_com::resolve(Path::new(path))
            .map_err(|e| CmdError::failed(format!("Failed to resolve '{path}': {e}")))?;
        let target = link
            .link_target()
            .ok_or_else(|| CmdError::failed(format!("'{path}' has no target")))?;

        crate::log::info("Rust::copy_target", &format!("'{path}' → '{target}'"));

        // Convert backslashes to forward slashes (consistent with @copy-path).
        let target = target.replace('\\', "/");

        let ctx = ClipboardContext::new()
            .map_err(|e| CmdError::failed(format!("Failed to open clipboard: {e}")))?;
        ctx.set_text(target.clone())
            .map_err(|e| CmdError::failed(format!("Failed to set clipboard: {e}")))?;
        Ok(format!("Copied target: {target}"))
    }
}

impl Command for CopyTarget {
    fn id(&self) -> &'static str {
        "@copy-target"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
