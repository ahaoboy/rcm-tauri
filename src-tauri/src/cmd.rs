//! Direct native command execution — bypasses the JS VM.
//! Takes a `CommandPayload` from the frontend and runs it synchronously in Rust.

use crate::rcm::CommandPayload;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Result of a synchronous command execution.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Execute a single command synchronously, returning stdout/stderr/exit code.
///
/// Uses `output()` so the caller blocks until the child process completes.
/// For fire-and-forget launching, use `spawn_command` instead.
#[tauri::command]
pub fn execute(cmd: CommandPayload) -> ExecResult {
    let mut command = build_command(&cmd);

    match command.output() {
        Ok(output) => ExecResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code(),
        },
        Err(e) => ExecResult {
            success: false,
            stdout: String::new(),
            stderr: format!("Failed to spawn {}: {}", cmd.exe, e),
            exit_code: None,
        },
    }
}

/// Spawn a command as a detached child process (fire-and-forget).
/// Returns immediately; the child runs independently.
#[tauri::command]
pub fn spawn_command(cmd: CommandPayload) -> Result<(), String> {
    let mut command = build_command(&cmd);

    command
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to spawn {}: {}", cmd.exe, e))
}

/// Build a `std::process::Command` from our payload descriptor.
fn build_command(cmd: &CommandPayload) -> Command {
    let mut command = Command::new(&cmd.exe);

    if !cmd.args.is_empty() {
        command.args(&cmd.args);
    }

    if !cmd.cwd.is_empty() {
        command.current_dir(&cmd.cwd);
    }

    #[cfg(target_os = "windows")]
    match cmd.window.as_str() {
        "Hidden" | "Minimized" => {
            command.creation_flags(CREATE_NO_WINDOW);
        }
        _ => {}
    }

    command
}
