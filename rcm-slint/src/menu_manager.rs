//! Menu manager — creates and manages Slint context menu windows.
//! Replaces the Tauri MenuManager from src-tauri/src/lib.rs.
//!
//! Uses click-based submenu navigation (same as native Windows menus).
//! All window operations happen on the Slint event loop thread.
//! Uses Rc<RefCell<>> because Slint components are !Send + !Sync.

use crate::{CommandData, MenuItemData, MenuWindow};
use rcm_core::{Item, Menu};
use rcm_core::{config, log};
use slint::{ComponentHandle, Model, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const MAX_SUBMENU_DEPTH: usize = 4;
const OFF_SCREEN: f64 = -9999.0;
static DEEPEST_DEPTH: AtomicUsize = AtomicUsize::new(0);
pub type MenuArc = Arc<Mutex<Option<Menu>>>;

pub struct MenuManager {
    menu: MenuArc,
    pub root_window: MenuWindow,
    pub submenus: Vec<Option<MenuWindow>>,
}

impl MenuManager {
    pub fn new(menu: MenuArc) -> Rc<RefCell<Self>> {
        let root = MenuWindow::new().expect("Failed to create root MenuWindow");
        root.set_depth(0);
        root.set_show_icons(config::is_icons());
        root.hide().ok();

        let mut submenus: Vec<Option<MenuWindow>> = Vec::with_capacity(MAX_SUBMENU_DEPTH);
        for d in 0..MAX_SUBMENU_DEPTH {
            let win = MenuWindow::new()
                .unwrap_or_else(|_| panic!("Failed to create submenu window {d}"));
            win.set_depth((d + 1) as i32);
            win.set_show_icons(config::is_icons());
            win.hide().ok();
            submenus.push(Some(win));
        }

        let mgr = Rc::new(RefCell::new(Self {
            menu,
            root_window: root,
            submenus,
        }));

        // Wire callbacks for root window
        {
            let _mgr_weak = Rc::downgrade(&mgr);
            let _w = mgr.borrow().root_window.as_weak();
            wire_callbacks(&mgr.borrow().root_window, 0);
        }

        // Wire up callbacks for submenu windows
        for (d, win_opt) in mgr.borrow().submenus.iter().enumerate() {
            if let Some(win) = win_opt {
                wire_callbacks(win, d + 1);
            }
        }

        mgr
    }

    pub fn show_root(&self, menu_data: Menu, x: f64, y: f64) {
        log::info(
            "MenuManager::show_root",
            &format!(
                "pos=({x:.0},{y:.0}) items={} icons={}",
                menu_data.groups.iter().map(|g| g.items.len()).sum::<usize>(),
                menu_data.icon_items.len()
            ),
        );
        *self.menu.lock().unwrap() = Some(menu_data.clone());
        DEEPEST_DEPTH.store(0, Ordering::SeqCst);
        self.populate_root(&self.root_window, &menu_data);
        self.root_window.set_show_icons(config::is_icons());
        self.root_window
            .window()
            .set_position(slint::PhysicalPosition::new(x as i32, y as i32));
        self.root_window.show().ok();
    }

    pub fn handle_submenu_request(
        &self,
        depth: usize,
        path: &[i32],
        parent_x: f64,
        parent_y: f64,
        parent_w: f64,
    ) {
        let menu_guard = self.menu.lock().unwrap();
        let menu = match menu_guard.as_ref() {
            Some(m) => m,
            None => return,
        };
        let item = match menu.get_item(&path.to_vec()) {
            Some(i) => i,
            None => return,
        };
        if item.disable || !item.has_children() {
            return;
        }
        let child_depth = depth + 1;
        if child_depth > MAX_SUBMENU_DEPTH {
            return;
        }
        let children: Vec<MenuItemData> =
            item.items.iter().map(convert_item_to_slint).collect();
        drop(menu_guard);
        self.hide_deeper_than(child_depth);

        if let Some(Some(sub_win)) = self.submenus.get(depth) {
            sub_win.set_items(Rc::new(VecModel::from(children)).into());
            sub_win.set_icon_items(Rc::new(VecModel::default()).into());
            sub_win.set_show_icons(config::is_icons());
            sub_win.window().set_position(slint::PhysicalPosition::new(
                (parent_x + parent_w) as i32,
                parent_y as i32,
            ));
            sub_win.show().ok();
        }
        DEEPEST_DEPTH.store(child_depth, Ordering::SeqCst);
    }

    pub fn handle_execute(&self, cmd: rcm_core::CommandPayload) {
        log::info("MenuManager::handle_execute", &format!("exe='{}'", cmd.exe));
        tokio::spawn(async move {
            let result = crate::cmd::execute(cmd).await;
            if !result.success {
                log::error(
                    "MenuManager::handle_execute",
                    &format!("FAILED: {:?}", result),
                );
            }
        });
        if !config::is_dev() {
            self.hide_all();
        }
    }

    pub fn handle_blur(&self, depth: usize) {
        if depth == DEEPEST_DEPTH.load(Ordering::SeqCst) && !config::is_dev() {
            self.hide_all();
        }
    }

    pub fn hide_all(&self) {
        DEEPEST_DEPTH.store(0, Ordering::SeqCst);
        self.root_window.hide().ok();
        self.root_window.window().set_position(
            slint::PhysicalPosition::new(OFF_SCREEN as i32, OFF_SCREEN as i32),
        );
        for win_opt in &self.submenus {
            if let Some(win) = win_opt {
                win.hide().ok();
                win.window().set_position(
                    slint::PhysicalPosition::new(OFF_SCREEN as i32, OFF_SCREEN as i32),
                );
            }
        }
    }

    fn hide_deeper_than(&self, depth: usize) {
        for d in depth..MAX_SUBMENU_DEPTH {
            if let Some(Some(win)) = self.submenus.get(d) {
                win.hide().ok();
                win.window().set_position(
                    slint::PhysicalPosition::new(OFF_SCREEN as i32, OFF_SCREEN as i32),
                );
            }
        }
        DEEPEST_DEPTH.store(depth, Ordering::SeqCst);
    }

    fn populate_root(&self, win: &MenuWindow, menu: &Menu) {
        let icon_items: Vec<MenuItemData> = menu
            .icon_items
            .iter()
            .map(convert_item_to_slint)
            .collect();
        win.set_icon_items(Rc::new(VecModel::from(icon_items)).into());

        let mut flat_items: Vec<MenuItemData> = Vec::new();
        for (gidx, group) in menu.groups.iter().enumerate() {
            if gidx > 0 {
                flat_items.push(MenuItemData {
                    key: String::new().into(),
                    icon: String::new().into(),
                    label: String::new().into(),
                    disable: true,
                    admin: false,
                    window: String::new().into(),
                    has_children: false,
                    is_separator: true,
                    command: CommandData::default(),
                });
            }
            for item in &group.items {
                flat_items.push(convert_item_to_slint(item));
            }
        }
        win.set_items(Rc::new(VecModel::from(flat_items)).into());
    }
}

// ── Callback wiring (free function to avoid borrow-checker issues) ──────

fn wire_callbacks(
    win: &MenuWindow,
    depth: usize,
) {
    // Use channel-based communication instead of Weak refs
    // (Slint callbacks may require Send, which Rc::Weak is not)

    let d = depth;
    win.on_submenu_requested(move |_depth_val, path_model| {
        let path: Vec<i32> = (0..path_model.row_count())
            .filter_map(|i| path_model.row_data(i))
            .collect();
        // Send through global channel
        crate::send_event(crate::AppEvent::SubmenuRequested {
            depth: d,
            path,
        });
    });

    let d2 = depth;
    win.on_item_executed(
        move |_depth_val, _path_model, exe, args_model, cwd, admin, window| {
            let exe_str = exe.to_string();
            let args: Vec<String> = (0..args_model.row_count())
                .filter_map(|i| args_model.row_data(i))
                .map(|s: slint::SharedString| s.to_string())
                .collect();
            let cwd_str = cwd.to_string();
            let win_str = window.to_string();
            crate::send_event(crate::AppEvent::ItemExecuted {
                depth: d2,
                cmd: rcm_core::CommandPayload {
                    exe: exe_str,
                    args,
                    cwd: cwd_str,
                    admin,
                    window: win_str,
                },
            });
        },
    );

    let d3 = depth;
    win.on_window_blurred(move |_depth_val| {
        crate::send_event(crate::AppEvent::WindowBlurred { depth: d3 });
    });
}

// ── Conversion helpers ──────────────────────────────────────────────────

fn convert_item_to_slint(item: &Item) -> MenuItemData {
    use slint::SharedString;

    let cmd = match &item.command {
        Some(c) => {
            let args_vec: Vec<SharedString> =
                c.args.iter().map(|s| SharedString::from(s.as_str())).collect();
            CommandData {
                exe: c.exe.clone().into(),
                args: Rc::new(VecModel::from(args_vec)).into(),
                cwd: c.cwd.clone().into(),
                admin: c.admin,
                window: c.window.clone().into(),
            }
        }
        None => CommandData::default(),
    };

    MenuItemData {
        key: item.key.clone().into(),
        icon: item.icon.clone().into(),
        label: item.label.clone().into(),
        disable: item.disable,
        admin: item.admin,
        window: item.window.clone().into(),
        has_children: !item.items.is_empty(),
        is_separator: false,
        command: cmd,
    }
}
