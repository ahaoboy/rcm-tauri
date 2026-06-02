//! `@sort-by` - Change the Windows 11 Explorer sort column for a directory.

use crate::types::CommandPayload;

use super::SystemCmdResult;
use super::shell_folder_view::{property_key_from_arg, target_dir, with_folder_view};
use windows::Win32::UI::Shell::{SORT_ASCENDING, SORTCOLUMN};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let sort_key = match cmd.args.first() {
        Some(arg) if !arg.is_empty() => arg.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No sort key specified".into(),
            };
        }
    };

    let Some(propkey) = property_key_from_arg(sort_key) else {
        return SystemCmdResult {
            success: false,
            message: format!("Unsupported sort key: {sort_key}"),
        };
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
        "Rust::sort_by",
        &format!("setting sort-by '{sort_key}' for '{}'", dir.display()),
    );

    let column = SORTCOLUMN {
        propkey,
        direction: SORT_ASCENDING,
    };

    match with_folder_view(&dir, |view| unsafe { view.SetSortColumns(&[column]) }) {
        Ok(()) => SystemCmdResult {
            success: true,
            message: format!("Sort by set to {sort_key}"),
        },
        Err(message) => {
            crate::log::error("Rust::sort_by", &message);
            SystemCmdResult {
                success: false,
                message,
            }
        }
    }
}
