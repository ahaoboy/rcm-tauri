//! Command execution engine.
//!
//! Provides [`execute`] (capture output) and [`execute_logged`] for running
//! external processes or built-in `@xxx` system commands.

mod build;

use crate::cmds;
use crate::types::CommandPayload;
use build::build_command;

/// Result of a command execution.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Execute a command and capture its stdout/stderr.
///
/// System commands (prefixed with `@`) are intercepted and handled
/// natively via [`cmds::run`].
pub async fn execute(cmd: &CommandPayload) -> ExecResult {
    if cmds::is_system_command(&cmd.cmd) {
        return run_system_cmd(cmd);
    }

    let mut command = build_command(cmd);

    match command.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if !output.status.success() {
                crate::log::warn(
                    "Runner",
                    &format!(
                        "execute '{}' failed (exit {:?}): {stderr}",
                        cmd.cmd,
                        output.status.code()
                    ),
                );
            }
            ExecResult {
                success: output.status.success(),
                stdout,
                stderr,
                exit_code: output.status.code(),
            }
        }
        Err(e) => {
            crate::log::error("Runner", &format!("execute '{}' spawn error: {e}", cmd.cmd));
            ExecResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to spawn {}: {}", cmd.cmd, e),
                exit_code: None,
            }
        }
    }
}

/// Run a `@xxx` system command and convert its result to [`ExecResult`].
///
/// This is the boundary that owns the transport shape: a successful message
/// goes to stdout, a [`cmds::CmdError`] goes to stderr, and both imply an exit
/// code. `cmds` itself stays free of these concerns.
fn run_system_cmd(cmd: &CommandPayload) -> ExecResult {
    match cmds::run(&cmd.cmd, cmd) {
        Some(Ok(message)) => system_ok(message),
        Some(Err(error)) => system_err(error.to_string()),
        None => system_err(format!("unknown system command: {}", cmd.cmd)),
    }
}

/// A successful built-in command result.
fn system_ok(message: String) -> ExecResult {
    ExecResult {
        success: true,
        stdout: message,
        stderr: String::new(),
        exit_code: Some(0),
    }
}

/// A failed built-in command result.
fn system_err(message: String) -> ExecResult {
    ExecResult {
        success: false,
        stdout: String::new(),
        stderr: message,
        exit_code: Some(1),
    }
}

/// Execute a command and log any failure, keyed by `tag`.
///
/// A menu command's outcome is not shown to the user — the menu window is
/// already gone — so the result is only ever logged. Both frontends spawn this
/// on their own runtime and ignore the return value.
pub async fn execute_logged(cmd: &CommandPayload, tag: &str) -> ExecResult {
    let result = execute(cmd).await;
    if !result.success {
        crate::log::error(tag, &format!("command '{}' FAILED: {result:?}", cmd.cmd));
    }
    result
}
