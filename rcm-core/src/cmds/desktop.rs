//! `@add-to-desktop` / `@remove-from-desktop` — desktop shortcut add/remove,
//! mimicking Explorer's *"Send to > Desktop (create shortcut)"*.
//!
//! Uses [`desktop_com`]: `add` creates the `.lnk` via Shell COM (duplicate
//! names auto-renamed `a.lnk` → `a(1).lnk` by upath); `remove` deletes every
//! desktop shortcut pointing at the target (user + machine scope).

use super::SystemCmdResult;
use crate::types::CommandPayload;
use desktop_com::Scope;
use std::path::Path;

const TAG: &str = "desktop";

fn ok(msg: impl Into<String>) -> SystemCmdResult {
    SystemCmdResult {
        success: true,
        message: msg.into(),
    }
}

fn fail(msg: impl Into<String>) -> SystemCmdResult {
    SystemCmdResult {
        success: false,
        message: msg.into(),
    }
}

/// First non-empty arg, or an error result.
fn first_arg(cmd: &CommandPayload) -> Result<&str, SystemCmdResult> {
    match cmd.args.first() {
        Some(p) if !p.is_empty() => Ok(p),
        _ => Err(fail("No file specified")),
    }
}

/// Desktop `.lnk` paths (user + public) — for frontend add/remove matching.
pub fn list() -> Vec<String> {
    match desktop_com::list_shortcuts() {
        Ok(items) => items
            .into_iter()
            .map(|it| it.path.to_string_lossy().into_owned())
            .collect(),
        Err(e) => {
            crate::log::error(TAG, &format!("list failed: {e}"));
            Vec::new()
        }
    }
}

/// `@add-to-desktop` — desktop shortcut to the arg target.
pub fn add(cmd: &CommandPayload) -> SystemCmdResult {
    let p = match first_arg(cmd) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let target = Path::new(p);
    let name = match target.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return fail(format!("Bad file name: '{p}'")),
    };
    crate::log::info(TAG, &format!("add '{p}'"));

    match desktop_com::add(Scope::User, name, target, None) {
        Ok(lnk) => ok(format!("Desktop shortcut: {}", lnk.display())),
        Err(e) => {
            crate::log::error(TAG, &e.to_string());
            fail(e.to_string())
        }
    }
}

/// `@remove-from-desktop` — remove every desktop shortcut to the arg target.
pub fn remove(cmd: &CommandPayload) -> SystemCmdResult {
    let p = match first_arg(cmd) {
        Ok(p) => p,
        Err(e) => return e,
    };
    crate::log::info(TAG, &format!("remove '{p}'"));

    match desktop_com::remove(Path::new(p)) {
        Ok(list) if list.is_empty() => ok("Not on desktop"),
        Ok(list) => ok(format!("Removed: {}", list.len())),
        Err(e) => {
            crate::log::error(TAG, &e.to_string());
            fail(e.to_string())
        }
    }
}
