//! RCM egui library — native context menu with egui UI and system tray.
//!
//! This crate provides:
//! - `RcmEguiApp` — the main egui application that renders context menus
//! - `tray` — system tray icon with full settings menu
//! - `pipe` — IPC named pipe server for shell extension communication
//! - `cmd` — command execution (spawn processes, system commands)
//!
//! Architecture mirrors the Tauri version but uses egui/eframe instead of
//! Tauri's multi-window webview approach.

#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod cmd;
pub mod pipe;
pub mod tray;

pub use app::RcmEguiApp;

