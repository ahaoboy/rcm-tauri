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

use super::SystemCmdResult;
use crate::types::CommandPayload;

/// Run `@add-to-autorun` — add a program to Windows startup.
///
/// Expects `args[0]` = entry name (file stem), `args[1]` = full path.
pub fn run_add(cmd: &CommandPayload) -> SystemCmdResult {
    let (name, command) = match (cmd.args.first(), cmd.args.get(1)) {
        (Some(n), Some(c)) if !n.is_empty() && !c.is_empty() => (n.as_str(), c.as_str()),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "Requires name and command arguments".into(),
            };
        }
    };

    crate::log::info(
        "Rust::add_to_autorun",
        &format!("adding '{name}' → '{command}' to startup"),
    );

    match autorun::add(name, command, autorun::Scope::User) {
        Ok(()) => {
            crate::log::info("Rust::add_to_autorun", "add OK");
            SystemCmdResult {
                success: true,
                message: format!("Added to startup: {name}"),
            }
        }
        Err(e) => {
            let msg = e.to_string();
            crate::log::error("Rust::add_to_autorun", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}

/// Run `@remove-from-autorun` — remove a program from Windows startup.
///
/// Expects `args[0]` = full path of the .exe (matched against stored commands).
pub fn run_remove(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No path specified".into(),
            };
        }
    };

    // Find the entry by matching the command path.
    let entry = match find_entry_name_by_command(path) {
        Some(e) => e,
        None => {
            return SystemCmdResult {
                success: false,
                message: format!("Not found in startup: {path}"),
            };
        }
    };

    crate::log::info(
        "Rust::remove_from_autorun",
        &format!("removing '{}' ({path}) from startup", entry.name),
    );

    match entry.remove() {
        Ok(()) => {
            crate::log::info("Rust::remove_from_autorun", "remove OK");
            SystemCmdResult {
                success: true,
                message: format!("Removed from startup: {path}"),
            }
        }
        Err(e) => {
            let msg = e.to_string();
            crate::log::error("Rust::remove_from_autorun", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
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
