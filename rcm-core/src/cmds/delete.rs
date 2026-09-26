//! `@delete` — Fast, permanent deletion of multiple files/folders.
//!
//! Unlike `@trash` (which sends to the recycle bin), `@delete` removes paths
//! permanently. Directories are removed with the [`remove_dir_all`] crate, which
//! tolerates the non-atomic deletes and readonly files seen on Windows (and can
//! parallelise deletion of a single tree via its `parallel` feature). Files are
//! removed with `std::fs::remove_file`. Every path is removed in its own scoped
//! thread for maximum throughput on large selections (e.g. several
//! `node_modules` folders).

use std::path::Path;
use std::thread;

use remove_dir_all::remove_dir_all;

use super::{CmdArgs, CmdError, Command};
use crate::types::CommandPayload;

/// `@delete` — permanently remove the given files and folders.
pub struct Delete;

/// Arguments for [`Delete`].
struct Args {
    /// Paths to remove permanently.
    paths: Vec<String>,
}

impl Delete {
    /// Extract the paths to delete.
    fn args(payload: &CommandPayload) -> Result<Args, CmdError> {
        Ok(Args {
            paths: CmdArgs::of(payload)
                .all("path", "one or more file or folder paths")?
                .to_vec(),
        })
    }

    /// Permanently remove every path, in parallel.
    fn execute(args: Args) -> Result<String, CmdError> {
        let paths = args.paths;

        // Scoped threads borrow the paths instead of cloning them into every
        // closure, and are guaranteed to be joined before the scope ends.
        let (deleted, errors) = thread::scope(|scope| {
            let handles: Vec<_> = paths
                .iter()
                .map(|path| scope.spawn(move || (path, remove(path))))
                .collect();

            let mut deleted = 0usize;
            let mut errors: Vec<String> = Vec::new();
            for handle in handles {
                match handle.join() {
                    Ok((_, Ok(()))) => deleted += 1,
                    Ok((path, Err(e))) => errors.push(format!("{path}: {e}")),
                    Err(_) => errors.push("deletion thread panicked".into()),
                }
            }
            (deleted, errors)
        });

        if errors.is_empty() {
            return Ok(format!("Deleted {deleted} item(s)"));
        }

        let total = deleted + errors.len();
        Err(CmdError::failed(format!(
            "Deleted {deleted}/{total} item(s). Errors: {}",
            errors.join("; ")
        )))
    }
}

impl Command for Delete {
    fn id(&self) -> &'static str {
        "@delete"
    }

    fn run(&self, payload: &CommandPayload) -> Result<String, CmdError> {
        Self::execute(Self::args(payload)?)
    }
}

/// Remove one path, choosing the right call for its kind.
fn remove(path: &str) -> std::io::Result<()> {
    let path = Path::new(path);
    if path.is_dir() {
        remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}
