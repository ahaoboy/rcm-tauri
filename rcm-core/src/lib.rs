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

/// Directory containing the current executable.
pub fn exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}
