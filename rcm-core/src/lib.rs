//! RCM Core — system & API calls, types, and utilities.
//! This crate is framework-agnostic and can be used with any frontend.

pub mod config;
pub mod lang;
pub mod log;
pub mod menu_defaults;
pub mod registry;
pub mod system_cmd;
pub mod types;

// Re-export commonly used types
pub use types::{CommandPayload, FileInfo, InvokeProps, Item, Menu, IndexPath, NavigateResult};
