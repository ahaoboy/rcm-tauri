//! Small modal-style error window.
//!
//! Used for startup / pull failures, mirroring the Tauri build's
//! `run_error` / `show_error_window` helpers.

use windows_reactor::*;

use crate::menu_window::window_theme;

#[derive(Clone)]
pub struct ErrorInput {
    pub title: String,
    pub message: String,
}

impl PartialEq for ErrorInput {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.message == other.message
    }
}

pub struct ErrorWindow {
    input: ErrorInput,
}

#[derive(Clone)]
pub enum Message {
    Close,
}

impl Component for ErrorWindow {
    type Message = Message;
    type Input = ErrorInput;

    fn create(input: &Self::Input, _context: &ComponentContext<Self>) -> Self {
        Self {
            input: input.clone(),
        }
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Close => {
                let _ = context.window().request_close();
            }
        }
    }

    fn view(&self, _input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        context.window_title(&self.input.title);
        context.window_visuals(
            WindowVisuals::new()
                .theme(window_theme())
                .client_size(460.0, 240.0),
        );

        Border::new().padding(16.0).content(
            StackPanel::new().spacing(12.0).children((
                TextBlock::new()
                    .text(self.input.title.clone())
                    .font_size(18.0),
                TextBlock::new()
                    .text(self.input.message.clone())
                    .max_lines(10),
                Button::new()
                    .style(ButtonStyle::Accent)
                    .on_click(context.message(Message::Close))
                    .content("OK"),
            )),
        )
    }
}
