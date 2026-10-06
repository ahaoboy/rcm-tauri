//! External right-click monitor.
//!
//! Listens on the `rcm_com` named pipe for events raised by the shell extension.
//! Filtering lives in [`rcm_core::monitor`] and menu evaluation in `rcm-vm`, so
//! this module is only the Tauri-specific plumbing that forwards a right-click
//! into the shared `MenuManager`.

use std::sync::atomic::{AtomicBool, Ordering};

use rcm_core::ui::Point;
use rcm_core::{log, monitor};

use crate::layout::MenuManager;

/// Whether the shell extension has been heard from.
///
/// Set only when an event arrives off the event pipe, because an event is the
/// one piece of evidence that cannot be misleading: this process *hosts* that
/// pipe and the extension connects to it, so anything read from it was sent by a
/// loaded extension. The extension also hosts a control pipe, but whether it
/// answers control requests is a narrower question than whether right-clicks
/// reach us, and the two can differ.
///
/// Never cleared. Losing the extension is not observable from here — a
/// connection is opened per event and closed immediately, and the absence of
/// events means nothing when right-clicks are sporadic — so the flag only ever
/// means "it has worked". A failure to build the menu from an event is a
/// different kind of problem; see `start_monitoring`.
static HEARD_FROM_EXTENSION: AtomicBool = AtomicBool::new(false);

/// Whether the shell extension has been heard from yet.
///
/// Polled by the shell-extension diagnostic window.
pub fn connected() -> bool {
    HEARD_FROM_EXTENSION.load(Ordering::Relaxed)
}

/// Start listening for external right-click events from the rcm_com pipe.
/// This runs in a background task and never returns.
pub fn start_monitoring(manager: MenuManager) {
    // Build the menu runtime now rather than during the first right-click: for
    // the `deno` engine that build dominates the click-to-menu latency. It runs
    // on the blocking pool, so it neither blocks this setup nor delays listening.
    tauri::async_runtime::spawn_blocking(rcm_vm::init);

    log::info("Rust::monitor", "begin listening for rcm_com events");
    tauri::async_runtime::spawn(async move {
        if let Err(e) = rcm_com::server::listen(move |event| {
            // Any event counts, even one that classifies as `Ignore`: getting a
            // message to us at all is what says the extension is loaded.
            HEARD_FROM_EXTENSION.store(true, Ordering::Relaxed);
            match monitor::classify(&event) {
                monitor::Action::Evaluate => {
                    let menu = match rcm_vm::from_info(&event) {
                        Ok(menu) => menu,
                        Err(e) => {
                            // Display, not Debug: a JS exception is only legible as
                            // "Error: <message>\n<stack>"; Debug escapes the newlines.
                            //
                            // The extension itself is fine — it delivered the event,
                            // so it stays "heard from". What failed is the menu, which
                            // is the user's script; the log is its only outlet today.
                            log::error("Rust::monitor", &format!("rcm error: {e}"));
                            return;
                        }
                    };
                    manager.show_root(menu, Point::new(event.x, event.y));
                }
                monitor::Action::HideAll => manager.hide_all(),
                monitor::Action::Ignore => {}
            }
        })
        .await
        {
            log::error("Rust::monitor", &format!("rcm_com listener stopped: {e}"));
        }
    });
}
