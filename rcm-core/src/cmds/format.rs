//! `@format` — Open the Windows "Format" dialog for a drive.

use super::SystemCmdResult;
use crate::types::CommandPayload;
use windows::Win32::UI::Shell::SEE_MASK_INVOKEIDLIST;
use windows::Win32::UI::Shell::{SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::core::PCWSTR;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No drive specified".into(),
            };
        }
    };

    crate::log::info(
        "Rust::format",
        &format!("opening format dialog for '{path}'"),
    );

    // Encode path and "format" verb as UTF-16 null-terminated strings
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let verb: Vec<u16> = "format\0".encode_utf16().collect();

    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        lpVerb: PCWSTR::from_raw(verb.as_ptr()),
        lpFile: PCWSTR::from_raw(wide_path.as_ptr()),
        nShow: SW_SHOW.0,
        fMask: SEE_MASK_INVOKEIDLIST,
        ..Default::default()
    };

    match unsafe { ShellExecuteExW(&mut sei) } {
        Ok(()) => {
            crate::log::info("Rust::format", "ShellExecuteExW format OK");
            SystemCmdResult {
                success: true,
                message: "Format dialog opened".into(),
            }
        }
        Err(e) => {
            crate::log::error("Rust::format", &format!("ShellExecuteExW failed: {e}"));
            SystemCmdResult {
                success: false,
                message: format!("ShellExecuteExW failed: {e}"),
            }
        }
    }
}
