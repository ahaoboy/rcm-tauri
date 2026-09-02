//! System command routing for `@xxx` prefixed command identifiers.
//!
//! When the frontend sends a `CommandPayload` whose `exe` starts with `@`,
//! it is parsed into a [`SystemCommand`] variant via [`FromStr`] and executed
//! natively rather than being spawned as an external process.
//!
//! Each command lives in its own file under this directory, matching the
//! constants defined in `rcm/src/system-commands.ts`.
//!
//! # Adding a new system command
//!
//! 1. Create a new file (e.g. `my_cmd.rs`) with a `pub fn run(cmd: &CommandPayload) -> SystemCmdResult`.
//! 2. Declare it here with `pub mod my_cmd;`.
//! 3. Add the variant to `SystemCommand`, the `@xxx` mapping in `FromStr`, and the arm in `SystemCommand::run`.
//! 4. Add the constant in `rcm/src/system-commands.ts`.

use crate::types::CommandPayload;
use std::str::FromStr;

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

// ═══════════════════════════════════════════════════════════════════════════
// SystemCommand enum
// ═══════════════════════════════════════════════════════════════════════════

/// Built-in system commands identified by `@`-prefixed strings.
///
/// Each variant corresponds to a constant exported from
/// `rcm/src/system-commands.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemCommand {
    /// Extract a zip archive (`@unzip`).
    Unzip,
    /// Create a zip archive (`@zip`).
    Zip,
    /// Rename a file or folder with collision avoidance (`@rename`).
    Rename,
    /// Create a new empty file (`@new-file`).
    NewFile,
    /// Create a new folder (`@new-folder`).
    NewFolder,
    /// Move to recycle bin (`@trash`).
    Trash,
    /// Open the "Open With" dialog (`@open-with`).
    OpenWith,
    /// Copy selected paths to clipboard with Linux-style separators (`@copy-path`).
    CopyPath,
    /// Copy file name(s) to clipboard (`@copy-name`).
    CopyName,
    /// Copy file content(s) as base64 to clipboard (`@copy-base64`).
    CopyBase64,
    /// Resolve .lnk target path and copy to clipboard (`@copy-target`).
    CopyTarget,
    /// Fast parallel permanent delete (`@delete`).
    Delete,
    /// Open file/folder properties dialog (`@properties`).
    Properties,
    /// Copy file(s) to clipboard as file-drop data (`@copy`).
    Copy,
    /// Open file location in Explorer; resolves shortcut targets (`@open-file-location`).
    OpenFileLocation,
    /// Paste files from clipboard to current directory (`@paste-files`).
    PasteFiles,
    /// Change Explorer grouping for the current directory (`@group-by`).
    GroupBy,
    /// Change Explorer sorting for the current directory (`@sort-by`).
    SortBy,
    /// Open Windows "Format" dialog for a drive (`@format`).
    Format,
    /// Eject a removable drive (`@eject`).
    Eject,
    /// Pin a file to the Start Menu (`@pin-to-start`).
    PinToStart,
    /// Unpin a file from the Start Menu (`@unpin-from-start`).
    UnpinFromStart,
    /// Add a file/folder to Quick Access (`@add-to-quick-access`).
    AddToQuickAccess,
    /// Remove a file/folder from Quick Access (`@remove-from-quick-access`).
    RemoveFromQuickAccess,
    /// Add an .exe to Windows startup (`@add-to-autorun`).
    AddToAutorun,
    /// Remove an .exe from Windows startup (`@remove-from-autorun`).
    RemoveFromAutorun,
    /// Add a desktop shortcut for the file (`@add-to-desktop`).
    AddToDesktop,
    /// Remove the desktop shortcut(s) pointing to the file (`@remove-from-desktop`).
    RemoveFromDesktop,
}

// ═══════════════════════════════════════════════════════════════════════════
// FromStr — parse "@unzip" → SystemCommand::Unzip, etc.
// ═══════════════════════════════════════════════════════════════════════════

impl FromStr for SystemCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "@unzip" => Ok(Self::Unzip),
            "@zip" => Ok(Self::Zip),
            "@rename" => Ok(Self::Rename),
            "@new-file" => Ok(Self::NewFile),
            "@new-folder" => Ok(Self::NewFolder),
            "@trash" => Ok(Self::Trash),
            "@open-with" => Ok(Self::OpenWith),
            "@copy-path" => Ok(Self::CopyPath),
            "@copy-name" => Ok(Self::CopyName),
            "@copy-base64" => Ok(Self::CopyBase64),
            "@copy-target" => Ok(Self::CopyTarget),
            "@delete" => Ok(Self::Delete),
            "@properties" => Ok(Self::Properties),
            "@copy" => Ok(Self::Copy),
            "@open-file-location" => Ok(Self::OpenFileLocation),
            "@paste-files" => Ok(Self::PasteFiles),
            "@group-by" => Ok(Self::GroupBy),
            "@sort-by" => Ok(Self::SortBy),
            "@format" => Ok(Self::Format),
            "@eject" => Ok(Self::Eject),
            "@pin-to-start" => Ok(Self::PinToStart),
            "@unpin-from-start" => Ok(Self::UnpinFromStart),
            "@add-to-quick-access" => Ok(Self::AddToQuickAccess),
            "@remove-from-quick-access" => Ok(Self::RemoveFromQuickAccess),
            "@add-to-autorun" => Ok(Self::AddToAutorun),
            "@remove-from-autorun" => Ok(Self::RemoveFromAutorun),
            "@add-to-desktop" => Ok(Self::AddToDesktop),
            "@remove-from-desktop" => Ok(Self::RemoveFromDesktop),
            _ => Err(format!("unknown system command: {s}")),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SystemCmdResult
// ═══════════════════════════════════════════════════════════════════════════

/// Result returned by [`SystemCommand::run`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemCmdResult {
    pub success: bool,
    pub message: String,
}

impl SystemCommand {
    /// Execute this system command with the given payload.
    ///
    /// Returns a [`SystemCmdResult`] describing success / failure.
    pub fn run(&self, cmd: &CommandPayload) -> SystemCmdResult {
        match self {
            Self::Unzip => unzip::run(cmd),
            Self::Zip => zip::run(cmd),
            Self::Rename => rename::run(cmd),
            Self::NewFile => new_file::run(cmd),
            Self::NewFolder => new_folder::run(cmd),
            Self::Trash => trash::run(cmd),
            Self::OpenWith => open_with::run(cmd),
            Self::CopyPath => copy_path::run(cmd),
            Self::CopyName => copy_name::run(cmd),
            Self::CopyBase64 => copy_base64::run(cmd),
            Self::CopyTarget => copy_target::run(cmd),
            Self::Delete => delete::run(cmd),
            Self::Properties => properties::run(cmd),
            Self::Copy => copy::run(cmd),
            Self::OpenFileLocation => open_file_location::run(cmd),
            Self::PasteFiles => paste_files::run(cmd),
            Self::GroupBy => group_by::run(cmd),
            Self::SortBy => sort_by::run(cmd),
            Self::Format => format::run(cmd),
            Self::Eject => eject::run(cmd),
            Self::PinToStart => pin_to_start::run_pin(cmd),
            Self::UnpinFromStart => pin_to_start::run_unpin(cmd),
            Self::AddToQuickAccess => quick_access::run_add(cmd),
            Self::RemoveFromQuickAccess => quick_access::run_remove(cmd),
            Self::AddToAutorun => autorun::run_add(cmd),
            Self::RemoveFromAutorun => autorun::run_remove(cmd),
            Self::AddToDesktop => desktop::add(cmd),
            Self::RemoveFromDesktop => desktop::remove(cmd),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared utilities
// ═══════════════════════════════════════════════════════════════════════════

/// Run a PowerShell scriptlet and return its stdout on success.
#[allow(dead_code)]
pub(crate) fn powershell(script: &str) -> Result<String, String> {
    let output = crate::sys_cmd("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("failed to spawn powershell: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if err.is_empty() {
            "powershell returned non-zero".into()
        } else {
            err
        })
    }
}

/// Build a `std::process::Command` from the payload, suitable for
/// synchronous execution (these are quick operations).
#[allow(dead_code)]
pub(crate) fn build_sys_cmd(exe: &str, cmd: &CommandPayload) -> std::process::Command {
    let mut c = crate::sys_cmd(exe);
    if !cmd.args.is_empty() {
        c.args(&cmd.args);
    }
    if !cmd.cwd.is_empty() {
        c.current_dir(&cmd.cwd);
    }
    c
}

/// Return a unique path by appending ` (2)`, ` (3)`, … if the target
/// already exists.
pub(crate) fn unique_path(path: &std::path::Path) -> std::path::PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let parent = path.parent().unwrap_or(std::path::Path::new("."));

    for n in 2..u32::MAX {
        let name = if ext.is_empty() {
            format!("{stem} ({n})")
        } else {
            format!("{stem} ({n}).{ext}")
        };
        let candidate = parent.join(&name);
        if !candidate.exists() {
            return candidate;
        }
    }

    path.to_path_buf()
}

// ═══════════════════════════════════════════════════════════════════════════
// Public helper
// ═══════════════════════════════════════════════════════════════════════════

/// Check whether a command string is a system command (`@` prefix).
pub fn is_system_command(exe: &str) -> bool {
    exe.starts_with('@')
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
    fn test_parse_system_command() {
        assert_eq!(
            "@unzip".parse::<SystemCommand>().unwrap(),
            SystemCommand::Unzip
        );
        assert_eq!("@zip".parse::<SystemCommand>().unwrap(), SystemCommand::Zip);
        assert_eq!(
            "@group-by".parse::<SystemCommand>().unwrap(),
            SystemCommand::GroupBy
        );
        assert_eq!(
            "@sort-by".parse::<SystemCommand>().unwrap(),
            SystemCommand::SortBy
        );
        assert_eq!(
            "@format".parse::<SystemCommand>().unwrap(),
            SystemCommand::Format
        );
        assert_eq!(
            "@eject".parse::<SystemCommand>().unwrap(),
            SystemCommand::Eject
        );
        assert_eq!(
            "@pin-to-start".parse::<SystemCommand>().unwrap(),
            SystemCommand::PinToStart
        );
        assert_eq!(
            "@unpin-from-start".parse::<SystemCommand>().unwrap(),
            SystemCommand::UnpinFromStart
        );
        assert_eq!(
            "@add-to-quick-access".parse::<SystemCommand>().unwrap(),
            SystemCommand::AddToQuickAccess
        );
        assert_eq!(
            "@remove-from-quick-access".parse::<SystemCommand>().unwrap(),
            SystemCommand::RemoveFromQuickAccess
        );
        assert!("@unknown".parse::<SystemCommand>().is_err());
    }
}
