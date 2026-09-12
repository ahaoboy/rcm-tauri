// ═══════════════════════════════════════════════════════════════════════════
// Monitor — listens for external right-click events from rcm_com.
//
// Filtering lives in `rcm_core::config::ignore_reason` and menu evaluation in
// `rcm-vm`, so this module is only the Tauri plumbing that forwards a
// right-click into the shared `MenuManager`.
// ═══════════════════════════════════════════════════════════════════════════

use rcm_core::ui::Point;
use rcm_core::{config, log};

use crate::layout::MenuManager;

/// Start listening for external right-click events from the rcm_com pipe.
/// This runs in a background task and never returns.
pub fn start_monitoring(manager: MenuManager) {
    log::info("Rust::monitor", "begin listening for rcm_com events");
    tauri::async_runtime::spawn(async move {
        if let Err(e) = rcm_com::server::listen(move |event| {
            log::event(
                "RECV",
                "rcm_com",
                &format!("{:?} pos=({},{})", event.event, event.x, event.y),
            );

            if let Some(reason) = config::ignore_reason(&event) {
                log::info("Rust::monitor", &format!("filtered: {reason}"));
                return;
            }

            match &event.event {
                rcm_com::Event::Menu { .. } => {
                    // When blocking is disabled, the native system menu is
                    // already showing. Do NOT open the custom menu, and hide any
                    // that are open — otherwise system + custom menus would
                    // appear at the same time.
                    if !rcm_core::ui::is_blocking_enabled() {
                        log::info(
                            "Rust::monitor",
                            "blocking disabled — suppressing custom menu (native menu shown)",
                        );
                        manager.hide_all();
                        return;
                    }

                    let menu_data = match rcm_vm::from_info(&event) {
                        Ok(menu_data) => menu_data,
                        Err(e) => {
                            log::error("Rust::monitor", &format!("rcm error: {e:?}"));
                            return;
                        }
                    };

                    manager.show_root(menu_data, Point::new(event.x, event.y));
                }
                _ => {
                    log::info(
                        "Rust::monitor",
                        &format!("non-Menu event (dev={})", config::is_dev()),
                    );
                    if !config::is_dev() {
                        manager.hide_all();
                    }
                }
            }
        })
        .await
        {
            log::error("Rust::monitor", &format!("ERROR: {e}"));
        }
    });
}
