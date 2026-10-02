//! Native `@xxx` system commands.
//!
//! When the frontend sends a [`CommandPayload`] whose `cmd` starts with `@`,
//! it is dispatched to a native command instead of being spawned as a process.
//!
//! # Architecture
//!
//! Every command is a stateless unit struct implementing [`Command`]. Each one
//! splits its work across three functions:
//!
//! * `args` — extract the structural command-line arguments into a typed struct.
//! * `execute` — perform the work given those arguments.
//! * [`Command::id`] / [`Command::run`] — name the command and glue `args`
//!   into `execute`.
//!
//! [`run`] looks the command up in the [`COMMANDS`] table and calls it. That
//! table is the single place commands are registered: add a unit struct and one
//! line there. This module deliberately knows nothing about transport concerns
//! such as exit codes or stdout/stderr; the caller (see `crate::runner`) maps
//! [`CmdError`] to its own result type.

use crate::types::CommandPayload;
use std::path::PathBuf;

mod args;
pub mod autorun;
pub mod copy;
pub mod copy_base64;
pub mod copy_name;
pub mod copy_path;
pub mod copy_target;
pub mod delete;
pub mod desktop;
pub mod eject;
pub mod format;
pub mod group_by;
pub mod new_file;
pub mod new_folder;
pub mod open_file_location;
pub mod open_with;
pub mod paste_files;
pub mod pin_to_start;
pub mod properties;
pub mod quick_access;
pub mod rename;
pub mod shell_folder_view;
pub mod sort_by;
pub mod trash;
pub mod unzip;
pub mod zip;

pub(crate) use args::CmdArgs;

/// Error raised while parsing or running a system command.
///
/// The variants distinguish *why* the command failed and carry enough detail
/// for a useful message: argument errors name the argument, the offending value
/// and the expected form, so the user can fix the call.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CmdError {
    /// A required argument is absent.
    #[error("missing argument '{name}' (expected {expected})")]
    Missing {
        /// Name of the argument, e.g. `"path"`.
        name: &'static str,
        /// What the argument should have been, e.g. `"a file path"`.
        expected: &'static str,
    },
    /// An argument is present but holds an unsupported value.
    #[error("invalid value '{value}' for '{name}' (expected {expected})")]
    Invalid {
        /// Name of the argument.
        name: &'static str,
        /// The offending value.
        value: String,
        /// What the argument should have been.
        expected: &'static str,
    },
    /// The command was valid but the underlying operation failed.
    #[error("{message}")]
    Failed {
        /// Human-readable failure reason.
        message: String,
    },
}

impl CmdError {
    /// A required argument is absent.
    pub fn missing(name: &'static str, expected: &'static str) -> Self {
        Self::Missing { name, expected }
    }

    /// An argument holds an unsupported value.
    pub fn invalid(name: &'static str, value: impl Into<String>, expected: &'static str) -> Self {
        Self::Invalid {
            name,
            value: value.into(),
            expected,
        }
    }

    /// The command parsed fine but the operation failed.
    pub fn failed(message: impl Into<String>) -> Self {
        Self::Failed {
            message: message.into(),
        }
    }
}

/// A native system command.
///
/// Implementors are stateless unit structs: arguments live in the payload, not
/// in the command, so a single shared instance can serve every call. `Sync` is
/// required so instances can live in the [`COMMANDS`] `static`.
pub trait Command: Sync {
    /// The `@`-prefixed identifier, e.g. `"@delete"`.
    fn id(&self) -> &'static str;

    /// Validate `payload`'s arguments and perform the work.
    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError>;
}

/// Every registered system command.
///
/// Each entry is a shared instance of a stateless command struct. Lookup and
/// dispatch are a plain scan; there is no enum or generated `match` to keep in
/// sync.
static COMMANDS: &[&dyn Command] = &[
    &unzip::Unzip,
    &zip::Zip,
    &rename::Rename,
    &new_file::NewFile,
    &new_folder::NewFolder,
    &trash::Trash,
    &open_with::OpenWith,
    &copy_path::CopyPath,
    &copy_name::CopyName,
    &copy_base64::CopyBase64,
    &copy_target::CopyTarget,
    &delete::Delete,
    &properties::Properties,
    &copy::CopyFiles,
    &open_file_location::OpenFileLocation,
    &paste_files::PasteFiles,
    &group_by::GroupBy,
    &sort_by::SortBy,
    &format::Format,
    &eject::Eject,
    &pin_to_start::PinToStart,
    &pin_to_start::UnpinFromStart,
    &quick_access::AddToQuickAccess,
    &quick_access::RemoveFromQuickAccess,
    &autorun::AddToAutorun,
    &autorun::RemoveFromAutorun,
    &desktop::AddToDesktop,
    &desktop::RemoveFromDesktop,
];

/// The command registered under `id`, if any — lookup without executing.
fn find(id: &str) -> Option<&'static dyn Command> {
    COMMANDS.iter().copied().find(|cmd| cmd.id() == id)
}

/// Run the system command named `id`.
///
/// Returns `None` when `id` is not registered, `Some(Ok(message))` on success
/// and `Some(Err(error))` on failure. Mapping that to exit codes or output
/// streams is the caller's job — see `crate::runner`.
pub fn run(id: &str, payload: &CommandPayload) -> Option<Result<String, CmdError>> {
    find(id).map(|cmd| cmd.run(payload))
}

/// Resolve `cwd` to a directory path, defaulting to `.` when empty.
pub(crate) fn cwd_dir(cwd: &str) -> PathBuf {
    if cwd.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(cwd)
    }
}

/// Return `path`, or a collision-safe variant with ` (2)`, ` (3)`, … appended.
pub(crate) fn unique_path(path: &std::path::Path) -> PathBuf {
    upath::upath(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Check whether a command string is a system command (`@` prefix).
pub fn is_system_command(exe: &str) -> bool {
    exe.starts_with('@')
}

/// Extract a PowerShell error message: trimmed stderr, or `fallback` when empty.
pub(crate) fn powershell_error(stderr: &[u8], fallback: &str) -> String {
    let text = String::from_utf8_lossy(stderr);
    let text = text.trim();
    if text.is_empty() {
        fallback.to_owned()
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_command() {
        assert!(is_system_command("@unzip"));
        assert!(is_system_command("@zip"));
        assert!(is_system_command("@group-by"));
        assert!(is_system_command("@sort-by"));
        assert!(is_system_command("@format"));
        assert!(is_system_command("@eject"));
        assert!(is_system_command("@pin-to-start"));
        assert!(is_system_command("@unpin-from-start"));
        assert!(is_system_command("@add-to-quick-access"));
        assert!(is_system_command("@remove-from-quick-access"));
        assert!(!is_system_command("notepad"));
        assert!(!is_system_command(""));
    }

    #[test]
    fn command_ids_are_unique_and_prefixed() {
        let mut ids: Vec<&str> = COMMANDS.iter().map(|cmd| cmd.id()).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "command ids must be unique");
        assert!(COMMANDS.iter().all(|cmd| cmd.id().starts_with('@')));
    }

    #[test]
    fn known_ids_resolve_but_unknown_do_not() {
        // Lookup only — running would have real side effects (@new-file, …).
        for cmd in COMMANDS {
            assert!(find(cmd.id()).is_some(), "{} should be registered", cmd.id());
        }
        assert!(find("@nope").is_none());
    }

    #[test]
    fn errors_describe_the_expected_argument() {
        assert_eq!(
            CmdError::missing("path", "a file path").to_string(),
            "missing argument 'path' (expected a file path)"
        );
        assert_eq!(
            CmdError::invalid("key", "foo", "name | size").to_string(),
            "invalid value 'foo' for 'key' (expected name | size)"
        );
    }
}
