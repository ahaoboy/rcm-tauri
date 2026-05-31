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

