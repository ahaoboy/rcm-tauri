//! `@add-to-desktop` / `@remove-from-desktop` — desktop shortcut add/remove,
//! mimicking Explorer's *"Send to > Desktop (create shortcut)"*.
//!
//! Uses [`desktop_com`]: `add` creates the `.lnk` via Shell COM (duplicate
//! names auto-renamed `a.lnk` → `a(1).lnk` by upath); `remove` deletes every
//! desktop shortcut pointing at the target (user + machine scope).

use std::path::Path;

use desktop_com::Scope;

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

const TAG: &str = "desktop";

/// `@add-to-desktop` — create a desktop shortcut for a file.
pub struct AddToDesktop;

/// Arguments for [`AddToDesktop`].
struct Args {
    /// File to create a shortcut to.
    path: String,
    /// Shortcut name (the file stem).
    name: String,
}

impl AddToDesktop {
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

    /// Create the desktop shortcut.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info(TAG, &format!("add '{path}'"));

        desktop_com::add(Scope::User, &args.name, Path::new(path), None)
            .map(|lnk| format!("Desktop shortcut: {}", lnk.display()))
            .map_err(|e| {
                crate::log::error(TAG, &e.to_string());
                CmdError::failed(e.to_string())
            })
    }
}

impl Command for AddToDesktop {
    fn id(&self) -> &'static str {
        "@add-to-desktop"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// `@remove-from-desktop` — remove every desktop shortcut to a file.
pub struct RemoveFromDesktop;

/// Arguments for [`RemoveFromDesktop`].
struct RemoveArgs {
    /// File whose desktop shortcuts should be removed.
    path: String,
}

impl RemoveFromDesktop {
    /// Extract the file path.
    fn args(payload: &CommandPayload) -> Result<RemoveArgs, CmdError> {
        Ok(RemoveArgs {
            path: CmdArgs::of(payload)
                .required(0, "path", "a file path")?
                .to_owned(),
        })
    }

    /// Remove every desktop shortcut to the file.
    fn execute(args: RemoveArgs) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info(TAG, &format!("remove '{path}'"));

        match desktop_com::remove(Path::new(path)) {
            Ok(list) if list.is_empty() => Ok("Not on desktop".into()),
            Ok(list) => Ok(format!("Removed: {}", list.len())),
            Err(e) => {
                crate::log::error(TAG, &e.to_string());
                Err(CmdError::failed(e.to_string()))
            }
        }
    }
}

impl Command for RemoveFromDesktop {
    fn id(&self) -> &'static str {
        "@remove-from-desktop"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// Desktop entries (user + public) as [`crate::types::Entry`]s — for
/// frontend add/remove matching.
pub fn list() -> Vec<crate::types::Entry> {
    match desktop_com::list_shortcuts() {
        Ok(items) => items
            .into_iter()
            .map(|it| crate::types::Entry {
                path: it.path.to_string_lossy().into_owned(),
                args: it.args.clone(),
                target: it.target().ok().flatten(),
            })
            .collect(),
        Err(e) => {
            crate::log::error(TAG, &format!("list failed: {e}"));
            Vec::new()
        }
    }
}
