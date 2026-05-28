//! RCM egui — entry point.
//!
//! Starts the egui/eframe application with a small transparent window
//! that can display the right-click context menu, along with system tray support.
//! Prevents a console window from appearing on Windows in release builds.

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rcm_core::config;

// ═══════════════════════════════════════════════════════════════════════════
// CLI check — if invoked as a client, forward and exit before opening GUI
// ═══════════════════════════════════════════════════════════════════════════

/// Check if this process was invoked as a CLI client sending coordinates
/// to the running instance. If so, handle it and return early.
fn check_cli() -> bool {
    rcm_egui::pipe::check_client_cli()
}

// ═══════════════════════════════════════════════════════════════════════════
// Native (Windows/Linux/macOS) entry point
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    // Log to stderr (use RUST_LOG=debug for verbose output)
    env_logger::init();

    // If invoked as CLI client, forward coordinates to running instance
    if check_cli() {
        return Ok(());
    }

    // Initialize configuration (must happen before tray setup)
    config::init();

    // Build egui viewport configuration
    // The window is:
    // - Small and transparent (like Tauri's root menu window)
    // - Always on top
    // - Without decorations (no title bar)
    // - Hidden from taskbar
    let viewport = egui::ViewportBuilder::default()
        .with_title("RCM — Right Click Menu")
        .with_inner_size([1.0, 1.0])
        .with_min_inner_size([1.0, 1.0])
        .with_position([-9999.0, -9999.0]) // Start off-screen
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_taskbar(false)
        .with_resizable(false)
        .with_maximized(false)
        .with_window_level(egui::viewport::WindowLevel::AlwaysOnTop);

    let native_options = eframe::NativeOptions {
        viewport,
        // Don't persist window position/size
        persist_window: false,
        // Use glow renderer (more stable on Windows with transparent windows)
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "RCM",
        native_options,
        Box::new(|cc| Ok(Box::new(rcm_egui::RcmEguiApp::new(cc)))),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Web (wasm32) entry point — included for compatibility, not the primary target
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element was not a canvas");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(rcm_egui::RcmEguiApp::new(cc)))),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => loading_text.remove(),
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p>The app has crashed. See console for details.</p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

