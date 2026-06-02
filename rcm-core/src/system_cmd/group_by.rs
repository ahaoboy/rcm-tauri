//! `@group-by` - Change the Windows 11 Explorer group column for a directory.

use crate::types::CommandPayload;

use super::SystemCmdResult;
use super::shell_folder_view::{PKEY_NULL, property_key_from_arg, target_dir, with_folder_view};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let group_key = match cmd.args.first() {
        Some(arg) if !arg.is_empty() => arg.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No group key specified".into(),
            };
        }
    };

    let (propkey, ascending) = if group_key == "none" {
        (PKEY_NULL, false)
    } else {
        match property_key_from_arg(group_key) {
            Some(propkey) => (propkey, true),
            None => {
                return SystemCmdResult {
                    success: false,
                    message: format!("Unsupported group key: {group_key}"),
                };
            }
        }
    };

    let dir = match target_dir(&cmd.cwd) {
        Ok(dir) => dir,
        Err(message) => {
            return SystemCmdResult {
                success: false,
                message,
            };
        }
    };

    crate::log::info(
        "Rust::group_by",
        &format!("setting group-by '{group_key}' for '{}'", dir.display()),
    );

    match with_folder_view(&dir, |view| unsafe {
        view.SetGroupBy(&propkey, ascending)
    }) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Group by set to {group_key}"),
        },
        Err(message) => {
            crate::log::error("Rust::group_by", &message);
            SystemCmdResult {
                success: false,
                message,
            }
        }
    }
}
