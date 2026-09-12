//! Shared window appearance for every Reactor window.
//!
//! RCM shows several kinds of window — the hidden anchor, the menu popups, the
//! config editor and the error dialog — and they should all look like they came
//! from the same app. Theme and backdrop are exactly that kind of decision, so
//! they are made once here rather than repeated in each component's `view`.

use windows_reactor::{WindowBackdrop, WindowTheme};

use rcm_core::config;

/// The window theme to publish.
///
/// `Theme::System` maps to [`WindowTheme::System`], which lets WinUI follow the
/// OS light/dark preference live; the explicit variants pin it.
pub fn window_theme() -> WindowTheme {
    match config::theme() {
        config::Theme::System => WindowTheme::System,
        config::Theme::Light => WindowTheme::Light,
        config::Theme::Dark => WindowTheme::Dark,
    }
}

/// The material drawn behind every window's content.
///
/// Applied globally and unconditionally for now, so the effect can be judged
/// before deciding how (or whether) to expose it. `Mica` samples the desktop
/// wallpaper with a subtle tint, which is the Windows 11 default for app
/// windows.
///
/// Note this only becomes visible where a window's own content lets it through.
/// The menu popups, for instance, paint `ThemeBrush::CardBackground` over their
/// whole client area, so Mica is hidden beneath them until that border is made
/// translucent.
pub const WINDOW_BACKDROP: WindowBackdrop = WindowBackdrop::Acrylic;
