//! External right-click monitor.
//!
//! Listens on the `rcm_com` named pipe for events raised by the shell extension.
//! Filtering lives in [`rcm_core::monitor`] and menu evaluation in `rcm-vm`, so
//! this module is only the Tauri-specific plumbing that forwards a right-click
//! into the shared `MenuManager`.

use rcm_core::ui::Point;
use rcm_core::{log, monitor};

use crate::layout::MenuManager;

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
            match monitor::classify(&event) {
                monitor::Action::Evaluate => {
                    let menu = match rcm_vm::from_info(&event) {
                        Ok(menu) => menu,
                        Err(e) => {
                            // Display, not Debug: a JS exception is only legible as
                            // "Error: <message>\n<stack>"; Debug escapes the newlines.
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
