// ═══════════════════════════════════════════════════════════════════════════
// Monitor — listens for external right-click events from rcm_com.
// ═══════════════════════════════════════════════════════════════════════════

use crate::events::{AutoHideEpoch, MenuArc};
use crate::menu_builder;
use crate::menu_manager::MenuManager;
use rcm_core::{config, log};

/// Start listening for external right-click events from the rcm_com pipe.
/// This runs in a background task and never returns.
pub fn start_monitoring(app_handle: tauri::AppHandle, menu: MenuArc, epoch: AutoHideEpoch) {
    log::info("Rust::monitor", "begin listening for rcm_com events");
    tauri::async_runtime::spawn(async move {
        if let Err(e) = rcm_com::server::listen(move |event| {
            log::event(
                "RECV",
                "rcm_com",
                &format!("{:?} pos=({},{})", event.event, event.x, event.y),
            );
            println!("{:?}", event);

            // Filter: ignore "Open With" dialog activation events.
            // When PowerShell invokes InvokeVerb('openas'), the "Open With"
            // picker window triggers a spurious Menu{flags:16} event from a
            // Chrome_WidgetWin_0 host window.
            if event.class.starts_with("Chrome_WidgetWin_") && event.event.flags() == 16 {
                log::info(
                    "Rust::monitor",
                    "filtered: OpenWith dialog (Chrome_WidgetWin_0, flags=16)",
                );
                return;
            }

            match &event.event {
                rcm_com::Event::Menu { .. } => {
                    let menu_data = match menu_builder::rcm_from_info(&event) {
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
