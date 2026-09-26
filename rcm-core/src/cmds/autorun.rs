//! `@add-to-autorun` / `@remove-from-autorun` — Add or remove an .exe
//! file from the Windows startup (autorun) list.
//!
//! Uses the `autorun` crate to manipulate the registry Run keys
//! (`HKCU\...\Run` and `HKLM\...\Run`). Only `User` scope is
//! used for add/remove (no elevation required).
//!
//! The frontend sends the file stem as `name` (arg[0]) and the
//! full path as `command` (arg[1]) for add; the full path alone for remove
//! (backend resolves name by matching the command).

use autorun::Entry;

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@add-to-autorun` — add a program to Windows startup.
pub struct AddToAutorun;

/// Arguments for [`AddToAutorun`].
struct Args {
    /// Startup entry name.
    name: String,
    /// Executable path to launch at startup.
    command: String,
}

impl AddToAutorun {
    /// Extract the entry name and command.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let args = CmdArgs::of(payload);
        Ok(Args {
            name: args.required(0, "name", "the startup entry name")?.to_owned(),
            command: args.required(1, "command", "the executable path")?.to_owned(),
        })
    }

    /// Register the program in the startup list.
    fn execute(args: Args) -> Result<String, CmdError> {
        crate::log::info(
            "Rust::add_to_autorun",
            &format!("adding '{}' → '{}' to startup", args.name, args.command),
        );

        autorun::add(&args.name, &args.command, autorun::Scope::User).map_err(|e| {
            let message = e.to_string();
            crate::log::error("Rust::add_to_autorun", &message);
            CmdError::failed(message)
        })?;

        crate::log::info("Rust::add_to_autorun", "add OK");
        Ok(format!("Added to startup: {}", args.name))
    }
}

impl Command for AddToAutorun {
    fn id(&self) -> &'static str {
        "@add-to-autorun"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// `@remove-from-autorun` — remove a program from Windows startup.
pub struct RemoveFromAutorun;

/// Arguments for [`RemoveFromAutorun`].
struct RemoveArgs {
    /// Executable path to unregister.
    path: String,
}

impl RemoveFromAutorun {
    /// Extract the executable path.
    fn args(payload: &CommandPayload) -> Result<RemoveArgs, CmdError> {
        Ok(RemoveArgs {
            path: CmdArgs::of(payload)
                .required(0, "path", "the executable path")?
                .to_owned(),
        })
    }

    /// Remove the matching startup entry.
    fn execute(args: RemoveArgs) -> Result<String, CmdError> {
        let path = args.path.as_str();
        // Resolve the entry by matching the stored command path.
        let entry = find_entry_name_by_command(path)
            .ok_or_else(|| CmdError::failed(format!("Not found in startup: {path}")))?;

        crate::log::info(
            "Rust::remove_from_autorun",
            &format!("removing '{}' ({path}) from startup", entry.name),
        );

        entry.remove().map_err(|e| {
            let message = e.to_string();
            crate::log::error("Rust::remove_from_autorun", &message);
            CmdError::failed(message)
        })?;

        crate::log::info("Rust::remove_from_autorun", "remove OK");
        Ok(format!("Removed from startup: {path}"))
    }
}

impl Command for RemoveFromAutorun {
    fn id(&self) -> &'static str {
        "@remove-from-autorun"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// List all autorun entries from both HKCU and HKLM scopes.
///
/// The frontend uses this list to decide whether a given .exe is
/// already in the startup list (matched by `command` = full path).
pub fn list_autorun_entries() -> Vec<Entry> {
    autorun::list().unwrap_or_else(|e| {
        crate::log::error("Rust::list_autorun", &e.to_string());
        Vec::new()
    })
}

/// Extract the bare .exe path from a registry command string.
/// Handles quoted paths, trailing NUL bytes, and extra arguments.
fn exe_path(command: &str) -> &str {
    let cmd = command.trim_end_matches('\0');
    if let Some(stripped) = cmd.strip_prefix('"') {
        // Quoted path: take everything until the closing quote
        stripped
            .find('"')
            .map(|i| &stripped[..i])
            .unwrap_or(stripped)
    } else {
        // Unquoted: take the first space-delimited token
        cmd.split_whitespace().next().unwrap_or(cmd)
    }
}

/// Find the autorun entry name that contains the given command path.
fn find_entry_name_by_command(target: &str) -> Option<Entry> {
    autorun::list()
        .ok()?
        .into_iter()
        .find(|e| exe_path(&e.command).eq_ignore_ascii_case(target))
}
