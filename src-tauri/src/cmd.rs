//! Tauri command wrappers — delegates to [`rcm_core::runner`] for all
//! process execution and system-command routing.

use rcm_core::CommandPayload;
use rcm_core::runner;

/// Execute a command and capture its stdout/stderr.
#[tauri::command]
pub async fn execute(cmd: CommandPayload) -> runner::ExecResult {
    runner::execute(&cmd).await
}

/// Spawn a command as a detached child process (fire-and-forget).
#[tauri::command]
pub async fn spawn_command(cmd: CommandPayload) -> Result<(), String> {
    runner::spawn(&cmd).await
}
