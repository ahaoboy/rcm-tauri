//! `@group-by` - Change the Windows 11 Explorer group column for a directory.

use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::UI::Shell::IFolderView2;

use super::shell_folder_view::{
    PKEY_NULL, default_ascending, property_key_from_arg, same_property_key, target_dir,
    with_folder_view,
};
use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// Accepted group keys: `property_key_from_arg`'s set plus `none`.
const GROUP_KEY_EXPECTED: &str = "one of: name, date-modified, type, size, date-created, none";

/// `@group-by` — set (and toggle) the Explorer group column for `cwd`.
pub struct GroupBy;

/// Arguments for [`GroupBy`].
struct Args {
    /// Requested group key, echoed back in the result.
    key: String,
    /// Resolved property key to group by.
    propkey: PROPERTYKEY,
    /// Directory whose view is updated.
    cwd: String,
}

impl GroupBy {
    /// Extract the group key and resolve it to a property key.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        let key = CmdArgs::of(payload).required(0, "key", GROUP_KEY_EXPECTED)?;
        let propkey = if key == "none" {
            PKEY_NULL
        } else {
            property_key_from_arg(key)
                .ok_or_else(|| CmdError::invalid("key", key, GROUP_KEY_EXPECTED))?
        };
        Ok(Args {
            key: key.to_owned(),
            propkey,
            cwd: payload.cwd.clone(),
        })
    }

    /// Apply the group column to the Explorer view.
    fn execute(args: Args) -> Result<String, CmdError> {
        let key = args.key.as_str();
        let dir = target_dir(&args.cwd)?;

        with_folder_view(&dir, |view| unsafe {
            let ascending = if key == "none" {
                false
            } else {
                next_group_ascending(view, &args.propkey)
            };

            crate::log::info(
                "Rust::group_by",
                &format!(
                    "setting group-by '{key}' ({}) for '{}'",
                    direction_label(ascending),
                    dir.display()
                ),
            );

            view.SetGroupBy(&args.propkey, ascending)
        })
        .map_err(|message| {
            crate::log::error("Rust::group_by", &message);
            CmdError::failed(message)
        })?;

        Ok(format!("Group by set to {key}"))
    }
}

impl Command for GroupBy {
    fn id(&self) -> &'static str {
        "@group-by"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

unsafe fn next_group_ascending(view: &IFolderView2, propkey: &PROPERTYKEY) -> bool {
    let mut current_key = PROPERTYKEY::default();
    let mut current_ascending = windows::core::BOOL::default();

    if unsafe { view.GetGroupBy(&mut current_key, Some(&mut current_ascending)) }.is_err() {
        return default_ascending(propkey);
    }

    if same_property_key(&current_key, propkey) {
        // Same key — flip the direction (toggle).
        !current_ascending.as_bool()
    } else {
        // Different key — start from the sensible default for that column.
        default_ascending(propkey)
    }
}

fn direction_label(ascending: bool) -> &'static str {
    if ascending { "ascending" } else { "descending" }
}
