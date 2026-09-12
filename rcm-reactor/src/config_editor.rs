//! Config editor window.
//!
//! The Tauri build used a CodeMirror editor inside a WebView. Reactor has no
//! rich text editing, so this is a plain multi-line text editor offering the
//! same file set and actions (select, save, open in the default program).
//!
//! Validation and I/O come from [`rcm_core::files`], so the two frontends
//! accept exactly the same files and report exactly the same errors.

use windows_reactor::*;

use rcm_core::files::{DEFAULT_FILE, VALID_FILES};
use rcm_core::log;

use crate::menu_window::window_theme;

pub struct ConfigEditor {
    file: String,
    content: String,
    status: String,
}

#[derive(Clone)]
pub enum Message {
    Select(String),
    Changed(String),
    Save,
    OpenExternal,
    Close,
}

impl ConfigEditor {
    /// Load `name` into the editor, or record the error in the status line.
    fn load(&mut self, name: String) {
        match rcm_core::files::read_config_file(&name) {
            Ok(text) => {
                self.status = format!("Loaded {name}");
                self.content = text;
                self.file = name;
            }
            Err(e) => self.status = e,
        }
    }
}

impl Component for ConfigEditor {
    type Message = Message;
    type Input = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        let mut editor = Self {
            file: DEFAULT_FILE.to_string(),
            content: String::new(),
            status: String::new(),
        };
        editor.load(DEFAULT_FILE.to_string());
        editor
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Select(name) => self.load(name),
            Message::Changed(text) => self.content = text,
            Message::Save => {
                self.status = match rcm_core::files::save_config_file(&self.file, &self.content) {
                    Ok(()) => {
                        log::info("ConfigEditor", &format!("saved {}", self.file));
                        format!("Saved {}", self.file)
                    }
                    Err(e) => e,
                };
            }
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
                .theme(window_theme())
                .client_size(860.0, 620.0),
        );

        let file_buttons: Vec<KeyedView> = VALID_FILES
            .iter()
            .map(|name| {
                let label = if *name == self.file {
                    format!("● {name}")
                } else {
                    (*name).to_string()
                };
                KeyedView::new(
                    (*name).to_string(),
                    Button::new()
                        .on_click(context.message(Message::Select((*name).to_string())))
                        .content(label),
                )
            })
            .collect();

        let editor = TextBox::new()
            .text(self.content.clone())
            .accepts_return(true)
            .header(format!("Editing: {}", self.file))
            .height(440.0)
            .on_text_changed(context.callback(Message::Changed));

        Border::new().padding(16.0).content(
            StackPanel::new().spacing(10.0).children((
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .keyed_children(file_buttons),
                editor,
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .children((
                        Button::new()
                            .style(ButtonStyle::Accent)
                            .on_click(context.message(Message::Save))
                            .content("Save"),
                        Button::new()
                            .on_click(context.message(Message::OpenExternal))
                            .content("Open in default editor"),
                        Button::new()
                            .on_click(context.message(Message::Close))
                            .content("Close"),
                    )),
                TextBlock::new().text(self.status.clone()).opacity(0.7),
            )),
        )
    }
}
