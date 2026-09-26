//! `@pin-to-start` / `@unpin-from-start` — Pin or unpin a file to the
//! Windows Start Menu by creating/removing a shortcut in
//! `%APPDATA%\Microsoft\Windows\Start Menu\Programs`.
//!
//! Uses the [`startmenu`] crate for file-level API (no external shell).

use std::path::Path;

use startmenu::{self, Scope};

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@pin-to-start` — create a Start Menu shortcut for a file.
pub struct PinToStart;

/// Arguments for [`PinToStart`].
struct Args {
    /// File to pin.
    path: String,
    /// Shortcut name (the file stem).
    name: String,
}

impl PinToStart {
    /// Extract the path and derive the shortcut name.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let path = CmdArgs::of(payload).required(0, "path", "a file path")?;
        let name = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| CmdError::invalid("path", path, "a path with a file name"))?;
        Ok(Args {
            path: path.to_owned(),
            name: name.to_owned(),
        })
    }

    /// Create the Start Menu shortcut.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info(
            "Rust::pin_to_start",
            &format!("pinning '{path}' as '{}'", args.name),
        );

        startmenu::add(Scope::User, &args.name, Path::new(path), None)
            .map(|lnk| {
                crate::log::info("Rust::pin_to_start", "shortcut created OK");
                format!("Pinned to Start: {}", lnk.display())
            })
            .map_err(|e| {
                crate::log::error("Rust::pin_to_start", &e.to_string());
                CmdError::failed(e.to_string())
            })
    }
}

impl Command for PinToStart {
    fn id(&self) -> &'static str {
        "@pin-to-start"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// `@unpin-from-start` — remove a Start Menu shortcut.
pub struct UnpinFromStart;

/// Arguments for [`UnpinFromStart`].
struct UnpinArgs {
    /// File whose Start Menu shortcut should be removed.
    path: String,
}

impl UnpinFromStart {
    /// Extract the file path.
    fn args(payload: &CommandPayload) -> Result<UnpinArgs, CmdError> {
        Ok(UnpinArgs {
            path: CmdArgs::of(payload)
                .required(0, "path", "a file path")?
                .to_owned(),
        })
    }

    /// Remove the matching Start Menu shortcut.
    fn execute(args: UnpinArgs) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info("Rust::unpin_from_start", &format!("unpinning '{path}'"));

        match startmenu::remove(Path::new(path)) {
            Ok(removed) if removed.is_empty() => {
                crate::log::info("Rust::unpin_from_start", "no matching shortcut found");
                Ok("Already not pinned".into())
            }
            Ok(removed) => {
                crate::log::info("Rust::unpin_from_start", "shortcut removed OK");
                let list = removed
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(format!("Unpinned from Start: {list}"))
            }
            Err(e) => {
                crate::log::error("Rust::unpin_from_start", &e.to_string());
                Err(CmdError::failed(e.to_string()))
            }
        }
    }
}

impl Command for UnpinFromStart {
    fn id(&self) -> &'static str {
        "@unpin-from-start"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// List all items pinned to the Start Menu as [`crate::types::Entry`]s
/// (user + machine scopes, with resolved args and target).
pub fn list_pinned_to_start() -> Vec<crate::types::Entry> {
    startmenu::list()
        .unwrap_or_default()
        .into_iter()
        .map(|lnk| crate::types::Entry {
            path: lnk.path.to_string_lossy().into_owned(),
            args: lnk.args.clone(),
            target: lnk.target().ok().flatten(),
        })
        .collect()
}
