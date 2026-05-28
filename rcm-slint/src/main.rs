// Prevent console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod cmd;
mod menu_manager;
mod pipe;
mod tray;

use menu_manager::MenuManager;
use rcm_core::{config, log, FileInfo, InvokeProps};
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════════════════
// Cross-thread event channel
// ═══════════════════════════════════════════════════════════════════════════

pub enum AppEvent {
    ShowRoot { menu: rcm_core::Menu, x: f64, y: f64 },
    HideAll,
    IconsChanged(bool),
    SubmenuRequested { depth: usize, path: Vec<i32> },
    ItemExecuted { depth: usize, cmd: rcm_core::CommandPayload },
    WindowBlurred { depth: usize },
    TrayMenuEvent(tray_icon::menu::MenuEvent),
}

/// Global sender for cross-thread events. Background threads send here;
/// the Slint event loop timer processes them on the main thread.
static EVENT_SENDER: Mutex<Option<std::sync::mpsc::Sender<AppEvent>>> = Mutex::new(None);

fn send_event(event: AppEvent) {
    if let Some(tx) = EVENT_SENDER.lock().unwrap().as_ref() {
        let _ = tx.send(event);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Menu builders
// ═══════════════════════════════════════════════════════════════════════════

pub fn rcm() -> Result<rcm_core::Menu, Box<dyn std::error::Error>> {
    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());
    rcm_vm::invoke(&InvokeProps {
        files: vec![],
        cwd: "C:\\".to_string(),
        env,
        admin: false,
        type_name: "Desktop".to_string(),
        lang: rcm_core::lang::system_lang(),
    })
}

pub fn rcm_from_info(
    info: &rcm_com::ContextMenuInfo,
) -> Result<rcm_core::Menu, Box<dyn std::error::Error>> {
    let files: Vec<FileInfo> = info
        .files
        .iter()
        .map(|path| {
            let p = std::path::Path::new(path);
            FileInfo {
                name: p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                path: path.clone(),
                is_dir: p.is_dir(),
            }
        })
        .collect();
    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());
    rcm_vm::invoke(&InvokeProps {
        files,
        cwd: info.dir.clone(),
        env,
        admin: false,
        type_name: if info.bg { "Background" } else { "File" }.to_string(),
        lang: rcm_core::lang::system_lang(),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Background event handlers
// ═══════════════════════════════════════════════════════════════════════════

fn start_monitoring() {
    log::info("monitor", "begin listening for rcm_com events");
    tokio::spawn(async move {
        if let Err(e) = rcm_com::server::listen(move |event| {
            let is_menu = matches!(&event.event, rcm_com::Event::Menu { .. });
            if is_menu {
                match rcm_from_info(&event) {
                    Ok(menu) => send_event(AppEvent::ShowRoot {
                        menu,
                        x: event.x as f64,
                        y: event.y as f64,
                    }),
                    Err(e) => log::error("monitor", &format!("rcm error: {e:?}")),
                }
            } else if !config::is_dev() {
                send_event(AppEvent::HideAll);
            }
        })
        .await
        {
            log::error("monitor", &format!("ERROR: {e}"));
        }
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// Main entry point
// ═══════════════════════════════════════════════════════════════════════════

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if pipe::check_client_cli() {
        return Ok(());
    }
    config::init();

    // Channel for cross-thread events
    let (tx, rx) = std::sync::mpsc::channel::<AppEvent>();
    *EVENT_SENDER.lock().unwrap() = Some(tx);

    // Shared menu state
    let menu: menu_manager::MenuArc = Arc::new(Mutex::new(None));

    // Create MenuManager (lives on the main thread only)
    let mgr = MenuManager::new(menu.clone());

    // Set up system tray
    let on_icons_changed: tray::IconChangeCallback = Box::new(move |show_icons| {
        send_event(AppEvent::IconsChanged(show_icons));
    });
    let on_icons_ptr: &'static tray::IconChangeCallback =
        Box::leak(Box::new(on_icons_changed));
    let _tray = tray::setup_tray(Box::new(|v| send_event(AppEvent::IconsChanged(v))))?;
    println!("[main] System tray created");

    // Start pipe server
    pipe::start_pipe_server(Arc::new(move |x, y| {
        match rcm() {
            Ok(menu) => send_event(AppEvent::ShowRoot { menu, x, y }),
            Err(e) => log::error("pipe", &format!("rcm error: {e:?}")),
        }
    }));

    // Start rcm_com monitoring
    start_monitoring();

    // Process cross-thread events on the Slint event loop using a timer
    let mgr_rc = mgr.clone();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(50),
        move || {
            // Process tray menu events
            // (tray events are polled via a static receiver)
            tray::process_events(&Box::new(|v| send_event(AppEvent::IconsChanged(v))));

            // Process app events from channel
            while let Ok(event) = rx.try_recv() {
                let mgr = mgr_rc.borrow();
                match event {
                    AppEvent::ShowRoot { menu, x, y } => mgr.show_root(menu, x, y),
                    AppEvent::HideAll => mgr.hide_all(),
                    AppEvent::IconsChanged(show) => {
                        mgr.root_window.set_show_icons(show);
                        for sub in &mgr.submenus {
                            if let Some(win) = sub {
                                win.set_show_icons(show);
                            }
                        }
                    }
                    AppEvent::SubmenuRequested { depth, path } => {
                        // Need window position — get from the root window
                        let pos = mgr.root_window.window().position();
                        let size = mgr.root_window.window().size();
                        mgr.handle_submenu_request(
                            depth,
                            &path,
                            pos.x as f64,
                            pos.y as f64,
                            size.width as f64,
                        );
                    }
                    AppEvent::ItemExecuted { depth: _, cmd } => {
                        mgr.handle_execute(cmd);
                    }
                    AppEvent::WindowBlurred { depth } => {
                        mgr.handle_blur(depth);
                    }
                }
            }
        },
    );

    println!("[main] Entering Slint event loop...");
    slint::run_event_loop()?;

    // Keep timer alive
    drop(timer);
    drop(mgr);

    Ok(())
}
