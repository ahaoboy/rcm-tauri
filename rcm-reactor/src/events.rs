//! Cross-thread event plumbing.
//!
//! The tray icon and the `rcm_com` pipe monitor both run on their own threads
//! and cannot touch the Reactor UI directly (Reactor is single-threaded on the
//! UI thread). Instead they push [`AppEvent`]s into a global queue which the
//! root [`crate::app::RcmApp`] component drains on a timer.

use rcm_core::Menu;
use rcm_core::ui::{MenuMetrics, Point};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

// ═══════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════

/// How often the root component polls the event queue and timers (milliseconds).
pub const POLL_MS: u64 = 40;

/// Root window label / title used for the hidden anchor window.
pub const ROOT_TITLE: &str = "rcm-reactor";

/// Title of every menu popup window.
pub const MENU_TITLE: &str = "rcm-menu";

/// Subtle fill used to highlight the row under the pointer (ARGB).
///
/// A translucent neutral grey reads as a highlight on both light and dark menu
/// backgrounds — WinUI's `CardStroke` is invisible on dark surfaces.
pub const MENU_HOVER_ARGB: (u8, u8, u8, u8) = (0x30, 0x80, 0x80, 0x80);

/// Menus rendered with Reactor.
///
/// Layout geometry (`submenu_gap`, `edge_gap`, auto-hide, depth limit) comes
/// from [`rcm_core::ui`] and is shared with the Tauri build; only the row
/// presentation metrics are Reactor-specific.
pub const METRICS: MenuMetrics = MenuMetrics::for_reactor();

// ═══════════════════════════════════════════════════════════════════════════
// Events
// ═══════════════════════════════════════════════════════════════════════════

/// Everything the tray thread and the pipe monitor can tell the UI thread.
#[derive(Clone)]
pub enum AppEvent {
    /// A right-click was captured — build + show a fresh menu tree.
    ShowMenu { menu: Arc<Menu>, at: Point },
    /// Close every open menu window.
    HideAll,
    /// Open the config editor window.
    OpenConfigEditor,
    /// Show a small error window.
    ShowError { title: String, message: String },
    /// The icon-ribbon preference changed (tray toggle).
    IconsChanged(bool),
    /// Dev mode changed (tray toggle).
    DevChanged(bool),
    /// Theme changed (tray toggle).
    ThemeChanged(String),
    /// The stylesheet was refreshed from the remote URL (tray "Pull CSS").
    ///
    /// Reactor's native UI does not consume CSS, so this is informational.
    StyleChanged(String),
}

fn queue() -> &'static Mutex<VecDeque<AppEvent>> {
    static QUEUE: OnceLock<Mutex<VecDeque<AppEvent>>> = OnceLock::new();
    QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// Push an event from any thread.
pub fn push(event: AppEvent) {
    if let Ok(mut q) = queue().lock() {
        q.push_back(event);
    }
}

/// Take every queued event (drained in FIFO order).
pub fn drain() -> Vec<AppEvent> {
    match queue().lock() {
        Ok(mut q) => q.drain(..).collect(),
        Err(_) => Vec::new(),
    }
}
