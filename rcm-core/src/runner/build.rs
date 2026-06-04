//! Build a [`tokio::process::Command`] from a [`CommandPayload`] descriptor.
//!
//! Handles path resolution, window visibility (Windows Terminal vs. hidden),
//! and shell selection for interactive terminal sessions.

use crate::types::CommandPayload;
use tokio::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Build a ready-to-spawn [`Command`] from the frontend payload.
///
/// - Bare command names are resolved to full paths via `which`.
/// - When `window` is `"Show"` / `"Visible"` / `"Maximized"`, the command
///   is launched inside Windows Terminal + PowerShell so the user can see
///   the output in a visible tab.
/// - Otherwise `CREATE_NO_WINDOW` is applied to suppress console windows.
pub fn build_command(cmd: &CommandPayload) -> Command {
    let exe = resolve_exe(&cmd.exe);

    let show_window = matches!(cmd.window.as_str(), "Show" | "Visible" | "Maximized");

    if show_window {
        let target = if cmd.args.is_empty() {
            format!("\"{}\"", exe)
        } else {
            format!("\"{}\" {}", exe, cmd.args.join(" "))
        };
        let shell = resolve_shell();
        let mut command = Command::new("wt");
        command.args([&shell, "-NoExit", "-Command", &target]);
        if !cmd.cwd.is_empty() {
            command.current_dir(&cmd.cwd);
        }
        command
    } else {
        let mut command = Command::new(&exe);
        if !cmd.args.is_empty() {
            command.args(&cmd.args);
        }
        if !cmd.cwd.is_empty() {
            command.current_dir(&cmd.cwd);
        }

        #[cfg(target_os = "windows")]
        {
            command.creation_flags(CREATE_NO_WINDOW);
        }

        command
    }
}

/// Resolve a bare command name to a full path via the `which` crate.
///
/// Handles PATHEXT resolution correctly on Windows (`.exe` → `.cmd`
/// → `.bat` → …), so `code` resolves to `code.cmd` rather than the
/// extensionless shell script.
///
/// Returns the original name unchanged if it is already a path, or if
/// resolution fails.
fn resolve_exe(exe: &str) -> String {
    if exe.contains('\\') || exe.contains('/') {
        return exe.to_string();
    }

    match which::which(exe) {
        Ok(full) => {
            let resolved = full.to_string_lossy().into_owned();
            eprintln!("resolve_exe: '{}' -> '{}'", exe, resolved);
            resolved
        }
        Err(_) => {
            eprintln!("resolve_exe: '{}' not found in PATH", exe);
            exe.to_string()
        }
    }
}

/// Resolve the PowerShell executable to use in Windows Terminal.
///
/// Tries `pwsh` (PowerShell 7+) first, falling back to the built-in
/// `powershell` (Windows PowerShell 5.1) which is always available.
fn resolve_shell() -> String {
    if let Ok(path) = which::which("pwsh") {
        return path.to_string_lossy().into_owned();
    }
    "powershell".to_string()
}
