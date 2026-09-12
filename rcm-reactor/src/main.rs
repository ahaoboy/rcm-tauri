// Prevents an additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! rcm-reactor — the RCM context-menu engine rebuilt with
//! [`windows_reactor`] (declarative WinUI 3) + `tray-icon`.
//!
//! Functionality is ported one-to-one from the Tauri build in `rcm-tauri`:
//! the shared `rcm-core` / `rcm-vm` / `rcm-com` / `rcm-reg` crates provide all
//! system behaviour, while Reactor replaces the WebView frontend and
//! `tray-icon` replaces `tauri`'s tray.

mod app;
mod config_editor;
mod error_window;
mod events;
mod exec;
mod menu_runtime;
mod menu_window;
mod metrics;
mod monitor;
mod tray;
mod visuals;
mod win32;

fn main() {
    // Step 1: check if another RCM process is already running.
    if rcm_core::process::is_rcm_process_running() {
        eprintln!("rcm-reactor: another instance is already running");
        return;
    }

    if let Err(e) = windows_reactor::App::run_component::<app::RcmApp>(()) {
        eprintln!("rcm-reactor: exited with error: {e}");
    }
}
