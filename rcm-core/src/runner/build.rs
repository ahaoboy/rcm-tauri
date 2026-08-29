//! Build a [`tokio::process::Command`] from a [`CommandPayload`] descriptor.
//!
//! Handles path resolution, window visibility (Windows Terminal vs. hidden),
//! shell selection for interactive terminal sessions, and re-reading the
//! latest environment variables from the registry before each spawn.

use crate::types::{CommandPayload, WindowMode};
use std::collections::HashMap;
use tokio::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;

/// Path expansion helper — expand `%VAR%` references in a value (e.g.
/// `%SystemRoot%`) using `cmdexpand`.
///
/// Lookup order for each `%VAR%`:
///   1. The freshly-read registry map (uppercased keys).
///   2. The current process environment (`std::env`), as a fallback.
///
/// Falling back to the process env prevents critical system variables that are
/// *not* stored in the registry (e.g. `SystemDrive`, `USERNAME`, `TEMP`,
/// `ComSpec` deps) from expanding to an empty string — which would otherwise
/// corrupt `Path` and break launching shells (e.g. VS Code's terminal).
fn expand_path(value: &str, upper: &HashMap<String, String>) -> String {
    // cmdexpand's context closure must be 'static, so clone the map into it.
    let ctx = upper.clone();
    cmdexpand::Expander::new(value)
        .add_context(&move |name: &str| {
            ctx.get(&name.to_uppercase())
                .cloned()
                .or_else(|| std::env::var(name).ok())
                .or_else(|| std::env::var(name.to_uppercase()).ok())
        })
        .expand()
        .unwrap_or_else(|_| value.to_string())
}

/// Read the latest system + user environment variables from the registry.
///
/// The current process may have been started long ago and caches an old
/// snapshot of `PATH` etc. Reading fresh from the registry ensures newly added
/// user/system variables are picked up when spawning children (e.g. VS Code
/// sees variables the user added after this program started).
///
/// - System env: `HKLM\SYSTEM\...\Session Manager\Environment`
/// - User env:   `HKCU\Environment`
/// - User `PATH` is appended after the system `PATH`.
/// - `%VAR%` references in values are expanded using the collected map.
fn get_fresh_windows_envs() -> HashMap<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut envs: HashMap<String, String> = HashMap::new();

    // 1. System environment variables
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(sys_key) =
        hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")
    {
        for item in sys_key.enum_values().flatten() {
            if let Ok(val) = sys_key.get_value::<String, _>(&item.0) {
                envs.insert(item.0.clone(), val);
            }
        }
    }

    // 2. User environment variables (override same-named system vars)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(user_key) = hkcu.open_subkey("Environment") {
        for item in user_key.enum_values().flatten() {
            if let Ok(val) = user_key.get_value::<String, _>(&item.0) {
                // Special case PATH: user PATH is appended after system PATH.
                if item.0.eq_ignore_ascii_case("PATH") {
                    let key = envs
                        .keys()
                        .find(|k| k.eq_ignore_ascii_case("PATH"))
                        .cloned()
                        .unwrap_or_else(|| "Path".to_string());
                    if let Some(sys_path) = envs.get(&key) {
                        envs.insert(key, format!("{};{}", sys_path, val));
                    } else {
                        envs.insert(key, val);
                    }
                    continue;
                }
                envs.insert(item.0, val);
            }
        }
    }

    // 3. Expand %VAR% references. Snapshot the map first (as an uppercased
    //    index) so expansion can borrow it while we mutate `envs`.
    let upper: HashMap<String, String> = envs
        .iter()
        .map(|(k, v)| (k.to_uppercase(), v.clone()))
        .collect();
    for v in envs.values_mut() {
        if v.contains('%') {
            *v = expand_path(v, &upper);
        }
    }

    envs
}

/// Add fresh registry environment variables to a command's environment,
/// but only those the process does not already define.
///
/// The process's own env is left untouched (existing vars keep their values,
/// so runtime/shell additions to e.g. `Path` are preserved). Only variables
/// that are entirely absent from the process env are taken from the registry —
/// this is how newly added user/system variables get picked up at spawn time
/// without clobbering anything.
fn apply_fresh_env(command: &mut Command) {
    let fresh = get_fresh_windows_envs();
    if fresh.is_empty() {
        return;
    }

    // Compare case-insensitively (Windows env names are case-insensitive).
    let existing: Vec<String> = std::env::vars()
        .map(|(k, _)| k.to_uppercase())
        .collect();

    for (key, val) in fresh {
        let upper = key.to_uppercase();
        if !existing.contains(&upper) {
            command.env(key, val);
        }
    }
}

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
                format!("& '{}'", exe)
            } else {
                let all: Vec<String> = std::iter::once(exe.as_str())
                    .chain(cmd.args.iter().map(String::as_str))
                    .map(|a| format!("'{}'", a))
                    .collect();
                format!("& {}", all.join(" "))
            };
            let shell = resolve_shell();
            let mut command = Command::new("wt");
            if !cmd.cwd.is_empty() {
                command.arg("-d");
                command.arg(&cmd.cwd);
            }
            command.args([&shell, "-NoExit", "-Command", &target]);
            apply_fresh_env(&mut command);
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
    apply_fresh_env(&mut command);
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
