//! `@open-file-location` — Open the containing folder in Explorer and
//! select the file. For shortcut (.lnk) files, resolves the target first.

use std::path::Path;

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@open-file-location` — reveal a file in Explorer, resolving shortcuts.
pub struct OpenFileLocation;

/// Arguments for [`OpenFileLocation`].
struct Args {
    /// File (or `.lnk`) to reveal.
    path: String,
}

impl OpenFileLocation {
    /// Extract the file path.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            path: CmdArgs::of(payload)
                .required(0, "path", "a file path")?
                .to_owned(),
        })
    }

    /// Open the containing folder, selecting the file.
    fn execute(args: Args) -> Result<String, CmdError> {
        let path = args.path.as_str();
        crate::log::info("Rust::open_file_location", &format!("opening location for '{path}'"));

        // Resolve shortcut target if it's a .lnk file
        let target = if path.to_lowercase().ends_with(".lnk") {
            match resolve_shortcut(path) {
                Ok(t) => {
                    crate::log::info(
                        "Rust::open_file_location",
                        &format!("resolved shortcut '{path}' -> '{t}'"),
                    );
                    t
                }
                Err(e) => {
                    crate::log::info(
                        "Rust::open_file_location",
                        &format!("shortcut resolve failed: {e}, falling back to .lnk itself"),
                    );
                    path.to_owned()
                }
            }
        } else {
            path.to_owned()
        };

        // If the target is a directory, open it directly. Otherwise use /select
        // to highlight the file in its parent folder.
        let mut cmd = crate::sys_cmd("explorer");
        if Path::new(&target).is_dir() {
            cmd.arg(&target);
        } else {
            cmd.arg("/select,").arg(&target);
        }

        cmd.spawn()
            .map_err(|e| CmdError::failed(format!("Failed to launch explorer: {e}")))?;

        crate::log::info("Rust::open_file_location", "explorer launched OK");
        Ok(format!("Opened location for: {target}"))
    }
}

impl Command for OpenFileLocation {
    fn id(&self) -> &'static str {
        "@open-file-location"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// Resolve a Windows shortcut (.lnk) to its target path using `lnk_com`.
fn resolve_shortcut(lnk_path: &str) -> Result<String, String> {
    let link = lnk_com::resolve(Path::new(lnk_path)).map_err(|e| e.to_string())?;
    link.link_target()
        .ok_or_else(|| "shortcut has no target".into())
}
