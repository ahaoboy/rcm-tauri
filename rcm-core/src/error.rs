//! User-facing errors surfaced in the error window.
//!
//! [`UiError`] is both a [`thiserror::Error`] (for idiomatic Rust error
//! handling) and `serde::Serialize` (for the wire). It is serialized as JSON
//! into `index.html#error/<json>`; the frontend mirrors the resulting
//! discriminated union and picks the rendering by `kind`.
//!
//! Adding an error kind is a single variant here plus one branch in the
//! frontend's `ErrorPage` — no other wiring.

use serde::Serialize;

/// Every error the error window can display.
///
/// The discriminant (`kind`, kebab-cased) and the fields are the contract with
/// the frontend; keep them in sync with `rcm-ui`'s `UiError` type.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum UiError {
    /// A menu action needs programs that are not resolvable on `PATH`.
    #[error("This action needs program(s) that are not installed, or not on PATH: {}", .programs.join(", "))]
    MissingPrograms {
        /// The missing program names, e.g. `["code", "mpv"]`.
        programs: Vec<String>,
    },

    /// A plain, pre-formatted message (diagnostics, reports).
    #[error("{message}")]
    Message {
        /// The message to display verbatim.
        message: String,
    },
}
