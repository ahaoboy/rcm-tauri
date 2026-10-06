//! `@properties` — Open the Windows file/folder properties dialog.

use windows::Win32::UI::Shell::SEE_MASK_INVOKEIDLIST;
use windows::Win32::UI::Shell::{SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::core::PCWSTR;

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@properties` — open the shell properties dialog for a path.
pub struct Properties;

/// Arguments for [`Properties`].
struct Args {
    /// Path to show properties for.
    path: String,
}

impl Properties {
    /// Extract the target path.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            path: CmdArgs::of(payload)
                .required(0, "path", "a file or folder path")?
                .to_owned(),
        })
    }

    /// Invoke the shell "properties" verb.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info(
            "Rust::properties",
            &format!("opening properties for '{path}'"),
        );

        // Encode path and "properties" verb as UTF-16 null-terminated strings.
        let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let verb: Vec<u16> = "properties\0".encode_utf16().collect();

        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            lpVerb: PCWSTR::from_raw(verb.as_ptr()),
            lpFile: PCWSTR::from_raw(wide_path.as_ptr()),
            nShow: SW_SHOW.0,
            fMask: SEE_MASK_INVOKEIDLIST,
            ..Default::default()
        };

        if let Err(e) = unsafe { ShellExecuteExW(&mut sei) } {
            crate::log::error("Rust::properties", &format!("ShellExecuteExW failed: {e}"));
            return Err(CmdError::failed(format!("ShellExecuteExW failed: {e}")));
        }

        crate::log::info("Rust::properties", "ShellExecuteExW OK");
        Ok("Properties opened".into())
    }
}

impl Command for Properties {
    fn id(&self) -> &'static str {
        "@properties"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}
