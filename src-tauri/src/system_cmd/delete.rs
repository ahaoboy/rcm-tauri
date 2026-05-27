//! `@delete` — Fast parallel deletion of multiple files/folders.
//!
//! Unlike `@trash` (which sends to recycle bin), `@delete` performs
//! permanent deletion using `std::fs::remove_dir_all` / `remove_file`.
//! Each path is deleted in its own thread for maximum throughput on
//! large selections (e.g. multiple `node_modules` folders).

use crate::rcm::CommandPayload;
use super::SystemCmdResult;
use std::thread;

pub fn run(cmd: &CommandPayload) -> SystemCmdResult {
    let paths: Vec<&str> = cmd.args.iter().map(|s| s.as_str()).collect();
    if paths.is_empty() {
        return SystemCmdResult { success: false, message: "No files specified".into() };
    }

    // Spawn one thread per path for parallel deletion
    let handles: Vec<_> = paths
        .iter()
        .map(|&path| {
            let owned = path.to_owned();
            thread::spawn(move || {
                let p = std::path::Path::new(&owned);
                let result = if p.is_dir() {
                    std::fs::remove_dir_all(&owned)
                } else {
                    std::fs::remove_file(&owned)
                };
                (owned, result)
            })
        })
        .collect();

    let mut errors: Vec<String> = Vec::new();
    let mut ok = 0usize;
    for h in handles {
        match h.join().unwrap() {
            (_, Ok(())) => ok += 1,
            (path, Err(e)) => errors.push(format!("{path}: {e}")),
        }
    }

    if errors.is_empty() {
        SystemCmdResult {
            success: true,
            message: format!("Deleted {ok} item(s)"),
        }
    } else {
        SystemCmdResult {
            success: false,
            message: format!(
                "Deleted {ok}/{} item(s). Errors: {}",
                ok + errors.len(),
                errors.join("; "),
            ),
        }
    }
}
