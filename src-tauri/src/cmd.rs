//! Direct native command execution — bypasses the JS VM.
//! Takes a `CommandPayload` from the frontend and runs it asynchronously via tokio.
//!
//! Commands whose `exe` starts with `@` (e.g. `@unzip`, `@zip`) are routed
//! to [`rcm_core::system_cmd::SystemCommand`] for native handling.

use rcm_core::CommandPayload;
use rcm_core::cmds;
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
///
/// System commands (prefixed with `@`) are intercepted and handled natively.
#[tauri::command]
pub async fn execute(cmd: CommandPayload) -> ExecResult {
    // Route @xxx system commands to native handler
    if cmds::is_system_command(&cmd.exe) {
        return run_system_cmd(&cmd);
    }

    let mut command = build_command(&cmd);

    println!("execute: {:?}", command);
    match command.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if !output.status.success() {
                eprintln!(
                    "execute '{}' failed (exit {:?}): {}",
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
            eprintln!("execute '{}' spawn error: {}", cmd.exe, e);
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
#[tauri::command]
pub async fn spawn_command(cmd: CommandPayload) -> Result<(), String> {
    // Route @xxx system commands to native handler
    if cmds::is_system_command(&cmd.exe) {
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
    println!("run_system_cmd: {:?}", cmd);
    match cmd.exe.parse::<cmds::SystemCommand>() {
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
///
/// Bare command names are resolved to full paths via the `which` crate
/// before spawning, so that MSYS2 / Cygwin `PATH` entries and
/// extensionless shell scripts don't cause `CreateProcess` failures.
fn build_command(cmd: &CommandPayload) -> Command {
    let exe = resolve_exe(&cmd.exe);

    // When the caller requests a visible window, launch via Windows Terminal
    // so the user can see the program's stdout/stderr in a dedicated tab.
    let show_window = matches!(cmd.window.as_str(), "Show" | "Visible" | "Maximized");

    if show_window {
        // Launch via Windows Terminal + PowerShell so the user can see
        // the program's output in a dedicated tab that stays open.
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
    // Already a path — use as-is
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
    // `powershell.exe` exists on every Windows 10+ installation.
    "powershell".to_string()
}
