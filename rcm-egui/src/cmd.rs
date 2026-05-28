//! Command execution — runs external processes and system commands.
//!
//! Mirrors the Tauri `cmd.rs` module. Commands whose `exe` starts with `@`
//! (e.g. `@unzip`, `@zip`) are routed to `rcm_core::system_cmd::SystemCommand`
//! for native handling.

use rcm_core::CommandPayload;
use rcm_core::system_cmd;
use tokio::process::Command;

/// Windows flag to prevent console windows from appearing.
#[cfg(target_os = "windows")]
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
/// System commands (prefixed with `@`) are intercepted and handled natively.
pub async fn execute(cmd: CommandPayload) -> ExecResult {
    // Route @xxx system commands to native handler
    if system_cmd::is_system_command(&cmd.exe) {
        return run_system_cmd(&cmd);
    }

    let mut command = build_command(&cmd);

    rcm_core::log::info("cmd", &format!("execute: {:?}", command));
    match command.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if !output.status.success() {
                rcm_core::log::error(
                    "cmd",
                    &format!(
                        "execute '{}' failed (exit {:?}): {}",
                        cmd.exe,
                        output.status.code(),
                        stderr
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
            rcm_core::log::error("cmd", &format!("execute '{}' spawn error: {}", cmd.exe, e));
            ExecResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to spawn {}: {}", cmd.exe, e),
                exit_code: None,
            }
        }
    }
}

/// Spawn a command as a detached child process (fire-and-forget).
/// Returns immediately; the child runs independently.
///
/// System commands (prefixed with `@`) are intercepted and handled natively.
pub async fn spawn_command(cmd: CommandPayload) -> Result<(), String> {
    // Route @xxx system commands to native handler
    if system_cmd::is_system_command(&cmd.exe) {
        let result = run_system_cmd(&cmd);
        return if result.success {
            Ok(())
        } else {
            Err(result.stderr)
        };
    }

    let mut command = build_command(&cmd);

    command
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to spawn {}: {}", cmd.exe, e))
}

/// Build a tokio `Command` from a `CommandPayload`.
fn build_command(cmd: &CommandPayload) -> Command {
    let mut command = Command::new(&cmd.exe);

    // Add arguments
    for arg in &cmd.args {
        command.arg(arg);
    }

    // Set working directory if specified and not empty
    if !cmd.cwd.is_empty() {
        command.current_dir(&cmd.cwd);
    }

    // Prevent console window from appearing on Windows
    #[cfg(target_os = "windows")]
    {
        #[allow(unused_imports)]
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
}

/// Run a `@xxx` system command and convert its result to [`ExecResult`].
fn run_system_cmd(cmd: &CommandPayload) -> ExecResult {
    rcm_core::log::info("cmd", &format!("run_system_cmd: {:?}", cmd));
    match cmd.exe.parse::<system_cmd::SystemCommand>() {
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
        Err(e) => {
            rcm_core::log::error("cmd", &format!("unknown system command '{}': {e}", cmd.exe));
            ExecResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Unknown system command: {}", cmd.exe),
                exit_code: Some(1),
            }
        }
    }
}
