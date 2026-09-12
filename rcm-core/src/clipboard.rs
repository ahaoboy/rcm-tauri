//! Clipboard format detection via clipboard-rs.
//! Sniffs available clipboard formats without reading the actual data.

use crate::types::ClipboardInfo;
use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};

/// Detect available clipboard formats at the current moment.
pub fn detect() -> ClipboardInfo {
    let ctx = match ClipboardContext::new() {
        Ok(c) => c,
        Err(_) => return ClipboardInfo::default(),
    };

    ClipboardInfo {
        has_text: ctx.has(ContentFormat::Text),
        has_image: ctx.has(ContentFormat::Image),
        has_files: ctx.has(ContentFormat::Files),
    }
}

/// Put `text` on the clipboard.
///
/// A frontend that owns its own text buffer (the config editor) has no command
/// to copy through, so the raw write lives here next to [`detect`] and uses the
/// same error wording as the copy commands.
pub fn write_text(text: &str) -> Result<(), String> {
    let ctx = ClipboardContext::new().map_err(|e| format!("Failed to open clipboard: {e}"))?;
    ctx.set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard: {e}"))
}
