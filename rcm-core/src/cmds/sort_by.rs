//! `@sort-by` - Change the Windows 11 Explorer sort column for a directory.

use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::UI::Shell::{
    IFolderView2, SORT_ASCENDING, SORT_DESCENDING, SORTCOLUMN, SORTDIRECTION,
};

use super::shell_folder_view::{
    default_ascending, property_key_from_arg, same_property_key, target_dir, with_folder_view,
};
use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// Accepted sort keys, matching `property_key_from_arg`.
const SORT_KEY_EXPECTED: &str = "one of: name, date-modified, type, size, date-created";

/// `@sort-by` — set (and toggle) the Explorer sort column for `cwd`.
pub struct SortBy;

/// Arguments for [`SortBy`].
struct Args {
    /// Requested sort key, echoed back in the result.
    key: String,
    /// Resolved property key to sort by.
    propkey: PROPERTYKEY,
    /// Directory whose view is updated.
    cwd: String,
}

impl SortBy {
    /// Extract the sort key and resolve it to a property key.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let key = CmdArgs::of(payload).required(0, "key", SORT_KEY_EXPECTED)?;
        let propkey = property_key_from_arg(key)
            .ok_or_else(|| CmdError::invalid("key", key, SORT_KEY_EXPECTED))?;
        Ok(Args {
            key: key.to_owned(),
            propkey,
            cwd: payload.cwd.clone(),
        })
    }

    /// Apply the sort column to the Explorer view.
    fn execute(args: Args) -> Result<String, CmdError> {
        let key = args.key.as_str();
        let dir = target_dir(&args.cwd)?;

        with_folder_view(&dir, |view| {
            let direction = unsafe { next_sort_direction(view, &args.propkey) };
            crate::log::info(
                "Rust::sort_by",
                &format!(
                    "setting sort-by '{key}' ({}) for '{}'",
                    direction_label(direction),
                    dir.display()
                ),
            );

            let column = SORTCOLUMN {
                propkey: args.propkey,
                direction,
            };

            unsafe { view.SetSortColumns(&[column]) }
        })
        .map_err(|message| {
            crate::log::error("Rust::sort_by", &message);
            CmdError::failed(message)
        })?;

        Ok(format!("Sort by set to {key}"))
    }
}

impl Command for SortBy {
    fn id(&self) -> &'static str {
        "@sort-by"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

unsafe fn next_sort_direction(view: &IFolderView2, propkey: &PROPERTYKEY) -> SORTDIRECTION {
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
