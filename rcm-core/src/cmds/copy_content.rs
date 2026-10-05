//! `@copy-content` — Copy the text content of selected file(s) to the
//! clipboard, one file per line-separated block.
//!
//! Unlike `@copy-base64`, which encodes any file, this copies the bytes as text,
//! so only files that *are* UTF-8 text are accepted. Directories, binary files
//! and oversized files are skipped rather than copied as garbage.

use std::fs;
use std::path::Path;

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// Maximum file size to read (10 MB), matching `@copy-base64`.
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// `@copy-content` — put the selected files' text on the clipboard.
pub struct CopyContent;

/// Arguments for [`CopyContent`].
struct Args {
    /// Files to read.
    paths: Vec<String>,
}

impl CopyContent {
    /// Extract the file list.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            paths: CmdArgs::of(payload)
                .all("path", "one or more file paths")?
                .to_vec(),
        })
    }

    /// Read each file and put the joined text on the clipboard.
    fn execute(args: Args) -> Result<String, CmdError> {
        let mut contents: Vec<String> = Vec::with_capacity(args.paths.len());
        let mut skipped = 0usize;

        for path in &args.paths {
            match read_text(Path::new(path)) {
                Some(text) => contents.push(text),
                None => skipped += 1,
            }
        }

        if contents.is_empty() {
            return Err(CmdError::failed(if skipped > 0 {
                format!("No text content read ({skipped} skipped)")
            } else {
                "No files to read".into()
            }));
        }

        let mut message = format!("Copied content of {} file(s)", contents.len());
        if skipped > 0 {
            message.push_str(&format!(" ({skipped} skipped)"));
        }

        crate::clipboard::write_text(&contents.join("\n")).map_err(CmdError::failed)?;
        Ok(message)
    }
}

impl Command for CopyContent {
    fn id(&self) -> &'static str {
        "@copy-content"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// Read `path` as UTF-8 text, or `None` when it is a directory, too large,
/// unreadable, or not valid UTF-8.
///
/// An empty file yields `Some("")`: empty content is a legitimate result, and
/// treating it as a failure would make a selected empty file look broken.
fn read_text(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.is_dir() || metadata.len() > MAX_FILE_SIZE {
        return None;
    }
    String::from_utf8(fs::read(path).ok()?).ok()
}
