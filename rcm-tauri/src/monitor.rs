// ═══════════════════════════════════════════════════════════════════════════
// Monitor — listens for external right-click events from rcm_com.
// ═══════════════════════════════════════════════════════════════════════════

use crate::events::{AutoHideEpoch, MenuArc};
use crate::layout::MenuManager;
use rcm_core::{config, log};

/// Check whether an event should be ignored (not trigger the context menu).
/// Returns `Some(reason)` if the event should be skipped, `None` otherwise.
/// Filters are read from `rcm.config.json` at startup.
fn should_ignore(event: &rcm_com::ContextMenuInfo) -> Option<String> {
    for rule in rcm_core::config::filters() {
        if rule.matches(event) {
            let reason = if rule.reason.is_empty() {
                format!(
                    "class_re={:?} file_eq={:?} flags_eq={:?}",
                    rule.class, rule.file, rule.flags
                )
            } else {
                format!("{} (hwnd={})", rule.reason, event.hwnd)
            };
            return Some(reason);
        }
    }
    None
}

/// Start listening for external right-click events from the rcm_com pipe.
/// This runs in a background task and never returns.
pub fn start_monitoring(app_handle: tauri::AppHandle, menu: MenuArc, epoch: AutoHideEpoch) {
    println!(
        "Rust::monitor: starting rcm_com listener (dev={})",
        config::is_dev()
    );
    log::info("Rust::monitor", "begin listening for rcm_com events");
    tauri::async_runtime::spawn(async move {
        if let Err(e) = rcm_com::server::listen(move |event| {
            log::event(
                "RECV",
                "rcm_com",
                &format!("{:?} pos=({},{})", event.event, event.x, event.y),
            );
            println!("{:?}", event);

            if let Some(reason) = should_ignore(&event) {
                log::info("Rust::monitor", &format!("filtered: {reason}"));
                return;
            }

            match &event.event {
                rcm_com::Event::Menu { .. } => {
                    // When blocking is disabled, the native system menu is
                    // already showing. Do NOT open the custom menu, and hide
                    // any that are open — otherwise system + custom menus
                    // would appear at the same time.
                    if !crate::is_blocking_enabled() {
                        log::info(
                            "Rust::monitor",
                            "blocking disabled — suppressing custom menu (native menu shown)",
                        );
                        let mgr = MenuManager {
                            menu: menu.clone(),
                            app: app_handle.clone(),
                            auto_hide_epoch: epoch.clone(),
                        };
                        mgr.hide_all();
                        return;
                    }

                    let menu_data = match rcm_vm::from_info(&event) {
                        Ok(m) => m,
                        Err(e) => {
                            log::error("Rust::monitor", &format!("rcm error: {:?}", e));
                            return;
                        }
                    };

                    let mgr = MenuManager {
                        menu: menu.clone(),
                        app: app_handle.clone(),
                        auto_hide_epoch: epoch.clone(),
                    };

                    mgr.show_root(menu_data, event.x as f64, event.y as f64);
                }
                _ => {
                    log::info(
                        "Rust::monitor",
                        &format!("non-Menu event (dev={})", config::is_dev()),
                    );
                    if !config::is_dev() {
                        let mgr = MenuManager {
                            menu: menu.clone(),
                            app: app_handle.clone(),
                            auto_hide_epoch: epoch.clone(),
                        };
                        mgr.hide_all();
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
