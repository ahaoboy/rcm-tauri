//! Embedded `style.css` handling.
//!
//! Both frontends ship the same default stylesheet and write it next to the
//! executable on startup, so it can be hand-edited or refreshed from the
//! configured remote URL. Reactor's native UI does not *consume* the CSS, but
//! keeping the file identical means the config editor and the Pull action
//! behave the same in both builds.

use std::sync::OnceLock;

use crate::log;

/// The default stylesheet, embedded at compile time.
pub const DEFAULT_STYLE: &str = include_str!("../../rcm-ui/style.css");

/// Path of the on-disk stylesheet next to the executable.
pub fn style_path() -> std::path::PathBuf {
    crate::exe_dir().join("style.css")
}

/// Write the embedded default stylesheet next to the executable.
pub fn write_style_defaults() {
    let path = style_path();
    match std::fs::write(&path, DEFAULT_STYLE) {
        Ok(()) => log::info("Style", &format!("wrote {}", path.display())),
        Err(e) => log::error("Style", &format!("write {} failed: {e}", path.display())),
    }
}

/// The stylesheet to serve to the UI, cached after the first load.
///
/// Reads `style.css` from disk when present, otherwise writes the embedded
/// default and returns that.
pub fn load_style_css() -> String {
    static LOADED: OnceLock<String> = OnceLock::new();

    LOADED
        .get_or_init(|| {
            let path = style_path();
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(css) => {
                        log::info("Style", "loaded style.css from disk");
                        return css;
                    }
                    Err(e) => {
                        log::error("Style", &format!("read {} failed: {e}", path.display()));
                    }
                }
            }

            match std::fs::write(&path, DEFAULT_STYLE) {
                Ok(()) => log::info("Style", "wrote default style.css"),
                Err(e) => {
                    log::error("Style", &format!("write {} failed: {e}", path.display()));
                }
            }
            DEFAULT_STYLE.to_string()
        })
        .clone()
}
