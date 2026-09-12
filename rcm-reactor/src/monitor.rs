//! External right-click monitor.
//!
//! Listens on the `rcm_com` named pipe for events raised by the shell
//! extension, then hands them to the UI thread. Filtering lives in the shared
//! [`rcm_core::config::ignore_reason`], and menu evaluation in `rcm-vm`, so this
//! module is only the Reactor-specific plumbing.

use rcm_core::ui::Point;
use rcm_core::{log, monitor};

use crate::events::{AppEvent, push};

/// Start listening for external right-click events. Runs on a dedicated thread
/// and never returns.
pub fn start() {
    log::info("Rust::monitor", "begin listening for rcm_com events");

    let spawned = std::thread::Builder::new()
        .name("rcm-monitor".to_string())
        .spawn(|| {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    log::error("Rust::monitor", &format!("tokio runtime failed: {e}"));
                    return;
                }
            };

            rt.block_on(async {
                if let Err(e) = rcm_com::server::listen(handle_event).await {
                    log::error("Rust::monitor", &format!("ERROR: {e}"));
                }
            });
        });

    if let Err(e) = spawned {
        log::error(
            "Rust::monitor",
            &format!("failed to spawn monitor thread: {e}"),
        );
    }
}

fn handle_event(event: rcm_com::ContextMenuInfo) {
    match rcm_core::monitor::classify(&event) {
        monitor::Action::Evaluate => {
            let menu = match rcm_vm::from_info(&event) {
                Ok(menu) => menu,
                Err(e) => {
                    log::error("Rust::monitor", &format!("rcm error: {e:?}"));
                    return;
                }
            };
            push(AppEvent::ShowMenu {
                menu: std::sync::Arc::new(menu),
                at: Point::new(event.x, event.y),
            });
        }
        monitor::Action::HideAll => push(AppEvent::HideAll),
        monitor::Action::Ignore => {}
    }
}
