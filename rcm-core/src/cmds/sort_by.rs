//! `@sort-by` - Change the Windows 11 Explorer sort column for a directory.

use crate::types::CommandPayload;

use super::SystemCmdResult;
use super::shell_folder_view::{
    default_ascending, property_key_from_arg, same_property_key, target_dir, with_folder_view,
};
use windows::Win32::UI::Shell::{SORT_ASCENDING, SORT_DESCENDING, SORTCOLUMN, SORTDIRECTION};

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

    match with_folder_view(&dir, |view| {
        let direction = unsafe { next_sort_direction(view, &propkey) };
        crate::log::info(
            "Rust::sort_by",
            &format!(
                "setting sort-by '{sort_key}' ({}) for '{}'",
                direction_label(direction),
                dir.display()
            ),
        );

        let column = SORTCOLUMN { propkey, direction };

        unsafe { view.SetSortColumns(&[column]) }
    }) {
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

unsafe fn next_sort_direction(
    view: &windows::Win32::UI::Shell::IFolderView2,
    propkey: &windows::Win32::Foundation::PROPERTYKEY,
) -> SORTDIRECTION {
    let count = unsafe { view.GetSortColumnCount() }.unwrap_or_default();
    if count <= 0 {
        return sort_direction(default_ascending(propkey));
    }

    let mut columns = vec![SORTCOLUMN::default(); count as usize];
    if unsafe { view.GetSortColumns(&mut columns) }.is_err() {
        return sort_direction(default_ascending(propkey));
    }

    let Some(current) = columns.first() else {
        return sort_direction(default_ascending(propkey));
    };

    if same_property_key(&current.propkey, propkey) {
        // Same key — flip the direction (toggle).
        if current.direction == SORT_ASCENDING {
            SORT_DESCENDING
        } else {
            SORT_ASCENDING
        }
    } else {
        // Different key — start from the sensible default for that column.
        sort_direction(default_ascending(propkey))
    }
}

fn sort_direction(ascending: bool) -> SORTDIRECTION {
    if ascending {
        SORT_ASCENDING
    } else {
        SORT_DESCENDING
    }
}

fn direction_label(direction: SORTDIRECTION) -> &'static str {
    if direction == SORT_DESCENDING {
        "descending"
    } else {
        "ascending"
    }
}
