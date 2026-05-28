//! Command execution — runs system commands and shell commands.
//! Direct port of src-tauri/src/cmd.rs, adapted for Slint (no Tauri dependency).
//!
//! Commands whose `exe` starts with `@` (e.g. `@unzip`, `@zip`) are routed
//! to [`rcm_core::system_cmd::SystemCommand`] for native handling.

use rcm_core::CommandPayload;
use rcm_core::system_cmd;
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

/// Execute a single command asynchronously.
pub async fn execute(cmd: CommandPayload) -> ExecResult {
    if system_cmd::is_system_command(&cmd.exe) {
        return run_system_cmd(&cmd);
    }

    let mut command = build_command(&cmd);

    println!("[cmd::execute] {:?}", command);
    match command.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if !output.status.success() {
                eprintln!(
                    "[cmd::execute] '{}' failed (exit {:?}): {}",
                    cmd.exe,
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
            eprintln!("[cmd::execute] '{}' spawn error: {}", cmd.exe, e);
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
pub async fn spawn_command(cmd: CommandPayload) -> Result<(), String> {
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

/// Run a `@xxx` system command and convert its result to [`ExecResult`].
fn run_system_cmd(cmd: &CommandPayload) -> ExecResult {
    println!("[cmd::run_system_cmd] {:?}", cmd);
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
        Err(e) => ExecResult {
            success: false,
            stdout: String::new(),
            stderr: e,
            exit_code: Some(1),
        },
    }
}

/// Build a `tokio::process::Command` from our payload descriptor.
fn build_command(cmd: &CommandPayload) -> Command {
    let exe = resolve_exe(&cmd.exe);
    let mut command = Command::new(&exe);

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

/// Resolve a bare command name to a full path via the `which` crate.
fn resolve_exe(exe: &str) -> String {
    if exe.contains('\\') || exe.contains('/') {
        return exe.to_string();
    }

    match which::which(exe) {
        Ok(full) => {
            let resolved = full.to_string_lossy().into_owned();
            eprintln!("[cmd::resolve_exe] '{}' -> '{}'", exe, resolved);
            resolved
        }
        Err(_) => {
            eprintln!("[cmd::resolve_exe] '{}' not found in PATH", exe);
            exe.to_string()
        }
    }
}
