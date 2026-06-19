//! Build a [`tokio::process::Command`] from a [`CommandPayload`] descriptor.
//!
//! Handles path resolution, window visibility (Windows Terminal vs. hidden),
//! and shell selection for interactive terminal sessions.

use crate::types::{CommandPayload, WindowMode};
use tokio::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;

/// Build a ready-to-spawn [`Command`] from the frontend payload.
///
/// - Bare command names are resolved to full paths via `which`.
/// - When `window` is `Show` / `Visible` / `Maximized`, the command
///   is launched inside Windows Terminal + PowerShell.
/// - `Minimized` launches the process minimized.
/// - `Hidden` suppresses the console window entirely via `CREATE_NO_WINDOW`.
pub fn build_command(cmd: &CommandPayload) -> Command {
    let exe = resolve_exe(&cmd.cmd);

    use WindowMode::*;
    match cmd.window {
        Visible | Maximized => {
            let target = if cmd.args.is_empty() {
                format!("\"{}\"", exe)
            } else {
                let quoted_args: Vec<String> =
                    cmd.args.iter().map(|a| format!("\"{}\"", a)).collect();
                format!("\"{}\" {}", exe, quoted_args.join(" "))
            };
            let shell = resolve_shell();
            let mut command = Command::new("wt");
            if !cmd.cwd.is_empty() {
                command.arg("-d");
                command.arg(&cmd.cwd);
            }
            command.args([&shell, "-NoExit", "-Command", &target]);
            command
        }
        Minimized => {
            let mut command = build_raw(cmd, &exe);
            #[cfg(target_os = "windows")]
            {
                command.creation_flags(SW_MINIMIZE.0 as u32);
            }
            command
        }
        Hidden => {
            let mut command = build_raw(cmd, &exe);
            #[cfg(target_os = "windows")]
            {
                command.creation_flags(CREATE_NO_WINDOW);
            }
            command
        }
    }
}

fn build_raw(cmd: &CommandPayload, exe: &str) -> Command {
    let mut command = Command::new(exe);
    if !cmd.args.is_empty() {
        command.args(&cmd.args);
    }
    if !cmd.cwd.is_empty() {
        command.current_dir(&cmd.cwd);
    }
    command
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
