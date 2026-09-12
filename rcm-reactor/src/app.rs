//! Application root component.
//!
//! Reactor needs at least one window for the process to stay alive. `RcmApp`
//! owns a tiny, off-screen tool window that anchors the UI thread, pumps the
//! cross-thread event queue on a timer, and opens the menu / config / error
//! popups on demand.

use std::time::Duration;

use windows_reactor::*;

use rcm_core::{config, log};

use crate::config_editor::ConfigEditor;
use crate::error_window::{ErrorInput, ErrorWindow};
use crate::events::{self, AppEvent, POLL_MS};
use crate::menu_window::{MenuInput, MenuWindow};
use crate::visuals;
use crate::{menu_runtime, monitor, tray, win32};

pub struct RcmApp {
    started: bool,
    init_timer: Option<ComponentTimer>,
    tick_timer: Option<ComponentTimer>,
}

#[derive(Clone)]
pub enum Message {
    /// Move the anchor window off-screen and mark it as a tool window.
    Init,
    /// Drain the event queue + run idle/blur/auto-hide checks.
    Tick,
    /// No-op message (used as the `run_window` return value).
    Noop,
}

impl RcmApp {
    fn handle_events(&self, context: &ComponentContext<Self>) {
        for event in events::drain() {
            match event {
                AppEvent::ShowMenu { menu, at } => {
                    // Let the shared controller build the level and reset its
                    // depth/activity bookkeeping before the window is created.
                    let Some(request) = menu_runtime::show_root((*menu).clone(), at) else {
                        log::error("Rust::app", "menu controller unavailable");
                        continue;
                    };
                    let input = MenuInput { menu, request };
                    if !context.open_window(View::component::<MenuWindow>(input)) {
                        log::warn("Rust::app", "open_window rejected for root menu");
                    }
                }
                AppEvent::HideAll => menu_runtime::hide_all(),
                AppEvent::OpenConfigEditor => {
                    if !context.open_window(View::component::<ConfigEditor>(())) {
                        log::warn("Rust::app", "open_window rejected for config editor");
                    }
                }
                AppEvent::ShowError { title, message } => {
                    let _ = context.open_window(View::component::<ErrorWindow>(ErrorInput {
                        title,
                        message,
                    }));
                }
                // Menu windows read these values live from the config file on
                // their next render; refresh the controller's cached copies and
                // log the change for observability.
                AppEvent::IconsChanged(icons) => {
                    menu_runtime::refresh_settings();
                    log::info("Rust::app", &format!("icons changed: {icons}"));
                }
                AppEvent::DevChanged(dev) => {
                    menu_runtime::refresh_settings();
                    log::info("Rust::app", &format!("dev mode changed: {dev}"));
                }
                AppEvent::ThemeChanged(theme) => {
                    log::info("Rust::app", &format!("theme changed: {theme}"));
                }
                AppEvent::StyleChanged(_css) => {
                    // Reactor renders native WinUI controls, so the stylesheet
                    // has no effect on the UI. The config editor still reads it
                    // from disk, which is all that is needed.
                    log::info("Rust::app", "style.css refreshed (not applied)");
                }
            }
        }
    }

    /// Hide the menus when the user clicked away, or after a period of
    /// inactivity.
    ///
    /// The policy (deepest-depth blur, two-miss debounce, auto-hide timeout)
    /// lives in [`rcm_core::ui::MenuController::handle_idle`]; this only reports
    /// whether the foreground window belongs to us.
    fn handle_idle(&self) {
        if !self.started {
            return;
        }
        menu_runtime::handle_idle();
    }
}

impl Component for RcmApp {
    type Message = Message;
    type Input = ();

    fn create(_input: &(), context: &ComponentContext<Self>) -> Self {
        config::init();
        if let Err(e) = rcm_com::enable() {
            log::error("Startup", &format!("rcm_com::enable failed: {e}"));
        }
        rcm_core::style::write_style_defaults();

        tray::start();
        monitor::start();

        let mut app = Self {
            started: false,
            init_timer: None,
            tick_timer: None,
        };
        app.init_timer = context
            .set_timeout(Duration::from_millis(1), Message::Init)
            .ok();
        app.tick_timer = context
            .set_timeout(Duration::from_millis(POLL_MS), Message::Tick)
            .ok();
        app
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Init => {
                let _ = context.run_window(|handle| {
                    let raw = handle.as_raw();
                    win32::mark_tool_window(raw);
                    win32::move_window(raw, -32000, -32000, 1, 1);
                    Message::Noop
                });
                self.started = true;
                self.init_timer = None;
            }
            Message::Tick => {
                self.handle_events(context);
                self.handle_idle();
                self.tick_timer = context
                    .set_timeout(Duration::from_millis(POLL_MS), Message::Tick)
                    .ok();
            }
            Message::Noop => {}
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title(events::ROOT_TITLE);
        context.window_visuals(
            WindowVisuals::new()
                .theme(visuals::window_theme())
                .backdrop(visuals::WINDOW_BACKDROP)
                .client_size(1.0, 1.0),
        );
        Border::new().content(TextBlock::new().text(""))
    }
}
