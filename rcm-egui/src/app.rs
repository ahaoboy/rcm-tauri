//! Main egui application — context menu UI and event processing.
//!
//! Renders the right-click context menu using egui's immediate-mode GUI.
//! Handles tray icon events, pipe IPC events, menu navigation, and
//! command execution. Mirrors the Tauri frontend functionality.

use crate::cmd;
use crate::pipe::{self, AppEvent};
use crate::tray;
use egui::{Color32, Stroke, Vec2};
use rcm_core::{CommandPayload, config, Menu, log};

// ═══════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════

/// Maximum submenu nesting depth.
#[allow(dead_code)]
const MAX_DEPTH: usize = 4;

/// Width of a menu item row.
const ITEM_WIDTH: f32 = 240.0;

/// Height of a single menu item row.
const ITEM_HEIGHT: f32 = 28.0;

/// Padding inside the menu panel.
const MENU_PADDING: f32 = 4.0;

/// Width of the icon column (0 if icons are disabled).
const ICON_WIDTH: f32 = 28.0;

/// Separator height.
const SEPARATOR_HEIGHT: f32 = 9.0;

// ═══════════════════════════════════════════════════════════════════════════
// Menu state
// ═══════════════════════════════════════════════════════════════════════════

/// Tracks the current context menu state: position, visibility,
/// navigation path for submenus, and configuration flags.
#[derive(Debug, Clone, Default)]
struct MenuState {
    /// The full menu tree (rebuilt on each right-click event).
    menu: Option<Menu>,
    /// Active navigation path at each depth.
    active_paths: Vec<Vec<i32>>,
    /// Screen position of the root menu.
    root_x: f64,
    root_y: f64,
    /// Whether the menu is currently visible.
    visible: bool,
    /// Whether to show icon column.
    show_icons: bool,
    /// Dev mode (keep menu open on external blur).
    dev_mode: bool,
}

impl MenuState {
    fn new() -> Self {
        Self {
            menu: None,
            active_paths: Vec::new(),
            root_x: 0.0,
            root_y: 0.0,
            visible: false,
            show_icons: config::is_icons(),
            dev_mode: config::is_dev(),
        }
    }

    /// Show the root menu at the given screen coordinates.
    fn show(&mut self, menu: Menu, x: f64, y: f64) {
        log::info("egui", &format!(
            "show_root pos=({:.0},{:.0}) groups={} icons={}",
            x, y,
            menu.groups.len(),
            menu.icon_items.len()
        ));
        self.menu = Some(menu);
        self.active_paths = vec![vec![]];
        self.root_x = x;
        self.root_y = y;
        self.visible = true;
    }

    /// Hide the menu and all submenus.
    fn hide(&mut self) {
        if self.dev_mode {
            log::info("egui", "dev mode: ignoring hide request");
            return;
        }
        log::info("egui", "hide_all menus");
        self.visible = false;
        self.active_paths.clear();
    }

    /// Calculate the pixel size needed for the current menu content.
    fn menu_size(&self) -> Option<Vec2> {
        let menu = self.menu.as_ref()?;
        let icon_items = &menu.icon_items;
        let groups = &menu.groups;

        let icon_row_count = if icon_items.is_empty() { 0 } else { 1 };
        let total_item_count: usize = icon_items.len()
            + groups.iter().map(|g| g.items.len()).sum::<usize>();
        let separator_count = if icon_row_count > 0 && !groups.is_empty() { 1 } else { 0 }
            + groups.len().saturating_sub(1);

        let menu_height = MENU_PADDING * 2.0
            + total_item_count as f32 * ITEM_HEIGHT
            + separator_count as f32 * SEPARATOR_HEIGHT;

        Some(Vec2::new(ITEM_WIDTH + MENU_PADDING * 2.0, menu_height))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RcmEguiApp
// ═══════════════════════════════════════════════════════════════════════════

/// The main egui application state.
/// Owns the tray icon handle (must be kept alive) and menu state.
pub struct RcmEguiApp {
    /// Current context menu state (position, items, visibility).
    menu_state: MenuState,
    /// Tray icon handle — must stay alive for the tray to work.
    #[allow(dead_code)]
    tray_icon: tray_icon::TrayIcon,
    /// Flag set by tray "Quit" event to request application exit.
    should_exit: bool,
}

impl RcmEguiApp {
    /// Create the application, initializing config, tray icon, and IPC server.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize persistent configuration from disk
        config::init();

        // Set up the system tray icon with menu
        let tray_icon = tray::setup_tray()
            .expect("Failed to create system tray icon");

        // Start the named pipe server for CLI coordinate forwarding
        pipe::start_pipe_server();

        // Start listening for shell extension right-click events
        start_monitoring();

        log::info("egui", "application initialized successfully");

        Self {
            menu_state: MenuState::new(),
            tray_icon,
            should_exit: false,
        }
    }
}

/// Build a [`Menu`] from a shell extension context info struct.
fn build_menu_from_info(
    info: &rcm_com::ContextMenuInfo,
) -> Result<Menu, Box<dyn std::error::Error>> {
    let files: Vec<rcm_core::FileInfo> = info
        .files
        .iter()
        .map(|path| {
            let p = std::path::Path::new(path);
            rcm_core::FileInfo {
                name: p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                path: path.clone(),
                is_dir: p.is_dir(),
            }
        })
        .collect();

    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());

    rcm_vm::invoke(&rcm_core::InvokeProps {
        files,
        cwd: info.dir.clone(),
        env,
        admin: false,
        type_name: if info.bg { "Background" } else { "File" }.to_string(),
        lang: rcm_core::lang::system_lang(),
    })
}

impl RcmEguiApp {
    fn process_background_events(&mut self) {
        // Process tray menu events (returns true if Quit was selected)
        if tray::process_events() {
            self.should_exit = true;
            return;
        }

        // Process events from rcm_com::server::listen and pipe IPC
        if let Some(rx) = pipe::EVENT_RECEIVER.lock().unwrap().as_ref() {
            while let Ok(event) = rx.try_recv() {
                match event {
                    AppEvent::ShowMenuWithInfo { x, y, info } => {
                        log::info("egui", &format!(
                            "menu from rcm_com at ({:.0},{:.0}) files={} bg={}",
                            x, y, info.files.len(), info.bg
                        ));
                        match build_menu_from_info(&info) {
                            Ok(menu) => self.menu_state.show(menu, x, y),
                            Err(e) => log::error("egui", &format!(
                                "failed to build menu: {e:?}"
                            )),
                        }
                    }
                    AppEvent::ShowContextMenu { x, y } => {
                        log::info("egui", &format!(
                            "pipe context menu at ({:.0},{:.0})", x, y
                        ));
                        // Fallback: pipe-based event with minimal context
                        let info = rcm_com::ContextMenuInfo {
                            cid: String::new(),
                            ts: String::new(),
                            x: x as i32,
                            y: y as i32,
                            dir: String::new(),
                            files: vec![],
                            bg: true,
                            hwnd: 0,
                            class: String::new(),
                            pid: 0,
                            event: rcm_com::Event::default(),
                        };
                        match build_menu_from_info(&info) {
                            Ok(menu) => self.menu_state.show(menu, x, y),
                            Err(e) => log::error("egui", &format!(
                                "failed to build menu: {e:?}"
                            )),
                        }
                    }
                    AppEvent::HideAll => {
                        self.menu_state.hide();
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Shell extension event monitor — rcm_com::server::listen
// ═══════════════════════════════════════════════════════════════════════════

/// Start listening for right-click events from the shell extension.
/// Runs in a dedicated OS thread with its own Tokio runtime.
/// Forwards events to the main egui thread via the crossbeam channel.
fn start_monitoring() {
    log::info("egui", "begin listening for rcm_com events");

    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime for rcm_com monitor");
        rt.block_on(async move {
            if let Err(e) = rcm_com::server::listen(move |event| {
                let is_menu = matches!(&event.event, rcm_com::Event::Menu { .. });
                let x = event.x;
                let y = event.y;

                if is_menu {
                    let dir = event.dir.clone();
                    let files_len = event.files.len();
                    let bg = event.bg;
                    log::info("egui", &format!(
                        "rcm_com Menu: pos=({},{}) dir='{}' files={} bg={}",
                        x, y, dir, files_len, bg
                    ));
                    if let Some(tx) = pipe::EVENT_SENDER.lock().unwrap().as_ref() {
                        let _ = tx.send(AppEvent::ShowMenuWithInfo {
                            x: x as f64,
                            y: y as f64,
                            info: event,
                        });
                    }
                } else {
                    log::info("egui", &format!(
                        "non-Menu event (dev={})", config::is_dev()
                    ));
                    if !config::is_dev() {
                        if let Some(tx) = pipe::EVENT_SENDER.lock().unwrap().as_ref() {
                            let _ = tx.send(AppEvent::HideAll);
                        }
                    }
                }
            })
            .await
            {
                log::error("egui", &format!("rcm_com::server::listen ERROR: {e}"));
            }
        });
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// Theme
// ═══════════════════════════════════════════════════════════════════════════

/// Color theme for menu rendering, matching Windows 11 context menu look.
struct MenuTheme {
    bg: Color32,
    hover_bg: Color32,
    disabled_text: Color32,
    text: Color32,
    separator: Color32,
    border: Color32,
}

impl MenuTheme {
    /// Dark mode theme (matches Windows 11 dark context menu).
    fn dark() -> Self {
        Self {
            bg: Color32::from_rgb(32, 32, 32),
            hover_bg: Color32::from_rgb(60, 60, 60),
            disabled_text: Color32::from_rgb(128, 128, 128),
            text: Color32::from_rgb(220, 220, 220),
            separator: Color32::from_rgb(80, 80, 80),
            border: Color32::from_rgb(60, 60, 60),
        }
    }

    /// Light mode theme (matches Windows 11 light context menu).
    fn light() -> Self {
        Self {
            bg: Color32::from_rgb(252, 252, 252),
            hover_bg: Color32::from_rgb(229, 243, 255),
            disabled_text: Color32::from_rgb(160, 160, 160),
            text: Color32::from_rgb(32, 32, 32),
            separator: Color32::from_rgb(220, 220, 220),
            border: Color32::from_rgb(200, 200, 200),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Menu rendering
// ═══════════════════════════════════════════════════════════════════════════

/// Render a single menu item row. Returns whether the item was clicked.
fn render_item(
    ui: &mut egui::Ui,
    item: &rcm_core::Item,
    theme: &MenuTheme,
    show_icons: bool,
) -> Option<bool> {
    let item_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        Vec2::new(ITEM_WIDTH, ITEM_HEIGHT),
    );

    if item.disable {
        // Disabled item: no interaction, grayed out text
        ui.add_enabled_ui(false, |ui| {
            ui.allocate_rect(item_rect, egui::Sense::hover());
        });
        // Paint label
        let label_x = item_rect.min.x + if show_icons { ICON_WIDTH } else { 8.0 };
        ui.painter().text(
            egui::pos2(label_x, item_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &item.label,
            egui::FontId::proportional(13.0),
            theme.disabled_text,
        );
        ui.advance_cursor_after_rect(item_rect);
        return None;
    }

    // Allocate clickable area
    let resp = ui.allocate_rect(item_rect, egui::Sense::click());
    let hovered = resp.hovered();
    let clicked = resp.clicked();

    // Hover background
    if hovered {
        ui.painter()
            .rect_filled(item_rect, egui::CornerRadius::ZERO, theme.hover_bg);
    }

    // Icon column placeholder
    if show_icons {
        let icon_rect = egui::Rect::from_min_size(
            item_rect.min + Vec2::new(4.0, 4.0),
            Vec2::new(ICON_WIDTH - 8.0, ITEM_HEIGHT - 8.0),
        );
        if !item.icon.is_empty() {
            // In a full implementation, we'd load/display the icon here
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                &item.icon,
                egui::FontId::proportional(14.0),
                theme.text,
            );
        }
    }

    // Label text
    let label_x = item_rect.min.x + if show_icons { ICON_WIDTH } else { 8.0 };
    ui.painter().text(
        egui::pos2(label_x, item_rect.center().y),
        egui::Align2::LEFT_CENTER,
        &item.label,
        egui::FontId::proportional(13.0),
        theme.text,
    );

    // Submenu arrow indicator
    if item.has_children() {
        ui.painter().text(
            egui::pos2(item_rect.max.x - 12.0, item_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "▶",
            egui::FontId::proportional(9.0),
            theme.disabled_text,
        );
    }

    // Advance UI cursor past this item
    ui.advance_cursor_after_rect(item_rect);

    if clicked {
        Some(hovered)
    } else {
        None
    }
}

/// Render a separator line between menu groups.
fn render_separator(ui: &mut egui::Ui, theme: &MenuTheme) {
    let rect = egui::Rect::from_min_size(
        ui.cursor().min + Vec2::new(8.0, 4.0),
        Vec2::new(ITEM_WIDTH - 16.0, 1.0),
    );
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, theme.separator);
    ui.advance_cursor_after_rect(egui::Rect::from_min_size(
        ui.cursor().min,
        Vec2::new(ITEM_WIDTH, SEPARATOR_HEIGHT),
    ));
}

/// Execute a menu item's command asynchronously and optionally hide the menu.
fn execute_command(cmd: &CommandPayload, _dev_mode: bool) {
    let command = cmd.clone();
    log::info("egui", &format!(
        "execute: exe='{}' args={:?}", command.exe, command.args
    ));

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime for command execution");
        rt.block_on(async move {
            let result = cmd::execute(command).await;
            if !result.success {
                log::error("egui", &format!("command failed: {:?}", result));
            }
        });
    });
}

/// Render the context menu directly in the egui window.
/// The window should already be positioned at the menu location by `logic()`.
fn render_menu(ui: &mut egui::Ui, menu_state: &mut MenuState) {
    let menu = match menu_state.menu.as_ref() {
        Some(m) => m.clone(),
        None => return,
    };

    let show_icons = menu_state.show_icons;
    let theme = if ui.ctx().global_style().visuals.dark_mode {
        MenuTheme::dark()
    } else {
        MenuTheme::light()
    };

    let icon_items = &menu.icon_items;
    let groups = &menu.groups;

    let menu_size = match menu_state.menu_size() {
        Some(s) => s,
        None => return,
    };

    // Background — fill the entire window content area
    let bg_rect = ui.max_rect();
    ui.painter().rect_filled(bg_rect, egui::CornerRadius::same(8), theme.bg);
    ui.painter()
        .rect_stroke(bg_rect, egui::CornerRadius::same(8), Stroke::new(1.0_f32, theme.border), egui::StrokeKind::Middle);

    // Inner padded UI
    let mut menu_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_size(
                egui::pos2(MENU_PADDING, MENU_PADDING),
                Vec2::new(ITEM_WIDTH, menu_size.y - MENU_PADDING * 2.0),
            ))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    let mut any_clicked = false;

    // ── Icon ribbon items ────────────────────────────────
    if !icon_items.is_empty() {
        for item in icon_items {
            if render_item(&mut menu_ui, item, &theme, show_icons).is_some() {
                if !item.disable {
                    if let Some(ref cmd) = item.command {
                        execute_command(cmd, menu_state.dev_mode);
                        any_clicked = true;
                    }
                }
            }
        }
        if !groups.is_empty() {
            render_separator(&mut menu_ui, &theme);
        }
    }

    // ── Groups ───────────────────────────────────────────
    for (gi, group) in groups.iter().enumerate() {
        if !group.label.is_empty() {
            let header_rect = egui::Rect::from_min_size(
                menu_ui.cursor().min,
                Vec2::new(ITEM_WIDTH, ITEM_HEIGHT * 0.7),
            );
            menu_ui.painter().text(
                egui::pos2(header_rect.min.x + 8.0, header_rect.center().y),
                egui::Align2::LEFT_CENTER,
                &group.label,
                egui::FontId::proportional(11.0),
                theme.disabled_text,
            );
            menu_ui.advance_cursor_after_rect(header_rect);
        }

        for item in &group.items {
            if render_item(&mut menu_ui, item, &theme, show_icons).is_some() {
                if !item.disable {
                    if let Some(ref cmd) = item.command {
                        execute_command(cmd, menu_state.dev_mode);
                        any_clicked = true;
                    }
                }
            }
        }

        if gi < groups.len() - 1 {
            render_separator(&mut menu_ui, &theme);
        }
    }

    if any_clicked && !menu_state.dev_mode {
        menu_state.hide();
    }

    ui.ctx().request_repaint();
}

// ═══════════════════════════════════════════════════════════════════════════
// eframe::App implementation
// ═══════════════════════════════════════════════════════════════════════════

impl eframe::App for RcmEguiApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    /// Called before each frame — process events and reposition the window.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_background_events();

        if self.should_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // ── Continuously reposition/size window based on menu visibility ──
        if self.menu_state.visible {
            if let Some(size) = self.menu_state.menu_size() {
                // rcm_com coordinates are in physical pixels — convert to egui points
                let dpi_scale = ctx.pixels_per_point();
                let x = self.menu_state.root_x as f32 / dpi_scale;
                let y = self.menu_state.root_y as f32 / dpi_scale;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                    egui::Pos2::new(x, y),
                ));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                egui::Pos2::new(-9999.0, -9999.0),
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                Vec2::new(1.0, 1.0),
            ));
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    /// Called each frame to render UI.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.menu_state.visible {
            render_menu(ui, &mut self.menu_state);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log::info("egui", "application shutting down");
    }
}
