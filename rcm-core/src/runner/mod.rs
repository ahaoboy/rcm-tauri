//! Command execution engine.
//!
//! Provides [`execute`] (capture output) and [`spawn`] (fire-and-forget)
//! for running external processes or built-in `@xxx` system commands.

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
/// natively via [`cmds::SystemCommand`].
pub async fn execute(cmd: &CommandPayload) -> ExecResult {
    if cmds::is_system_command(&cmd.cmd) {
        return run_system_cmd(cmd);
    }

    let mut command = build_command(cmd);

    println!("execute: {:?}", command);
    match command.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if !output.status.success() {
                eprintln!(
                    "execute '{}' failed (exit {:?}): {}",
                    cmd.cmd,
                    output.status.code(),
                    stderr
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
            eprintln!("execute '{}' spawn error: {}", cmd.cmd, e);
            ExecResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to spawn {}: {}", cmd.cmd, e),
                exit_code: None,
            }
        }
    }
}

/// Spawn a command as a detached child process (fire-and-forget).
///
/// Returns immediately; the child runs independently.
pub async fn spawn(cmd: &CommandPayload) -> Result<(), String> {
    if cmds::is_system_command(&cmd.cmd) {
        let result = run_system_cmd(cmd);
        return if result.success {
            Ok(())
        } else {
            Err(result.stderr)
        };
    }

    let mut command = build_command(cmd);

    command
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to spawn {}: {}", cmd.cmd, e))
}

/// Run a `@xxx` system command and convert its result to [`ExecResult`].
fn run_system_cmd(cmd: &CommandPayload) -> ExecResult {
    println!("run_system_cmd: {:?}", cmd);
    match cmd.cmd.parse::<cmds::SystemCommand>() {
        Ok(sys_cmd) => {
            let result = sys_cmd.run(cmd);
            ExecResult {
                success: result.success,
                stdout: if result.success {
                    result.message.clone()
                } else {
                    String::new()
                },
                stderr: if result.success {
                    String::new()
                } else {
                    result.message.clone()
                },
                exit_code: if result.success { Some(0) } else { Some(1) },
            }
        }
        Err(e) => ExecResult {
            success: false,
            stdout: String::new(),
            stderr: e,
            exit_code: Some(1),
        },
    }
}
