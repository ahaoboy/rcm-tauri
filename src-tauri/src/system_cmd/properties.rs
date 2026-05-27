//! `@properties` — Open the Windows file/folder properties dialog.

use crate::rcm::CommandPayload;
use super::SystemCmdResult;
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::Win32::UI::Shell::SEE_MASK_INVOKEIDLIST;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => return SystemCmdResult { success: false, message: "No file specified".into() },
    };

    crate::log::info("Rust::properties", &format!("opening properties for '{path}'"));

    // Encode path and "properties" verb as UTF-16 null-terminated strings
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let verb: Vec<u16> = "properties\0".encode_utf16().collect();

    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        lpVerb: PCWSTR::from_raw(verb.as_ptr()),
        lpFile: PCWSTR::from_raw(wide_path.as_ptr()),
        nShow: SW_SHOW.0 as i32,
        fMask: SEE_MASK_INVOKEIDLIST,
        ..Default::default()
    };

    let result = unsafe { ShellExecuteExW(&mut sei) };

    match result {
        Ok(()) => {
            crate::log::info("Rust::properties", "ShellExecuteExW OK");
            SystemCmdResult { success: true, message: "Properties opened".into() }
        }
        Err(e) => {
            crate::log::error("Rust::properties", &format!("ShellExecuteExW failed: {e}"));
            SystemCmdResult {
                success: false,
                message: format!("ShellExecuteExW failed: {e}"),
            }
        }
    }
}
