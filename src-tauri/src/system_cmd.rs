//! System command routing for `@xxx` prefixed command identifiers.
//!
//! When the frontend sends a `CommandPayload` whose `exe` starts with `@`,
//! it is parsed into a [`SystemCommand`] variant via [`FromStr`] and executed
//! natively rather than being spawned as an external process.
//!
//! # Adding a new system command
//!
//! 1. Add the variant to the [`SystemCommand`] enum.
//! 2. Add the `@xxx` → variant mapping in [`FromStr`].
//! 3. Implement the `run` arm in [`SystemCommand::run`].
//! 4. Add the constant in `rcm/src/system-commands.ts`.

use crate::rcm::CommandPayload;
use std::os::windows::process::CommandExt;
use std::str::FromStr;

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
            _ => Err(format!("unknown system command: {s}")),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Execution
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
            Self::Unzip => run_unzip(cmd),
            Self::Zip => run_zip(cmd),
            Self::Rename => run_rename(cmd),
            Self::NewFile => run_new_file(cmd),
            Self::NewFolder => run_new_folder(cmd),
            Self::Trash => run_trash(cmd),
            Self::OpenWith => run_open_with(cmd),
            Self::CopyPath => run_copy_path(cmd),
        }
    }
}

// ── Implementation helpers ───────────────────────────────────────────────

/// Build a `std::process::Command` from the payload, suitable for
/// synchronous execution (these are quick operations).
fn build_sys_cmd(exe: &str, cmd: &CommandPayload) -> std::process::Command {
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

/// Run a PowerShell scriptlet and return its stdout on success.
fn powershell(script: &str) -> Result<String, String> {
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

// ── Per-command implementations ───────────────────────────────────────────

fn run_unzip(cmd: &CommandPayload) -> SystemCmdResult {
    let archive = cmd.args.first().cloned().unwrap_or_default();
    let dest = cmd.args.get(1).cloned().unwrap_or_else(|| ".".into());

    let script = format!(
        "Expand-Archive -Path '{archive}' -DestinationPath '{dest}' -Force"
    );
    match powershell(&script) {
        Ok(_) => SystemCmdResult { success: true, message: format!("Extracted: {archive}") },
        Err(e) => SystemCmdResult { success: false, message: e },
    }
}

fn run_zip(cmd: &CommandPayload) -> SystemCmdResult {
    let archive = cmd.args.first().cloned().unwrap_or_default();
    // Remaining args are the source paths
    let sources: Vec<&str> = cmd.args.iter().skip(1).map(|s| s.as_str()).collect();
    let sources_str = sources
        .iter()
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(", ");

    let script = format!(
        "Compress-Archive -Path {sources_str} -DestinationPath '{archive}' -Force"
    );
    match powershell(&script) {
        Ok(_) => SystemCmdResult { success: true, message: format!("Created: {archive}") },
        Err(e) => SystemCmdResult { success: false, message: e },
    }
}

fn run_rename(cmd: &CommandPayload) -> SystemCmdResult {
    // args[0] = source path, args[1] = new name
    let src = cmd.args.first().cloned().unwrap_or_default();
    let new_name = cmd.args.get(1).cloned().unwrap_or_default();

    let src_path = std::path::Path::new(&src);
    let parent = src_path.parent().unwrap_or(std::path::Path::new("."));
    let dest = parent.join(&new_name);

    // Collision avoidance: if dest exists, append (2), (3), …
    let dest = unique_path(&dest);

    match std::fs::rename(&src, &dest) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Renamed to: {}", dest.display()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Rename failed: {e}"),
        },
    }
}

fn run_new_file(cmd: &CommandPayload) -> SystemCmdResult {
    let path = cmd.args.first().cloned().unwrap_or_default();
    let path = unique_path(std::path::Path::new(&path));

    match std::fs::File::create(&path) {
        Ok(_) => SystemCmdResult {
            success: true,
            message: format!("Created: {}", path.display()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Create file failed: {e}"),
        },
    }
}

fn run_new_folder(cmd: &CommandPayload) -> SystemCmdResult {
    let path = cmd.args.first().cloned().unwrap_or_default();
    let path = unique_path(std::path::Path::new(&path));

    match std::fs::create_dir(&path) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Created: {}", path.display()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Create folder failed: {e}"),
        },
    }
}

fn run_trash(cmd: &CommandPayload) -> SystemCmdResult {
    let paths: Vec<&str> = cmd.args.iter().map(|s| s.as_str()).collect();
    if paths.is_empty() {
        return SystemCmdResult { success: false, message: "No files specified".into() };
    }

    let quoted = paths
        .iter()
        .map(|p| format!("'{p}'"))
        .collect::<Vec<_>>()
        .join(", ");

    // Use Shell.Application COM via PowerShell to send to recycle bin
    let script = format!(
        "$shell = New-Object -ComObject Shell.Application; \
         $items = @({quoted}); \
         foreach ($item in $items) {{ \
           $shell.Namespace(0).ParseName((Get-Item $item).Name).InvokeVerb('delete') \
         }}"
    );
    match powershell(&script) {
        Ok(_) => SystemCmdResult { success: true, message: format!("Moved to recycle bin") },
        Err(e) => SystemCmdResult { success: false, message: e },
    }
}

fn run_open_with(cmd: &CommandPayload) -> SystemCmdResult {
    let path = cmd.args.first().cloned().unwrap_or_default();
    match build_sys_cmd("rundll32.exe", &CommandPayload {
        exe: "rundll32.exe".into(),
        args: vec!["shell32.dll,OpenAs_RunDLL".into(), path],
        ..cmd.clone()
    })
    .spawn()
    {
        Ok(_) => SystemCmdResult { success: true, message: "Open With dialog launched".into() },
        Err(e) => SystemCmdResult { success: false, message: format!("OpenWith failed: {e}") },
    }
}

fn run_copy_path(cmd: &CommandPayload) -> SystemCmdResult {
    let text = cmd.args.join("\n");
    if text.is_empty() {
        return SystemCmdResult { success: false, message: "No paths to copy".into() };
    }

    let script = format!("Set-Clipboard -Value '{text}'");
    match powershell(&script) {
        Ok(_) => SystemCmdResult { success: true, message: "Paths copied to clipboard".into() },
        Err(e) => SystemCmdResult { success: false, message: e },
    }
}

// ── Utility ───────────────────────────────────────────────────────────────

/// Return a unique path by appending ` (2)`, ` (3)`, … if the target
/// already exists.
fn unique_path(path: &std::path::Path) -> std::path::PathBuf {
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

    // Fallback (should never happen)
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
    fn parse_all_variants() {
        assert_eq!("@unzip".parse(), Ok(SystemCommand::Unzip));
        assert_eq!("@zip".parse(), Ok(SystemCommand::Zip));
        assert_eq!("@rename".parse(), Ok(SystemCommand::Rename));
        assert_eq!("@new-file".parse(), Ok(SystemCommand::NewFile));
        assert_eq!("@new-folder".parse(), Ok(SystemCommand::NewFolder));
        assert_eq!("@trash".parse(), Ok(SystemCommand::Trash));
        assert_eq!("@open-with".parse(), Ok(SystemCommand::OpenWith));
        assert_eq!("@copy-path".parse(), Ok(SystemCommand::CopyPath));
    }

    #[test]
    fn parse_unknown() {
        assert!("@unknown".parse::<SystemCommand>().is_err());
        assert!("normal_exe".parse::<SystemCommand>().is_err());
    }

    #[test]
    fn is_system_cmd() {
        assert!(is_system_command("@unzip"));
        assert!(is_system_command("@zip"));
        assert!(!is_system_command("cmd"));
        assert!(!is_system_command(""));
    }
}
