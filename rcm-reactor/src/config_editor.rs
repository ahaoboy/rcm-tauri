//! Config editor window.
//!
//! The Tauri build used a CodeMirror editor inside a WebView. Reactor has no
//! rich text editing, so this is a plain multi-line text editor offering the
//! same file set and actions (select, save, copy, open in the default program).
//!
//! The editor is a [`RichEditBox`] rather than a [`TextBox`]: a `TextBox` only
//! keeps newlines once `AcceptsReturn` is set, and Reactor applies `Text` before
//! `AcceptsReturn`, so a `TextBox` populated on its first frame silently
//! truncated to the first line. `RichEditBox` is multi-line by construction, so
//! the content survives the initial render.
//!
//! Validation and I/O come from [`rcm_core::files`], so the two frontends
//! accept exactly the same files and report exactly the same errors.

use std::collections::BTreeMap;

use windows_reactor::*;

use rcm_core::files::{DEFAULT_FILE, VALID_FILES};
use rcm_core::log;

use crate::visuals;

/// Convert the editor control's text into file text.
///
/// A WinUI rich-edit document differs from a plain file in two ways, both of
/// which have to be undone here:
///  * it uses `\r` as its paragraph mark, so lines come back `\r`-separated;
///  * it always ends with a paragraph mark, so the text comes back with one
///    trailing newline that is not part of the file.
///
/// Applied only at the **file and clipboard boundary**. The buffer keeps the
/// control's text verbatim, because Reactor only suppresses the echo of a value
/// the app writes back when it matches the control's value byte for byte —
/// normalising before storing would make the two differ on every render and
/// write the text back forever.
fn to_file_text(text: &str) -> String {
    let normalized = if text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        text.to_string()
    };
    normalized
        .strip_suffix('\n')
        .map_or(normalized.clone(), str::to_string)
}

/// One editable file: what the editor control holds and what the file contained.
#[derive(Default)]
struct Buffer {
    /// The control's text exactly as reported, i.e. with the document's own
    /// paragraph marks. See [`to_file_text`] for why it is not normalised.
    content: String,
    /// The file's text as last read or saved, in file form (`\n`, no trailing
    /// paragraph mark), so it compares directly with [`Self::content`].
    baseline: String,
}

pub struct ConfigEditor {
    /// File the editor is currently showing.
    file: String,
    /// Per-file state, so switching files neither discards unsaved edits nor
    /// loses track of which files are modified.
    buffers: BTreeMap<String, Buffer>,
    status: String,
}

#[derive(Clone)]
pub enum Message {
    Select(String),
    Changed(String),
    Save,
    Copy,
    OpenExternal,
    Close,
}

impl ConfigEditor {
    /// The current file's edited content.
    fn content(&self) -> &str {
        self.buffers
            .get(&self.file)
            .map_or("", |buffer| buffer.content.as_str())
    }

    /// Whether `name` has unsaved edits.
    ///
    /// Both sides are put in file form, so the document's paragraph marks do
    /// not read as an edit against a freshly loaded file.
    fn is_modified(&self, name: &str) -> bool {
        self.buffers
            .get(name)
            .is_some_and(|buffer| to_file_text(&buffer.content) != buffer.baseline)
    }

    /// Load `name` from disk the first time it is shown, or record the error.
    ///
    /// A file that is already buffered keeps its in-memory edits, so switching
    /// away and back never loses work.
    fn ensure_loaded(&mut self, name: &str) {
        if self.buffers.contains_key(name) {
            return;
        }
        match rcm_core::files::read_config_file(name) {
            Ok(text) => {
                let baseline = to_file_text(&text);
                self.buffers.insert(
                    name.to_string(),
                    Buffer {
                        baseline,
                        content: text,
                    },
                );
                self.status = format!("Loaded {name}");
            }
            Err(e) => self.status = e,
        }
    }

    /// Write the current buffer to disk and adopt it as the new baseline.
    fn save(&mut self) {
        let name = self.file.clone();
        let Some(buffer) = self.buffers.get_mut(&name) else {
            return;
        };
        // Converted here, not in `Message::Changed`, so the editor's own text
        // is never rewritten under it (which would loop; see `Buffer::content`).
        let text = to_file_text(&buffer.content);
        match rcm_core::files::save_config_file(&name, &text) {
            Ok(()) => {
                buffer.baseline = text;
                log::info("ConfigEditor", &format!("saved {name}"));
                self.status = format!("Saved {name}");
            }
            Err(e) => self.status = e,
        }
    }

    /// Copy the current buffer to the clipboard.
    fn copy(&mut self) {
        let result = rcm_core::clipboard::write_text(&to_file_text(self.content()));
        self.status = match result {
            Ok(()) => format!("Copied {} to clipboard", self.file),
            Err(e) => e,
        };
    }
}

impl Component for ConfigEditor {
    type Message = Message;
    type Input = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        let mut editor = Self {
            file: DEFAULT_FILE.to_string(),
            buffers: BTreeMap::new(),
            status: String::new(),
        };
        editor.ensure_loaded(DEFAULT_FILE);
        editor
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Select(name) => {
                self.file = name.clone();
                self.ensure_loaded(&name);
            }
            // Stored verbatim: Reactor treats a value equal to the control's
            // own as its own echo and suppresses the round trip. Normalising
            // here would make the two differ, so every render would write the
            // text back into the control and fire another change — an endless
            // loop that also grows the document by one paragraph mark each pass.
            Message::Changed(text) => {
                if let Some(buffer) = self.buffers.get_mut(&self.file) {
                    buffer.content = text;
                }
            }
            Message::Save => self.save(),
            Message::Copy => self.copy(),
            Message::OpenExternal => {
                self.status = match rcm_core::files::open_config_file(&self.file) {
                    Ok(()) => format!("Opened {} in default editor", self.file),
                    Err(e) => e,
                };
            }
            Message::Close => {
                let _ = context.window().request_close();
            }
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("RCM Config Editor");
        context.window_visuals(
            WindowVisuals::new()
                .theme(visuals::window_theme())
                .backdrop(visuals::WINDOW_BACKDROP)
                .client_size(860.0, 620.0),
        );

        // The file being edited is highlighted and a dot marks a file with
        // unsaved edits: two cues for two different questions ("where am I?"
        // and "what have I changed?").
        let file_buttons: Vec<KeyedView> = VALID_FILES
            .iter()
            .map(|name| {
                let label = if self.is_modified(name) {
                    format!("● {name}")
                } else {
                    (*name).to_string()
                };
                let style = if *name == self.file {
                    ButtonStyle::Accent
                } else {
                    ButtonStyle::Default
                };
                KeyedView::new(
                    (*name).to_string(),
                    Button::new()
                        .style(style)
                        .on_click(context.message(Message::Select((*name).to_string())))
                        .content(label),
                )
            })
            .collect();

        // A `RichEditBox` is multi-line from the moment it exists, so the
        // content can be published on the very first render.
        let editor = RichEditBox::new()
            .text(self.content().to_string())
            .header(format!("Editing: {}", self.file))
            .on_text_changed(context.callback(Message::Changed));

        // The editor takes the free space (`Star`) instead of a fixed height, so
        // it always lays out against a real size.
        Border::new().padding(16.0).content(
            Grid::new()
                .rows([
                    GridLength::Auto,
                    GridLength::Star(1.0),
                    GridLength::Auto,
                    GridLength::Auto,
                ])
                .children((
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(8.0)
                        .keyed_children(file_buttons),
                    editor.grid_row(1),
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(8.0)
                        .grid_row(2)
                        .children((
                            Button::new()
                                .style(ButtonStyle::Accent)
                                .on_click(context.message(Message::Save))
                                .content("Save"),
                            Button::new()
                                .on_click(context.message(Message::Copy))
                                .content("Copy"),
                            Button::new()
                                .on_click(context.message(Message::OpenExternal))
                                .content("Open in default editor"),
                            Button::new()
                                .on_click(context.message(Message::Close))
                                .content("Close"),
                        )),
                    TextBlock::new()
                        .text(self.status.clone())
                        .opacity(0.7)
                        .grid_row(3),
                )),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::to_file_text;

    #[test]
    fn editor_paragraph_marks_become_newlines() {
        // What the control reports for a two-line document.
        assert_eq!(to_file_text("{\r  \"dev\": false\r}\r"), "{\n  \"dev\": false\n}");
    }

    #[test]
    fn the_documents_final_paragraph_mark_is_not_file_content() {
        // The trailing mark is mandatory in the document, so it must not make a
        // freshly loaded file look edited (or be written back to disk).
        assert_eq!(to_file_text("}\r"), "}");
        assert_eq!(to_file_text("}\n"), "}");
        assert_eq!(to_file_text("}"), "}");
    }

    #[test]
    fn line_endings_are_normalised_regardless_of_form() {
        assert_eq!(to_file_text("a\r\nb\r\n"), to_file_text("a\rb\r"));
        assert_eq!(to_file_text("a\r\nb\r\n"), "a\nb");
    }
}
