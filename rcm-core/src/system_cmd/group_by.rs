//! `@group-by` - Change the Windows 11 Explorer group column for a directory.

use crate::types::CommandPayload;

use super::SystemCmdResult;
use super::shell_folder_view::{
    PKEY_NULL, property_key_from_arg, same_property_key, target_dir, with_folder_view,
};
use windows::Win32::Foundation::PROPERTYKEY;

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

    let propkey = if group_key == "none" {
        PKEY_NULL
    } else {
        match property_key_from_arg(group_key) {
            Some(propkey) => propkey,
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

    match with_folder_view(&dir, |view| unsafe {
        let ascending = if group_key == "none" {
            false
        } else {
            next_group_ascending(view, &propkey)
        };

        crate::log::info(
            "Rust::group_by",
            &format!(
                "setting group-by '{group_key}' ({}) for '{}'",
                direction_label(ascending),
                dir.display()
            ),
        );

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

unsafe fn next_group_ascending(
    view: &windows::Win32::UI::Shell::IFolderView2,
    propkey: &PROPERTYKEY,
) -> bool {
    let mut current_key = PROPERTYKEY::default();
    let mut current_ascending = windows::core::BOOL::default();

    if unsafe { view.GetGroupBy(&mut current_key, Some(&mut current_ascending)) }.is_err() {
        return true;
    }

    !(same_property_key(&current_key, propkey) && current_ascending.as_bool())
}

fn direction_label(ascending: bool) -> &'static str {
    if ascending { "ascending" } else { "descending" }
}
