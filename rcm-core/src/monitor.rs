//! Right-click event classification, shared by every frontend.
//!
//! The shell extension forwards right-clicks over the `rcm_com` pipe. Both
//! frontends must apply exactly the same filters before showing anything, so
//! the decisions live here and each frontend only maps [`Action`] onto its own
//! event transport.
//!
//! Evaluating the menu itself stays in the frontend: it needs `rcm-vm`, which
//! is built on top of this crate.

use crate::{config, log};

/// What a frontend should do with one incoming event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Evaluate the menu with `rcm-vm` and show it at the event's position.
    Evaluate,
    /// Close every open menu.
    HideAll,
    /// Handled — nothing to do.
    Ignore,
}

/// Log `event`, apply the shared filters, and say what to do with it.
pub fn classify(event: &rcm_com::ContextMenuInfo) -> Action {
    log::event(
        "RECV",
        "rcm_com",
        &format!("{:?} pos=({},{})", event.event, event.x, event.y),
    );

    if let Some(reason) = config::ignore_reason(event) {
        log::info("Rust::monitor", &format!("filtered: {reason}"));
        return Action::Ignore;
    }

    match &event.event {
        rcm_com::Event::Menu { .. } => {
            // With blocking disabled the native system menu is already showing.
            // Showing ours too would stack two menus on top of each other, so
            // close any that are open and leave the native one alone.
            if !crate::ui::is_blocking_enabled() {
                log::info(
                    "Rust::monitor",
                    "blocking disabled — suppressing custom menu (native menu shown)",
                );
                return Action::HideAll;
            }
            Action::Evaluate
        }
        _ => {
            log::info(
                "Rust::monitor",
                &format!("non-Menu event (dev={})", config::is_dev()),
            );
            // Dev mode keeps the menu up so it can be inspected.
            if config::is_dev() {
                Action::Ignore
            } else {
                Action::HideAll
            }
        }
    }
}
