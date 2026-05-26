//! Direct native command execution — bypasses the JS VM.
//! Takes a `CommandPayload` from the frontend and runs it asynchronously via tokio.

use crate::rcm::CommandPayload;
use tokio::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Result of a command execution.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Execute a single command asynchronously, returning stdout/stderr/exit code.
///
/// Uses `tokio::process::Command::output()` so it doesn't block the async runtime.
/// For fire-and-forget launching, use `spawn_command` instead.
#[tauri::command]
pub async fn execute(cmd: CommandPayload) -> ExecResult {
    let mut command = build_command(&cmd);

    match command.output().await {
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
pub async fn spawn_command(cmd: CommandPayload) -> Result<(), String> {
    let mut command = build_command(&cmd);

    command
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to spawn {}: {}", cmd.exe, e))
}

/// Build a `tokio::process::Command` from our payload descriptor.
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
