use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::window::Color;
use tauri::{Emitter, Listener, Manager, PhysicalPosition, WebviewUrl};
use serde::{Deserialize, Serialize};
use rcm_core::{CommandPayload, FileInfo, InvokeProps, Menu};
use rcm_core::{config, lang, log};

pub mod cmd;
pub mod pipe;
pub mod tray;

// ═══════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════

/// Maximum submenu depth (0 = root, 1-3 = submenus).
const MAX_SUBMENU_DEPTH: usize = 4;

/// Off-screen position for hidden windows.
const OFF_SCREEN: PhysicalPosition<f64> = PhysicalPosition { x: -9999.0, y: -9999.0 };

/// Tracks the deepest menu depth currently visible.
/// Used to decide whether a blur event should hide all menus
/// (only if the deepest window lost focus).
static DEEPEST_DEPTH: AtomicUsize = AtomicUsize::new(0);

// ═══════════════════════════════════════════════════════════════════════════
// Config payload (for frontend)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
struct ConfigPayload {
    dev: bool,
    icons: bool,
    menu: &'static str,
}

#[tauri::command]
fn get_config() -> ConfigPayload {
    ConfigPayload {
        dev: config::is_dev(),
        icons: config::is_icons(),
        menu: if config::is_lite() { "lite" } else { "full" },
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared state
// ═══════════════════════════════════════════════════════════════════════════

/// Holds the last built menu so hover/click handlers can navigate it.
type MenuArc = Arc<Mutex<Option<Menu>>>;

// ═══════════════════════════════════════════════════════════════════════════
// Event payloads — Rust → Frontend
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
struct MenuShowPayload {
    /// Full menu data — every window gets the complete tree.
    menu: Menu,
    /// Index path to render. Empty `[]` = root.
    path: Vec<i32>,
    /// Absolute screen position for the window.
    x: f64,
    y: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// Event payloads — Frontend → Rust
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct MenuHoverPayload {
    /// Depth of the emitting window (0 = root).
    depth: usize,
    /// Index path to the hovered item.
    path: Vec<i32>,
    /// Parent window's absolute screen position.
    #[serde(rename = "parentX")]
    parent_x: f64,
    #[serde(rename = "parentY")]
    parent_y: f64,
    /// Parent window's size.
    #[serde(rename = "parentW")]
    parent_w: f64,
    #[serde(rename = "parentH")]
    parent_h: f64,
    /// Hovered item's position relative to the parent window.
    #[serde(rename = "itemX")]
    item_x: f64,
    #[serde(rename = "itemY")]
    item_y: f64,
    /// Hovered item's size.
    #[serde(rename = "itemW")]
    item_w: f64,
    #[serde(rename = "itemH")]
    item_h: f64,
    /// Absolute screen X of the parent window's content right edge (no shadow).
    #[serde(default)]
    content_right: f64,
    /// Height of the parent's .rcm-root content element (for boundary clamping).
    #[serde(rename = "parentContentHeight", default)]
    parent_content_h: f64,
    /// Width of the parent's .rcm-root content element (for precise X alignment).
    #[serde(rename = "parentContentWidth", default)]
    parent_content_w: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct MenuHoverOutPayload {
    /// Depth of the emitting window.
    depth: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct MenuExecutePayload {
    /// Index path to the clicked item.
    path: Vec<i32>,
    /// Command to execute (sent directly from frontend).
    command: CommandPayload,
}

#[derive(Debug, Clone, Deserialize)]
struct MenuBlurPayload {
    /// Depth of the window that lost focus.
    depth: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// Window helpers
// ═══════════════════════════════════════════════════════════════════════════

fn root_label() -> &'static str { "main" }

fn submenu_label(depth: usize) -> String {
    format!("submenu-{}", depth)
}

fn window_label(depth: usize) -> String {
    if depth == 0 {
        root_label().to_string()
    } else {
        submenu_label(depth - 1)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Menu manager — central hub for all menu window logic
// ═══════════════════════════════════════════════════════════════════════════

struct MenuManager {
    menu: MenuArc,
    app: tauri::AppHandle,
}

impl MenuManager {
    /// Show the root menu at the cursor position.
    fn show_root(&self, menu: Menu, x: f64, y: f64) {
        log::info("Rust::show_root", &format!("pos=({x:.0},{y:.0}) groups={} icons={} max_depth={}",
            menu.groups.len(), menu.icon_items.len(), menu.max_depth()));

        *self.menu.lock().unwrap() = Some(menu.clone());
        DEEPEST_DEPTH.store(0, Ordering::SeqCst);

        // Determine how many submenu windows we need
        let max_depth = menu.max_depth().min(MAX_SUBMENU_DEPTH);
        for d in 0..max_depth {
            let label = submenu_label(d);
            if self.app.get_webview_window(&label).is_none() {
                log::info("Rust::show_root", &format!("creating window '{label}'"));
                self.create_submenu_window(&label);
            }
        }

        // Show root window
        let payload = MenuShowPayload {
            menu,
            path: vec![],
            x,
            y,
        };

        let label = root_label();
        if let Some(win) = self.app.get_webview_window(label) {
            let _ = win.set_position(PhysicalPosition { x, y });
            let _ = win.show();
            log::info("Rust::show_root", &format!("window '{label}' shown"));
        }
        log::event("SEND", "menu-show", &format!("to={label} path=[]"));
        let _ = self.app.emit("menu-show", payload);
    }

    /// Handle hover on a menu item: show submenu if the item has children.
    fn handle_hover(&self, payload: MenuHoverPayload) {
        log::event("RECV", "menu-hover", &format!("depth={} path={:?} parent=({:.0},{:.0})",
            payload.depth, payload.path, payload.parent_x, payload.parent_y));

        let menu_guard = self.menu.lock().unwrap();
        let menu = match menu_guard.as_ref() {
            Some(m) => m,
            None => {
                log::warn("Rust::handle_hover", "no menu data, ignoring");
                return;
            }
        };

        // Navigate to the hovered item
        let item = match menu.get_item(&payload.path) {
            Some(i) => i,
            None => {
                log::warn("Rust::handle_hover", &format!("item not found at path {:?}", payload.path));
                return;
            }
        };

        log::info("Rust::handle_hover", &format!("item='{}' has_children={} disable={}",
            item.label, item.has_children(), item.disable));

        // If the item is disabled or has no children, hide deeper submenus
        if item.disable || !item.has_children() {
            drop(menu_guard);
            log::info("Rust::handle_hover", &format!("leaf/disabled, hiding deeper than {}", payload.depth));
            self.hide_deeper_than(payload.depth);
            return;
        }

        // Compute child depth
        let child_depth = payload.depth + 1;
        if child_depth > MAX_SUBMENU_DEPTH {
            log::warn("Rust::handle_hover", "max depth reached, ignoring");
            return;
        }

        // ── Horizontal positioning ───────────────────────────────────────
        // Position submenu at parent's .rcm-root right edge + gap.
        // #root padding is identical in both windows, so it cancels out.
        let sub_x = if payload.parent_content_w > 0.0 {
            payload.parent_x + payload.parent_content_w + 8.0
        } else if payload.content_right > 0.0 {
            payload.content_right
        } else {
            // Fallback for old frontends
            payload.parent_x + payload.parent_w - 28.0
        };

        // ── Vertical positioning ─────────────────────────────────────────
        // Frontend sends item_y in physical px (DPI-corrected), so we can
        // position the submenu directly without CSS constant estimates.
        let ideal_y = payload.parent_y + payload.item_y;

        // Safety clamp: don't let the submenu start below the parent window.
        let parent_bottom = payload.parent_y + payload.parent_h;
        let sub_y = if ideal_y > parent_bottom {
            let clamped = parent_bottom;
            log::info("Rust::handle_hover", &format!(
                "pos: ideal_y={:.0} parent_bottom={:.0} → clamped_y={:.0}",
                ideal_y, parent_bottom, clamped
            ));
            clamped
        } else {
            ideal_y
        };

        // ── Debug: log all positioning inputs ───────────────────────────
        log::info("Rust::handle_hover", &format!(
            "pos_debug: parent=({:.0},{:.0}) parent_w={:.0} parent_h={:.0} parentCW={:.0} parentCH={:.0} | \
             item=({:.0},{:.0}) item_w={:.0} item_h={:.0} | \
             children={} | sub=({:.0},{:.0})",
            payload.parent_x, payload.parent_y,
            payload.parent_w, payload.parent_h,
            payload.parent_content_w, payload.parent_content_h,
            payload.item_x, payload.item_y,
            payload.item_w, payload.item_h,
            item.items.len(),
            sub_x, sub_y
        ));
        let child_label = window_label(child_depth);

        log::info("Rust::handle_hover", &format!("showing '{child_label}' at ({sub_x:.0},{sub_y:.0}) depth={child_depth}"));

        // Only hide windows deeper than the child we're about to show
        drop(menu_guard);
        self.hide_deeper_than(child_depth);

        let menu_guard2 = self.menu.lock().unwrap();
        let menu = match menu_guard2.as_ref() {
            Some(m) => m,
            None => return,
        };

        let show_payload = MenuShowPayload {
            menu: menu.clone(),
            path: payload.path.clone(),
            x: sub_x,
            y: sub_y,
        };

        if let Some(win) = self.app.get_webview_window(&child_label) {
            let _ = win.set_position(PhysicalPosition { x: sub_x, y: sub_y });
            let _ = win.show();
        }
        DEEPEST_DEPTH.store(child_depth, Ordering::SeqCst);
        log::info("Rust::handle_hover", &format!("DEEPEST_DEPTH={child_depth}"));
        log::event("SEND", "menu-show", &format!("to={child_label} path={:?}", payload.path));
        let _ = self.app.emit("menu-show", show_payload);
    }

    /// Handle hover-out: hide all windows deeper than this one.
    fn handle_hover_out(&self, payload: MenuHoverOutPayload) {
        // Don't hide on hover-out immediately; let the next hover handle it.
        // Only hide if mouse truly left all menu windows.
        // For simplicity, we use a small delay approach handled in frontend.
        // The hover on a sibling item will trigger hide_deeper_than anyway.
        let _ = payload;
    }

    /// Handle execute: run the command and close all menus.
    fn handle_execute(&self, payload: MenuExecutePayload) {
        log::event("RECV", "menu-execute", &format!("exe='{}' path={:?}", payload.command.exe, payload.path));

        let cmd = payload.command;
        tauri::async_runtime::spawn(async move {
            let result = cmd::execute(cmd).await;
            if !result.success {
                log::error("Rust::handle_execute", &format!("FAILED: {:?}", result));
            } else {
                log::info("Rust::handle_execute", "OK");
            }
        });

        if !config::is_dev() {
            log::info("Rust::handle_execute", &format!("hiding all (dev={})", config::is_dev()));
            self.hide_all();
        }
    }

    /// Handle blur from a menu window.
    fn handle_blur(&self, payload: MenuBlurPayload) {
        let deepest = DEEPEST_DEPTH.load(Ordering::SeqCst);
        log::event("RECV", "menu-blur", &format!("depth={} deepest={}", payload.depth, deepest));

        if payload.depth != deepest {
            log::info("Rust::handle_blur", "IGNORED (depth != deepest)");
            return;
        }
        log::info("Rust::handle_blur", "MATCH → hiding all");
        self.hide_all();
    }

    /// Hide all menu windows.
    fn hide_all(&self) {
        log::info("Rust::hide_all", "START");
        DEEPEST_DEPTH.store(0, Ordering::SeqCst);
        // Hide root
        if let Some(win) = self.app.get_webview_window(root_label()) {
            let _ = win.hide();
            let _ = win.set_position(OFF_SCREEN);
        }
        // Hide submenus
        for d in 0..MAX_SUBMENU_DEPTH {
            let label = submenu_label(d);
            if let Some(win) = self.app.get_webview_window(&label) {
                let _ = win.hide();
                let _ = win.set_position(OFF_SCREEN);
            }
        }
        log::event("SEND", "menu-hide-all", "broadcast");
        let _ = self.app.emit("menu-hide-all", true);
    }

    /// Hide all submenu windows strictly deeper than `depth`,
    /// then update DEEPEST_DEPTH to reflect the new reality.
    fn hide_deeper_than(&self, depth: usize) {
        for d in (depth + 1)..=MAX_SUBMENU_DEPTH {
            let label = window_label(d);
            if let Some(win) = self.app.get_webview_window(&label) {
                let _ = win.hide();
                let _ = win.set_position(OFF_SCREEN);
            }
        }
        // After hiding deeper windows, `depth` is now the deepest visible
        log::info("Rust::hide_deeper_than", &format!("hid >{depth}, DEEPEST_DEPTH→{depth}"));
        DEEPEST_DEPTH.store(depth, Ordering::SeqCst);
    }

    /// Create a transparent submenu window.
    fn create_submenu_window(&self, label: &str) {
        if self.app.get_webview_window(label).is_some() {
            return;
        }

        let url = format!("index.html#{label}");
        let builder = tauri::WebviewWindowBuilder::new(
            &self.app,
            label,
            WebviewUrl::App(url.into()),
        )
        .title("rcm-submenu")
        .decorations(false)
        .background_color(Color(0, 0, 0, 0))
        .position(0., 0.)
        .inner_size(1., 1.)
        .always_on_top(true)
        .skip_taskbar(true)
        .fullscreen(false)
        .visible(false)
        .closable(false)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .focused(false)
        .shadow(false);

        #[cfg(not(target_os = "macos"))]
        let builder = builder.transparent(true);

        builder.build().unwrap();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Monitor — listens for external right-click events from rcm_com
// ═══════════════════════════════════════════════════════════════════════════

fn start_monitoring(app_handle: tauri::AppHandle, menu: MenuArc) {
    log::info("Rust::monitor", "begin listening for rcm_com events");
    tauri::async_runtime::spawn(async move {
        if let Err(e) = rcm_com::server::listen(move |event| {
            log::event("RECV", "rcm_com", &format!("{:?} pos=({},{})", event.event, event.x, event.y));
            println!("{:?}", event);

            // Filter: ignore "Open With" dialog activation events.
            // When PowerShell invokes InvokeVerb('openas'), the "Open With"
            // picker window triggers a spurious Menu{flags:16} event from a
            // Chrome_WidgetWin_0 host window.
            if event.class.starts_with("Chrome_WidgetWin_") && event.event.flags() == 16 {
                log::info("Rust::monitor", "filtered: OpenWith dialog (Chrome_WidgetWin_0, flags=16)");
                return;
            }

            match &event.event {
                rcm_com::Event::Menu { .. } => {
                    let menu_data = match rcm_from_info(&event) {
                        Ok(m) => m,
                        Err(e) => {
                            log::error("Rust::monitor", &format!("rcm error: {:?}", e));
                            return;
                        }
                    };

                    let mgr = MenuManager {
                        menu: menu.clone(),
                        app: app_handle.clone(),
                    };

                    mgr.show_root(menu_data, event.x as f64, event.y as f64);
                }
                _ => {
                    log::info("Rust::monitor", &format!("non-Menu event (dev={})", config::is_dev()));
                    if !config::is_dev() {
                        let mgr = MenuManager {
                            menu: menu.clone(),
                            app: app_handle.clone(),
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

// ═══════════════════════════════════════════════════════════════════════════
// Menu builders — generate menu from context data
// ═══════════════════════════════════════════════════════════════════════════

/// Build a menu from a blank desktop context (no files selected).
pub fn rcm() -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());
    let props = InvokeProps {
        files: vec![],
        cwd: "C:\\".to_string(),
        env,
        admin: false,
        type_name: "Desktop".to_string(),
        lang: lang::system_lang(),
    };

    rcm_vm::invoke(&props)
}

/// Build a menu from real right-click context data received via the pipe.
pub fn rcm_from_info(info: &rcm_com::ContextMenuInfo) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());

    let files: Vec<FileInfo> = info.files.iter().map(|path| {
        let p = std::path::Path::new(path);
        FileInfo {
            name: p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            path: path.clone(),
            is_dir: p.is_dir(),
        }
    }).collect();

    let props = InvokeProps {
        files,
        cwd: info.dir.clone(),
        env,
        admin: false,
        type_name: if info.bg { "Background".to_string() } else { "File".to_string() },
        lang: lang::system_lang(),
    };

    rcm_vm::invoke(&props)
}

// ═══════════════════════════════════════════════════════════════════════════
// Tauri command — create submenu window (called from frontend for lazy init)
// ═══════════════════════════════════════════════════════════════════════════

#[tauri::command]
async fn create_window(app: tauri::AppHandle, label: String) {
    let mgr = MenuManager {
        menu: Arc::new(Mutex::new(None)),
        app,
    };
    mgr.create_submenu_window(&label);
}

// ═══════════════════════════════════════════════════════════════════════════
// Application entry point
// ═══════════════════════════════════════════════════════════════════════════

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if pipe::check_client_cli() {
        return;
    }

    let menu: MenuArc = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .manage(menu.clone())
        .setup(move |app| {
            config::init();
            tray::setup_tray(app)?;
            pipe::start_pipe_server(app.app_handle().clone());

            // Pre-create submenu windows
            for d in 0..MAX_SUBMENU_DEPTH {
                let label = submenu_label(d);
                let mgr = MenuManager {
                    menu: menu.clone(),
                    app: app.app_handle().clone(),
                };
                mgr.create_submenu_window(&label);
            }

            // Register event listeners for frontend → backend communication
            let app_handle = app.app_handle().clone();
            let menu_clone = menu.clone();

            // ── Frontend log bridge ─────────────────────────────────
            app_handle.listen("log-event", move |event| {
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    let tag = payload["tag"].as_str().unwrap_or("FE");
                    let msg = payload["msg"].as_str().unwrap_or("");
                    log::frontend(tag, msg);
                }
            });

            // Listen for hover events
            let ah1 = app_handle.clone();
            let m1 = menu_clone.clone();
            app_handle.listen("menu-hover", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuHoverPayload>(event.payload()) {
                    let mgr = MenuManager { menu: m1.clone(), app: ah1.clone() };
                    mgr.handle_hover(payload);
                }
            });

            // Listen for hover-out events
            let ah2 = app_handle.clone();
            let m2 = menu_clone.clone();
            app_handle.listen("menu-hover-out", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuHoverOutPayload>(event.payload()) {
                    let mgr = MenuManager { menu: m2.clone(), app: ah2.clone() };
                    mgr.handle_hover_out(payload);
                }
            });

            // Listen for execute events
            let ah3 = app_handle.clone();
            let m3 = menu_clone.clone();
            app_handle.listen("menu-execute", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuExecutePayload>(event.payload()) {
                    let mgr = MenuManager { menu: m3.clone(), app: ah3.clone() };
                    mgr.handle_execute(payload);
                }
            });

            // Listen for close-all from frontend (e.g. Escape key)
            let ah4 = app_handle.clone();
            let m4 = menu_clone.clone();
            app_handle.listen("menu-close-all", move |_| {
                if !config::is_dev() {
                    let mgr = MenuManager { menu: m4.clone(), app: ah4.clone() };
                    mgr.hide_all();
                }
            });

            // Listen for blur events — only hide all if deepest window lost focus
            let ah5 = app_handle.clone();
            let m5 = menu_clone.clone();
            app_handle.listen("menu-blur", move |event| {
                if let Ok(payload) = serde_json::from_str::<MenuBlurPayload>(event.payload()) {
                    let mgr = MenuManager { menu: m5.clone(), app: ah5.clone() };
                    mgr.handle_blur(payload);
                }
            });

            // Start the external event monitor
            start_monitoring(app_handle, menu);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_window,
            get_config,
            cmd::execute,
            cmd::spawn_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
