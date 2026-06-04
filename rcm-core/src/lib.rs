//! RCM Core — system & API calls, types, and utilities.
//! This crate is framework-agnostic and can be used with any frontend.

pub mod clipboard;
pub mod config;
pub mod lang;
pub mod log;
pub mod menu_defaults;
pub mod registry;
pub mod cmds;
pub mod types;

// Re-export commonly used types
pub use types::{CommandPayload, FileInfo, IndexPath, InvokeProps, Item, Menu, NavigateResult};
