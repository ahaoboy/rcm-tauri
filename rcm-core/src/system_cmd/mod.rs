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

pub mod copy;
pub mod copy_path;
pub mod delete;
pub mod new_file;
pub mod new_folder;
pub mod open_file_location;
pub mod open_with;
pub mod properties;
pub mod rename;
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
    /// Copy selected paths to clipboard (`@copy-path`).
    CopyPath,
    /// Fast parallel permanent delete (`@delete`).
    Delete,
    /// Open file/folder properties dialog (`@properties`).
    Properties,
    /// Copy file(s) to clipboard as file-drop data (`@copy`).
    Copy,
    /// Open file location in Explorer; resolves shortcut targets (`@open-file-location`).
    OpenFileLocation,
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
            "@delete" => Ok(Self::Delete),
            "@properties" => Ok(Self::Properties),
            "@copy" => Ok(Self::Copy),
            "@open-file-location" => Ok(Self::OpenFileLocation),
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
            Self::Delete => delete::run(cmd),
            Self::Properties => properties::run(cmd),
            Self::Copy => copy::run(cmd),
            Self::OpenFileLocation => open_file_location::run(cmd),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared utilities
// ═══════════════════════════════════════════════════════════════════════════

/// Run a PowerShell scriptlet and return its stdout on success.
#[allow(dead_code)]
pub(crate) fn powershell(script: &str) -> Result<String, String> {
    use std::os::windows::process::CommandExt;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| format!("failed to spawn powershell: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if err.is_empty() { "powershell returned non-zero".into() } else { err })
    }
}

/// Build a `std::process::Command` from the payload, suitable for
/// synchronous execution (these are quick operations).
#[allow(dead_code)]
pub(crate) fn build_sys_cmd(exe: &str, cmd: &CommandPayload) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut c = std::process::Command::new(exe);
    if !cmd.args.is_empty() {
        c.args(&cmd.args);
    }
    if !cmd.cwd.is_empty() {
        c.current_dir(&cmd.cwd);
    }
    c.creation_flags(CREATE_NO_WINDOW);
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
        assert!(!is_system_command("notepad"));
        assert!(!is_system_command(""));
    }

    #[test]
    fn test_parse_system_command() {
        assert_eq!("@unzip".parse::<SystemCommand>().unwrap(), SystemCommand::Unzip);
        assert_eq!("@zip".parse::<SystemCommand>().unwrap(), SystemCommand::Zip);
        assert!("@unknown".parse::<SystemCommand>().is_err());
    }
}
