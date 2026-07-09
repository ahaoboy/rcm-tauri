pub mod clipboard;
pub mod cmds;
pub mod config;
pub mod lang;
pub mod log;
pub mod menu;
pub mod registry;
pub mod runner;
pub mod types;
pub use types::{CommandPayload, FileInfo, IndexPath, InvokeProps, Item, Menu, NavigateResult};

/// Windows process creation flag: suppress console window.
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Create a [`std::process::Command`] with `CREATE_NO_WINDOW` pre-set.
/// Use this instead of `Command::new()` to avoid console flash on Windows.
pub fn sys_cmd(program: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// Directory containing the current executable.
pub fn exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}
