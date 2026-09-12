//! Native context-menu popup rendered with Reactor.
//!
//! This is the Reactor counterpart of the Tauri build's WebView menu windows.
//! Each nesting level is its own borderless, always-on-top popup (opened with
//! [`ComponentContext::open_window`]).
//!
//! The component is a pure view: it draws the level it is handed, reports
//! pointer/focus events, and measures itself. Every decision — which level is
//! shown, where it goes, when it closes — is made by
//! [`rcm_core::ui::MenuController`] in [`crate::menu_runtime`].
//!
//! Measurement is the one thing Reactor must do itself, because only the
//! renderer knows the laid-out size. It is reported as a
//! [`Measurement`] and the controller answers with the final rectangle.

use std::sync::Arc;

use windows_reactor::*;

use rcm_core::log;
use rcm_core::types::{CommandPayload, Item, Menu};
use rcm_core::ui::{HoverInfo, HoverResult, Measurement, MenuLevel, MenuRow, MenuShowRequest};

use crate::events::{MENU_HOVER_ARGB, MENU_TITLE};
use crate::metrics::{MENU_STYLE, MenuStyle, level_height, row_offset};
use crate::{exec, menu_runtime, visuals, win32};

// ═══════════════════════════════════════════════════════════════════════════
// Input
// ═══════════════════════════════════════════════════════════════════════════

/// Input for one menu level.
#[derive(Clone)]
pub struct MenuInput {
    /// The complete menu tree (shared by every level).
    pub menu: Arc<Menu>,
    /// What this level should display, and where it wants to be.
    pub request: MenuShowRequest,
}

impl MenuInput {
    pub fn depth(&self) -> usize {
        self.request.depth
    }
}

impl PartialEq for MenuInput {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.menu, &other.menu) && self.request == other.request
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Component
// ═══════════════════════════════════════════════════════════════════════════

pub struct MenuWindow {
    input: MenuInput,
    /// How this window draws its rows. Owned here, not by `rcm_core`.
    style: MenuStyle,
    hovered: Option<usize>,
    /// Content size in DIPs once the renderer has reported it.
    measured: Option<(f64, f64, f64)>,
    /// Height in DIPs to use until the renderer reports a real one.
    estimated_height: f64,
    attach_timer: Option<ComponentTimer>,
    /// Measures the rendered menu so the controller can clamp against it.
    measurer: ElementRef<Grid>,
    /// Whether the controller has been told about this window.
    attached: bool,
}

#[derive(Clone)]
pub enum Message {
    /// Ask Reactor for the native handle (the only thing `run_window` is for).
    Attach,
    /// The native handle arrived: register the window and place it.
    Attached { raw: isize },
    /// The renderer reported the laid-out content size (DIPs) and DPI scale.
    Measured(f64, f64, f64),
    /// Pointer entered row `usize`.
    Hover(usize),
    /// Pointer left the row it was on — clears the highlight.
    HoverOut,
    /// Row `usize` was activated (click).
    Activate(usize),
    /// A ribbon item at `usize` with children was activated.
    ShowSubmenu(usize),
    /// Run a command.
    Execute(CommandPayload),
    /// Close every menu popup.
    CloseAll,
}

impl MenuWindow {
    /// The level this window renders.
    fn level(&self) -> &MenuLevel {
        &self.input.request.level
    }

    fn ribbon_visible(&self) -> bool {
        self.level().ribbon_visible
    }

    /// DPI scale reported by the measurer (defaults to 1 before measurement).
    fn scale(&self) -> f64 {
        match self.measured {
            Some((_, _, scale)) if scale > 0.0 => scale,
            _ => 1.0,
        }
    }

    /// Content size in DIPs: measured when available, else the estimate.
    fn content_size(&self) -> (f64, f64) {
        match self.measured {
            Some((w, h, _)) if w > 0.0 && h > 0.0 => (w, h),
            _ => (self.style.fallback_width, self.estimated_height),
        }
    }

    /// The item at `index`, unless it is a separator or disabled.
    ///
    /// Rows that cannot be interacted with are silently ignored, which is what
    /// both hover and click need.
    fn active_item(&self, index: usize) -> Option<(&Item, Vec<i32>)> {
        let MenuRow::Item { item, path } = self.level().row(index)? else {
            return None;
        };
        (!item.disable).then(|| (item.as_ref(), path.clone()))
    }

    /// The measurement to hand to the controller, in physical pixels.
    ///
    /// Reactor sizes its window exactly to the content, so the window and the
    /// content are the same size and there is no padding offset.
    fn measurement(&self) -> Measurement {
        let scale = self.scale();
        let (dip_w, dip_h) = self.content_size();
        let size = rcm_core::ui::Size::new(
            (dip_w * scale).round() as i32,
            (dip_h * scale).round() as i32,
        );
        Measurement::exact(size)
    }

    /// Report a pointer entering row `index` to the controller.
    ///
    /// The controller decides whether that opens a submenu; this only creates
    /// the window it asks for. Activating a parent row reuses this path.
    fn enter_row(&self, context: &ComponentContext<Self>, index: usize, path: Vec<i32>) {
        let info = HoverInfo {
            depth: self.input.depth(),
            path,
            index,
            item_y: Some(self.item_offset(index)),
        };

        let HoverResult::Show(request) = menu_runtime::hover(&info) else {
            return;
        };

        let child = MenuInput {
            menu: self.input.menu.clone(),
            request: *request,
        };
        if !context.open_window(View::component::<MenuWindow>(child)) {
            log::warn("Rust::menu", "open_window rejected (no active publication)");
        }
    }

    /// Physical-pixel offset of row `index` from the top of the content.
    fn item_offset(&self, index: usize) -> i32 {
        (row_offset(self.level(), index, &self.style) * self.scale()).round() as i32
    }

    /// Ask the controller to position this window, and log the outcome.
    ///
    /// `reason` identifies which event triggered the placement, so a missing or
    /// stale placement is visible in the log.
    fn place(&self, reason: &str) {
        let depth = self.input.depth();
        match menu_runtime::place(depth, self.measurement()) {
            Some(rect) => log::event("Rust::menu", "placed", &format!("{reason} rect={rect}")),
            None => log::warn("Rust::menu", &format!("{reason} — placement refused")),
        }
    }

    /// Run a command and apply the shared close-after-execute policy.
    fn run(&self, cmd: CommandPayload) {
        exec::execute(cmd.clone());
        menu_runtime::finish_execute(&cmd);
    }
}

impl Component for MenuWindow {
    type Message = Message;
    type Input = MenuInput;

    fn create(input: &Self::Input, context: &ComponentContext<Self>) -> Self {
        let mut window = Self {
            input: input.clone(),
            style: MENU_STYLE,
            hovered: None,
            measured: None,
            estimated_height: level_height(&input.request.level, &MENU_STYLE),
            attach_timer: None,
            measurer: ElementRef::new(),
            attached: false,
        };
        // The timer keeps itself alive until it fires; dropping it cancels it.
        window.attach_timer = context
            .set_timeout(std::time::Duration::from_millis(1), Message::Attach)
            .ok();
        window
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Attach => {
                // `run_window` is used for exactly one thing: learning the
                // native handle. Reactor accepts at most one pending window
                // operation at a time, so issuing a second one for placement
                // could be silently discarded — that is what left windows
                // unconfigured (title bar visible, stuck at the origin). Once
                // the handle is known, everything else is a plain call.
                //
                // The window is prepared *inside* this closure, because Reactor
                // shows a window as soon as it exists: stripping the chrome and
                // hiding it any later leaves a brief flash of a normal window at
                // the OS default position.
                let size = self.measurement().window;
                let _ = context.run_window(move |handle| {
                    let raw = handle.as_raw() as isize;
                    menu_runtime::prepare_popup(raw, size);
                    Message::Attached { raw }
                });
                self.attach_timer = None;
            }
            Message::Attached { raw } => {
                let depth = self.input.depth();

                if !menu_runtime::adopt(depth, raw, self.scale()) {
                    // The level was dismissed while this window was being
                    // created. Close it: Reactor shows windows as soon as they
                    // exist, so an unregistered one would linger on screen with
                    // a normal title bar, at the OS default position.
                    log::warn(
                        "Rust::menu",
                        &format!("depth={depth} adopt rejected — closing orphan window"),
                    );
                    win32::post_close(raw as *mut core::ffi::c_void);
                    return;
                }

                self.attached = true;
                self.place(&format!("depth={} attached", self.input.depth()));
            }
            Message::Measured(width, height, scale) => {
                // Only the first measurement is used. Applying it changes the
                // window size, which produces another `Metrics` event; ignoring
                // that keeps the menu from oscillating.
                if self.measured.is_some() || width <= 0.0 || height <= 0.0 {
                    return;
                }
                log::event(
                    "Rust::menu",
                    "measured",
                    &format!(
                        "depth={} {:.0}x{:.0} @{:.2}x",
                        self.input.depth(),
                        width,
                        height,
                        scale
                    ),
                );
                self.measured = Some((width, height, scale));

                // Keep the host's DPI in step with the real value: it is
                // adopted before the first measurement, so without this it
                // would report the 1.0 default forever.
                menu_runtime::set_scale(scale);

                // `view` publishes the measured `client_size`, so the window
                // resizes on this commit; re-clamp with the real size. If the
                // handle has not arrived yet, `Attached` will place instead.
                if self.attached {
                    self.place(&format!("depth={} measured", self.input.depth()));
                }
            }
            Message::Hover(index) => {
                if self.hovered == Some(index) {
                    return;
                }
                self.hovered = Some(index);

                if let Some((_, path)) = self.active_item(index) {
                    self.enter_row(context, index, path);
                }
            }
            Message::HoverOut => {
                self.hovered = None;
            }
            Message::Activate(index) => {
                let Some((item, path)) = self.active_item(index) else {
                    return;
                };
                if item.has_children() {
                    self.enter_row(context, index, path);
                } else if let Some(cmd) = &item.command {
                    self.run(cmd.clone());
                }
            }
            Message::ShowSubmenu(index) => {
                // Ribbon buttons align their submenu with the ribbon itself.
                self.enter_row(context, index, vec![-1, index as i32]);
            }
            Message::Execute(cmd) => {
                self.run(cmd);
            }
            Message::CloseAll => {
                menu_runtime::hide_all();
            }
        }
    }

    fn view(&self, _input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        // Reactor has no general measurement API, but `observe_composition_host`
        // on an `ElementRef<Grid>` reports the bound grid's real
        // `ActualWidth`/`ActualHeight` and re-reports on every WinUI
        // `SizeChanged`. Binding the popup's root grid gives us the laid-out
        // content size, which the controller then clamps against.
        let measurer = self.measurer.clone();
        let sender = context.sender();
        context.use_effect_guard("rcm-menu-measure", (), move || {
            measurer.observe_composition_host(move |event| {
                let size = match event {
                    CompositionHostEvent::Ready {
                        width,
                        height,
                        scale,
                        ..
                    }
                    | CompositionHostEvent::Metrics {
                        width,
                        height,
                        scale,
                    } => Some((width, height, scale)),
                };
                if let Some((width, height, scale)) = size {
                    let _ = sender.send(Message::Measured(width, height, scale));
                }
            })
        });

        let (dip_w, dip_h) = self.content_size();
        context.window_title(MENU_TITLE);
        context.window_visuals(
            WindowVisuals::new()
                .theme(visuals::window_theme())
                .backdrop(visuals::WINDOW_BACKDROP)
                .client_size(dip_w, dip_h),
        );

        let mut children: Vec<KeyedView> = Vec::new();

        // ── Icon ribbon (root only) ──────────────────────────────────
        if self.ribbon_visible() {
            let mut buttons: Vec<KeyedView> = Vec::new();
            for (idx, item) in self.level().ribbon.iter().enumerate() {
                let text = if item.icon.is_empty() {
                    item.key.clone()
                } else {
                    item.icon.clone()
                };
                let mut button = Button::new().is_enabled(!item.disable);
                if !item.disable {
                    if let Some(cmd) = &item.command {
                        button = button.on_click(context.message(Message::Execute(cmd.clone())));
                    } else if item.has_children() {
                        button = button.on_click(context.message(Message::ShowSubmenu(idx)));
                    }
                }
                let button = button.content(TextBlock::new().text(text));
                buttons.push(KeyedView::new(format!("ribbon-{idx}"), button));
            }
            children.push(KeyedView::new(
                "ribbon",
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(4.0)
                    .keyed_children(buttons),
            ));
            children.push(KeyedView::new(
                "ribbon-sep",
                separator_view(self.style.separator_height),
            ));
        }

        // ── Rows ─────────────────────────────────────────────────────
        for (index, row) in self.level().rows.iter().enumerate() {
            match row {
                MenuRow::Separator => children.push(KeyedView::new(
                    format!("sep-{index}"),
                    separator_view(self.style.separator_height),
                )),
                MenuRow::Item { item, .. } => {
                    let key = if item.key.is_empty() {
                        format!("row-{index}")
                    } else {
                        item.key.clone()
                    };
                    children.push(KeyedView::new(key, self.row_view(index, item, context)));
                }
            }
        }

        // The root grid sizes to its content (`Left`/`Top` alignment plus a
        // width clamp) so the measured size is the menu's natural size rather
        // than the window's. `view` then publishes that size as `client_size`.
        Grid::new()
            .element_ref(&self.measurer)
            .horizontal_alignment(HorizontalAlignment::Left)
            .vertical_alignment(VerticalAlignment::Top)
            .min_width(self.style.min_width)
            .max_width(self.style.max_width)
            .keyed_children([KeyedView::new(
                "body",
                Border::new()
                    .background(ThemeBrush::CardBackground)
                    .border_brush(ThemeBrush::CardStroke)
                    .border_thickness(1.0)
                    .corner_radius(self.style.corner_radius)
                    .padding(self.style.padding)
                    .is_tab_stop(true)
                    .allow_focus_on_interaction(true)
                    .on_preview_key_down(context.routed_callback(|event: KeyEventInfo| {
                        if event.key == VirtualKey::ESCAPE {
                            RoutedMessage::handled(Message::CloseAll)
                        } else {
                            RoutedMessage::bubble_without_message()
                        }
                    }))
                    .content(StackPanel::new().spacing(0.0).keyed_children(children)),
            )])
    }
}

impl MenuWindow {
    /// Build a single interactive menu row.
    fn row_view(&self, index: usize, item: &Item, context: &mut ViewContext<Self>) -> View {
        let label = if item.label.is_empty() {
            item.key.clone()
        } else {
            item.label.clone()
        };
        let has_children = item.has_children();

        let icon_text = if self.level().icons && !item.icon.is_empty() {
            item.icon.clone()
        } else {
            String::new()
        };
        let arrow = if has_children { "\u{25B8}" } else { "" };

        // Only reserve the icon gutter when this menu actually renders icons
        // (decided per level by `MenuLevel::icons`), because the icon ribbon is
        // off by default and the reserved strip would be dead space.
        //
        // The label column is `Auto` and an empty `Star` column sits before the
        // arrow. `Auto` keeps the grid's desired width equal to its natural
        // content width — which is exactly what the measurer reads — while the
        // `Star` spacer absorbs any slack once the window has been sized to
        // that width, pinning the arrow to the right edge.
        let grid = if self.level().icons {
            Grid::new()
                .columns([
                    GridLength::Pixel(self.style.icon_width),
                    GridLength::Auto,
                    GridLength::Star(1.0),
                    GridLength::Auto,
                ])
                .keyed_children([
                    KeyedView::new("icon", TextBlock::new().text(icon_text).grid_column(0)),
                    KeyedView::new("label", self.label_view(label).grid_column(1)),
                    KeyedView::new("arrow", TextBlock::new().text(arrow).grid_column(3)),
                ])
        } else {
            Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
                .keyed_children([
                    KeyedView::new("label", self.label_view(label).grid_column(0)),
                    KeyedView::new("arrow", TextBlock::new().text(arrow).grid_column(2)),
                ])
        };

        // A non-null background is required for the row to take part in WinUI
        // hit testing, so every row gets a transparent one; the row under the
        // pointer switches to a subtle fill so the user can tell which item a
        // click will activate.
        let highlighted = self.hovered == Some(index) && !item.disable;
        let background = if highlighted {
            let (a, r, g, b) = MENU_HOVER_ARGB;
            Brush::Solid(Color::argb(a, r, g, b))
        } else {
            Brush::Solid(Color::transparent())
        };

        let (pad_left, pad_right) = self.style.row_padding;
        let mut row = Border::new()
            .background(background)
            .corner_radius(self.style.row_corner_radius)
            .padding(Thickness::new(pad_left, 0.0, pad_right, 0.0))
            .height(self.style.row_height);

        if item.disable {
            row = row.opacity(0.45);
        } else {
            row = row
                .on_pointer_entered(
                    context.callback(move |_: PointerEventInfo| Message::Hover(index)),
                )
                .on_pointer_exited(context.callback(|_: PointerEventInfo| Message::HoverOut))
                .on_pointer_released(
                    context.callback(move |_: PointerEventInfo| Message::Activate(index)),
                );
        }

        row.content(grid)
    }

    /// A menu row's label text.
    ///
    /// The width is capped so a very long label ellipsizes instead of
    /// stretching the menu past `max_width` (which would also make the popup
    /// wider than the region the position clamp assumes).
    fn label_view(&self, label: String) -> TextBlock {
        let m = &self.style;
        let reserve = m.icon_width + m.padding * 2.0 + m.arrow_gutter;
        TextBlock::new()
            .text(label)
            .max_lines(1)
            .text_wrapping(TextWrapping::NoWrap)
            .text_trimming(TextTrimming::CharacterEllipsis)
            .max_width((m.max_width - reserve).max(96.0))
    }
}

/// A horizontal separator line.
fn separator_view(height: f64) -> View {
    Border::new()
        .height(1.0)
        .background(ThemeBrush::CardStroke)
        .margin(Thickness::new(
            0.0,
            (height - 1.0) / 2.0,
            0.0,
            (height - 1.0) / 2.0,
        ))
        .content(TextBlock::new().text(""))
}
