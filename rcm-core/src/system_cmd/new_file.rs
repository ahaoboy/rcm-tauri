//! `@new-file` — Create a new empty file.
//!
//! If the first argument starts with `.` (e.g. `.txt`), it is treated
//! as a file extension and the base name defaults to `New File`.
//! Otherwise the argument is used as the full file name.  The final
//! path is resolved relative to `cmd.cwd` with collision avoidance.

use crate::types::CommandPayload;
use super::{SystemCmdResult, unique_path};

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let base = cmd.args.first().map(|s| s.as_str()).unwrap_or("New File");

    let (name, ext) = if let Some(stripped) = base.strip_prefix('.') {
        ("New File", stripped)
    } else {
        let p = std::path::Path::new(base);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("New File");
        let e = p.extension().and_then(|s| s.to_str()).unwrap_or("");
        (stem, e)
    };

    let filename = if ext.is_empty() { name.to_string() } else { format!("{name}.{ext}") };
    let dir = if cmd.cwd.is_empty() { "." } else { &cmd.cwd };
    let path = unique_path(&std::path::Path::new(dir).join(&filename));

    match std::fs::File::create(&path) {
        Ok(_) => SystemCmdResult {
            success: true,
            message: format!("Created: {}", path.display()),
        },
        Err(e) => SystemCmdResult {
            success: false,
            message: format!("Create file failed: {e}"),
        },
    }
}
