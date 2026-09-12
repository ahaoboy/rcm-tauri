//! Shared menu-window bookkeeping.
//!
//! Everything here is pure state: which menu windows are open at which depth,
//! how deep the visible menu goes, when the last interaction happened, and
//! whether focus has been lost. None of it touches a toolkit, so the Tauri and
//! Reactor frontends behave identically.
//!
//! The window handle type is generic ([`MenuState::Window`]) — Tauri would use
//! its window labels and Reactor its `HWND`s.

use std::collections::BTreeMap;
use std::time::Instant;

/// Tracks the open menu windows and the interaction/focus bookkeeping.
///
/// Mirrors the state `rcm-tauri`'s `MenuManager` kept via globals
/// (`DEEPEST_DEPTH`, the auto-hide epoch) plus the frontend's blur handling.
#[derive(Debug)]
pub struct MenuState<W: Copy + Ord> {
    /// `depth -> window handle`.
    windows: BTreeMap<usize, W>,
    /// Depth of the deepest menu currently visible.
    deepest: usize,
    /// Whether one of our windows has actually held focus since the menu was
    /// last hidden.
    ///
    /// Dismissal only engages after this is true, so a host that is refused
    /// foreground (e.g. Windows denying `SetForegroundWindow`) cannot cause the
    /// menu to close the instant it opens.
    was_foreground: bool,
    /// Consecutive polls on which none of our windows held focus.
    ///
    /// A single miss is normal while a parent hands activation to the submenu
    /// it just opened, so dismissal waits for two in a row.
    foreground_misses: u32,
    /// When the user last interacted with the menu.
    last_activity: Option<Instant>,
}

impl<W: Copy + Ord> Default for MenuState<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Copy + Ord> MenuState<W> {
    pub const fn new() -> Self {
        Self {
            windows: BTreeMap::new(),
            deepest: 0,
            was_foreground: false,
            foreground_misses: 0,
            last_activity: None,
        }
    }

    // ── Window registry ─────────────────────────────────────────────────

    /// Record that `window` is now showing at `depth`, and reset the idle timer.
    pub fn register(&mut self, depth: usize, window: W) {
        self.windows.insert(depth, window);
        if depth >= self.deepest {
            self.deepest = depth;
        }
        self.touch();
    }

    /// Whether any menu window is open.
    pub fn has_windows(&self) -> bool {
        !self.windows.is_empty()
    }

    /// The window at `depth`, if open.
    pub fn window(&self, depth: usize) -> Option<W> {
        self.windows.get(&depth).copied()
    }

    /// Depths of every open level, shallowest first.
    pub fn depths(&self) -> Vec<usize> {
        self.windows.keys().copied().collect()
    }

    /// The depth a window is registered at, if it is ours.
    pub fn depth_of(&self, window: W) -> Option<usize> {
        self.windows
            .iter()
            .find_map(|(depth, w)| (*w == window).then_some(*depth))
    }

    /// Whether `window` belongs to the open menu.
    pub fn is_open_window(&self, window: W) -> bool {
        self.windows.values().any(|w| *w == window)
    }

    /// Depth of the deepest visible menu.
    pub const fn deepest(&self) -> usize {
        self.deepest
    }

    // ── Auto-hide / activity ────────────────────────────────────────────

    /// Reset the idle timer and clear any pending "focus left" suspicion.
    ///
    /// Call on every user interaction. Clearing the miss counter here is what
    /// keeps the menu alive while the pointer moves through it: a blur can
    /// arrive without focus having actually left the menu (focus moving between
    /// our own windows, or a level we just closed), so only a run of
    /// *uninterrupted* misses may dismiss.
    pub fn touch(&mut self) {
        self.last_activity = Some(Instant::now());
        self.foreground_misses = 0;
    }

    /// Milliseconds since the last interaction, or `None` if there was never
    /// one (in which case nothing should auto-hide).
    pub fn idle_ms(&self) -> Option<u128> {
        self.last_activity.map(|t| t.elapsed().as_millis())
    }

    /// Whether the menu has been idle for at least `timeout_ms`.
    pub fn is_idle(&self, timeout_ms: u64) -> bool {
        self.idle_ms()
            .is_some_and(|idle| idle >= u128::from(timeout_ms))
    }

    // ── Focus / blur ────────────────────────────────────────────────────

    /// Record that one of our windows became the foreground window.
    pub fn note_foreground(&mut self) {
        self.was_foreground = true;
        self.foreground_misses = 0;
    }

    /// Whether a menu window has held focus since the menu was last hidden.
    pub const fn was_foreground(&self) -> bool {
        self.was_foreground
    }

    /// Record that the foreground window is *not* one of ours.
    ///
    /// Returns `true` once this has happened on two consecutive checks, which
    /// is when the menu is considered to have genuinely lost focus.
    pub fn note_foreground_miss(&mut self) -> bool {
        self.foreground_misses += 1;
        self.foreground_misses >= 2
    }

    // ── Closing ─────────────────────────────────────────────────────────

    /// Remove and return every window deeper than `depth`, and lower the
    /// deepest-visible marker to `depth`.
    ///
    /// The caller is responsible for actually hiding/closing the returned
    /// handles, which keeps this decoupled from the toolkit.
    pub fn take_deeper_than(&mut self, depth: usize) -> Vec<W> {
        let doomed: Vec<usize> = self
            .windows
            .keys()
            .copied()
            .filter(|d| *d > depth)
            .collect();
        let windows = doomed
            .into_iter()
            .filter_map(|d| self.windows.remove(&d))
            .collect();
        self.deepest = depth;
        windows
    }

    /// Remove and return every open window, resetting all bookkeeping.
    pub fn take_all(&mut self) -> Vec<W> {
        let windows = std::mem::take(&mut self.windows).into_values().collect();
        self.reset();
        windows
    }

    /// Reset depth, focus and activity tracking.
    fn reset(&mut self) {
        self.deepest = 0;
        self.was_foreground = false;
        self.foreground_misses = 0;
        self.last_activity = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Windows are just integers in tests (e.g. HWNDs).
    type W = i32;

    #[test]
    fn starts_empty() {
        let state = MenuState::<W>::new();
        assert!(!state.has_windows());
        assert_eq!(state.deepest(), 0);
        assert_eq!(state.idle_ms(), None);
        assert!(!state.was_foreground());
    }

    #[test]
    fn register_tracks_deepest_and_resets_idle() {
        let mut state = MenuState::<W>::new();
        state.register(0, 10);
        state.register(2, 20);
        state.register(1, 30);

        assert_eq!(state.deepest(), 2);
        assert!(state.has_windows());
        assert!(state.idle_ms().is_some());
        assert!(state.window(2) == Some(20));
        assert_eq!(state.depth_of(30), Some(1));
        assert!(state.is_open_window(10));
        assert!(!state.is_open_window(99));
    }

    #[test]
    fn take_deeper_than_removes_only_deeper_and_lowers_depth() {
        let mut state = MenuState::<W>::new();
        state.register(0, 10);
        state.register(1, 20);
        state.register(2, 30);

        let closed = state.take_deeper_than(0);
        assert_eq!(closed.len(), 2);
        assert!(closed.contains(&20) && closed.contains(&30));
        assert_eq!(state.deepest(), 0);
        assert!(state.has_windows(), "the root window survives");
        assert!(state.window(0) == Some(10));
    }

    #[test]
    fn take_deeper_than_nothing_is_a_noop_apart_from_depth() {
        let mut state = MenuState::<W>::new();
        state.register(0, 10);
        state.register(1, 20);

        assert!(state.take_deeper_than(1).is_empty());
        assert_eq!(state.deepest(), 1);
    }

    #[test]
    fn take_all_clears_everything() {
        let mut state = MenuState::<W>::new();
        state.register(0, 10);
        state.register(1, 20);
        state.note_foreground();

        let closed = state.take_all();
        assert_eq!(closed.len(), 2);
        assert!(!state.has_windows());
        assert_eq!(state.deepest(), 0);
        assert_eq!(state.idle_ms(), None);
        assert!(!state.was_foreground());
    }

    #[test]
    fn interaction_clears_a_pending_focus_loss() {
        let mut state = MenuState::<W>::new();
        state.register(0, 10);
        state.note_foreground();
        state.note_foreground_miss();

        // The user is still moving through the menu, so a blur that arrived
        // without focus actually leaving must not count towards dismissal.
        state.touch();
        assert!(
            !state.note_foreground_miss(),
            "the miss counter restarted on interaction"
        );
        assert!(state.note_foreground_miss(), "a fresh run still dismisses");
    }

    #[test]
    fn blur_requires_two_consecutive_misses() {
        let mut state = MenuState::<W>::new();
        assert!(!state.note_foreground_miss(), "first miss is tolerated");
        assert!(state.note_foreground_miss(), "second miss dismisses");
        assert!(state.note_foreground_miss(), "stays true after that");
    }

    #[test]
    fn regaining_foreground_resets_the_miss_counter() {
        let mut state = MenuState::<W>::new();
        state.note_foreground_miss();
        state.note_foreground();
        assert!(state.was_foreground());
        assert!(
            !state.note_foreground_miss(),
            "counter restarted after regaining focus"
        );
    }

    #[test]
    fn idle_only_reports_after_an_interaction() {
        let mut state = MenuState::<W>::new();
        assert!(!state.is_idle(0), "never interacted => never idle");

        state.register(0, 10);
        assert!(!state.is_idle(60_000));
        assert!(state.is_idle(0), "idle for at least 0ms");
    }
}
