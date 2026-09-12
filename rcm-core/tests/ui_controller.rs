//! Integration tests for [`MenuController`], driven by a toolkit-free host.
//!
//! This file lives in `tests/` on purpose. An integration test can only use the
//! crate's **public** API, so it doubles as a check on the design goal: that
//! `rcm_core::ui` is enough for an outside frontend to drive a whole menu with
//! no privileged access.
//!
//! If a test here ever needs `pub(crate)`, that is a signal the public surface
//! has a gap for external frontends — not a reason to move the test back.
//!
//! `MockHost` implements [`MenuHost`] with nothing but `u32` window ids and
//! shared state, so no toolkit type appears anywhere. A real frontend (iced,
//! slint, …) has the same shape: it owns the UI, implements `MenuHost`, forwards
//! user input, and lets the controller decide placement and visibility.
//!
//! Run with `cargo test -p rcm-core` (this file is not part of `--lib`).

use std::cell::RefCell;
use std::rc::Rc;

use rcm_core::types::WindowMode;
use rcm_core::ui::{
    HoverInfo, HoverResult, Measurement, MenuController, MenuHost, MenuMetrics, MenuWindowInput,
    Point, Rect, Size,
};
use rcm_core::{CommandPayload, Item, Menu};

// ═══════════════════════════════════════════════════════════════════════════
// Mock host
// ═══════════════════════════════════════════════════════════════════════════

/// Everything the controller can ask of a host, recorded.
#[derive(Default)]
struct HostState {
    /// Monitors reported to the controller.
    areas: Vec<Rect>,
    /// DIP → pixel scale returned by [`MenuHost::scale_factor`].
    scale: f64,
    /// The next window id handed out.
    next_window: u32,
    /// Levels passed to `open_window`, in order.
    opened: Vec<MenuWindowInput>,
    /// Every `place_window` call, in order.
    placed: Vec<(u32, Rect)>,
    /// Windows passed to `hide_window`.
    hidden: Vec<u32>,
    /// Windows passed to `close_window`.
    closed: Vec<u32>,
    /// Windows passed to `focus_window`, in order.
    focused: Vec<u32>,
    /// What [`MenuHost::row_offset`] should answer: a per-row height and a top
    /// padding, or `None` to model a host that cannot estimate.
    row_estimate: Option<(f64, f64)>,
}

/// A host that records everything the controller asks for.
#[derive(Clone, Default)]
struct MockHost {
    state: Rc<RefCell<HostState>>,
}

impl MockHost {
    /// A host with one 1920×1080 monitor at the origin and scale 1.
    fn new() -> Self {
        let host = Self::default();
        let mut state = host.state.borrow_mut();
        state.areas = vec![Rect::new(0, 0, 1920, 1080)];
        state.scale = 1.0;
        state.next_window = 1;
        drop(state);
        host
    }
}

impl MenuHost for MockHost {
    type Window = u32;

    /// The mock *can* create windows from outside a render callback — the
    /// Tauri-shaped host. Reactor reports `None` here and uses `adopt` instead.
    fn open_window(&mut self, input: &MenuWindowInput) -> Option<Self::Window> {
        let mut state = self.state.borrow_mut();
        state.opened.push(input.clone());
        let id = state.next_window;
        state.next_window += 1;
        Some(id)
    }

    fn place_window(&mut self, window: Self::Window, _input: &MenuWindowInput, rect: Rect) {
        self.state.borrow_mut().placed.push((window, rect));
    }

    fn hide_window(&mut self, window: Self::Window) {
        self.state.borrow_mut().hidden.push(window);
    }

    fn close_window(&mut self, window: Self::Window) {
        self.state.borrow_mut().closed.push(window);
    }

    fn focus_window(&mut self, window: Self::Window) {
        self.state.borrow_mut().focused.push(window);
    }

    fn work_areas(&self) -> Vec<Rect> {
        self.state.borrow().areas.clone()
    }

    fn scale_factor(&self, _window: Self::Window) -> f64 {
        self.state.borrow().scale
    }

    /// Models a frontend whose rows are a fixed height: `padding + index * row`.
    fn row_offset(&self, level: &rcm_core::ui::MenuLevel, index: usize) -> Option<f64> {
        let (padding, row_height) = self.state.borrow().row_estimate?;
        let rows = level.rows.iter().take(index).count() as f64;
        Some(padding + rows * row_height)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Fixtures
// ═══════════════════════════════════════════════════════════════════════════

/// A leaf item.
fn leaf(key: &str) -> Item {
    Item {
        key: key.to_string(),
        icon: String::new(),
        label: key.to_string(),
        disable: false,
        admin: false,
        window: WindowMode::default(),
        items: Vec::new(),
        command: None,
    }
}

/// An item with children, i.e. a submenu.
fn parent(key: &str, children: Vec<Item>) -> Item {
    Item {
        items: children,
        ..leaf(key)
    }
}

/// A single-group menu containing `items`.
fn menu(items: Vec<Item>) -> Menu {
    Menu {
        icon_items: Vec::new(),
        groups: vec![Item {
            items,
            ..leaf("group")
        }],
    }
}

/// A controller showing `menu` at `at`, plus a handle to the host's record.
fn showing(menu: Menu, at: Point) -> (MenuController<MockHost>, Rc<RefCell<HostState>>) {
    let host = MockHost::new();
    let state = Rc::clone(&host.state);
    let mut controller = MenuController::new(host, MenuMetrics::default());
    let request = controller.show_root(menu, at);
    controller.open(&request);
    (controller, state)
}

/// A controller showing a menu shaped like the reported case: a parent row with
/// a submenu at `[0, 0]`, followed by a plain leaf row at `[0, 1]`.
///
/// Hovering the leaf closes the submenu — the sequence that used to dismiss the
/// whole menu.
fn showing_parent(at: Point) -> (MenuController<MockHost>, Rc<RefCell<HostState>>) {
    let (mut controller, state) = showing(
        menu(vec![
            parent("b", vec![leaf("b1"), leaf("b2")]),
            leaf("sibling"),
        ]),
        at,
    );
    controller.place(0, Measurement::exact(Size::new(200, 120)));
    (controller, state)
}

/// Hover the row at `path`, deriving the row index from the path's last element.
fn hover_row(
    controller: &mut MenuController<MockHost>,
    depth: usize,
    path: Vec<i32>,
) -> HoverResult {
    let index = path.last().copied().unwrap_or(0).max(0) as usize;
    controller.hover(&HoverInfo {
        depth,
        path,
        index,
        item_y: Some(0),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Opening: what the host receives
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn open_window_receives_the_level_to_draw_and_no_geometry() {
    let (controller, state) = showing(menu(vec![leaf("a")]), Point::new(100, 100));
    let state = state.borrow();

    assert_eq!(state.opened.len(), 1, "exactly one window opened");
    let opened = &state.opened[0];
    assert_eq!(opened.level.depth, 0, "root level");
    assert!(opened.level.path.is_empty(), "root path is empty");
    assert_eq!(opened.level.rows.len(), 1);
    assert!(
        state.placed.is_empty(),
        "opening must not place — that waits for a measurement"
    );
    drop(state);

    assert!(controller.has_levels());
    assert_eq!(controller.deepest(), 0);
}

#[test]
fn show_root_carries_the_menu_tree_for_frontends_that_redraw_it() {
    let (controller, state) = showing(menu(vec![leaf("a")]), Point::new(0, 0));
    let state = state.borrow();

    assert_eq!(state.opened[0].menu.groups.len(), 1);
    assert_eq!(state.opened[0].menu.groups[0].items.len(), 1);
    let _ = controller;
}

// ═══════════════════════════════════════════════════════════════════════════
// Placement
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn place_puts_the_content_on_the_ideal_point() {
    let (mut controller, state) = showing(menu(vec![leaf("a")]), Point::new(100, 100));

    let rect = controller
        .place(0, Measurement::exact(Size::new(200, 120)))
        .expect("placed");

    assert_eq!(rect, Rect::new(100, 100, 200, 120));
    assert_eq!(state.borrow().placed.last().unwrap().1, rect);
}

#[test]
fn place_honours_the_window_padding_offset() {
    // A CSS-padded container: the content is inset inside the window.
    let (mut controller, _state) = showing(menu(vec![leaf("a")]), Point::new(100, 100));

    let measurement = Measurement::new(Size::new(212, 132), Size::new(200, 120), Point::new(6, 6));
    let rect = controller.place(0, measurement).expect("placed");

    // The window origin shifts back so the *content* lands on the point.
    assert_eq!(rect, Rect::new(94, 94, 212, 132));
}

#[test]
fn place_flips_left_when_it_would_overflow_the_right_edge() {
    let (mut controller, _state) = showing(menu(vec![leaf("a")]), Point::new(1900, 100));

    let rect = controller
        .place(0, Measurement::exact(Size::new(200, 120)))
        .expect("placed");

    // The menu's right edge sits at the cursor.
    assert_eq!(rect.right(), 1900);
}

#[test]
fn place_clamps_into_the_work_area() {
    let (mut controller, _state) = showing(menu(vec![leaf("a")]), Point::new(100, 5000));
    let metrics = MenuMetrics::default();

    let rect = controller
        .place(0, Measurement::exact(Size::new(200, 120)))
        .expect("placed");

    assert_eq!(rect.bottom(), 1080 - metrics.edge_gap);
}

#[test]
fn place_ignores_an_invalid_measurement() {
    let (mut controller, _state) = showing(menu(vec![leaf("a")]), Point::new(0, 0));

    assert!(
        controller
            .place(0, Measurement::exact(Size::new(0, 0)))
            .is_none()
    );
    assert!(
        controller
            .place(0, Measurement::exact(Size::new(-5, 10)))
            .is_none()
    );
}

#[test]
fn place_on_an_unknown_level_is_a_no_op() {
    let (mut controller, _state) = showing(menu(vec![leaf("a")]), Point::new(0, 0));

    assert!(
        controller
            .place(7, Measurement::exact(Size::new(10, 10)))
            .is_none()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Hover
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hovering_a_leaf_shows_nothing() {
    let (mut controller, _state) = showing(menu(vec![leaf("a")]), Point::new(100, 100));
    controller.place(0, Measurement::exact(Size::new(200, 120)));

    let result = hover_row(&mut controller, 0, vec![0, 0]);
    assert_eq!(result, HoverResult::Leaf);
    assert_eq!(controller.deepest(), 0);
}

#[test]
fn hovering_a_disabled_parent_shows_nothing() {
    let mut disabled = parent("b", vec![leaf("b1")]);
    disabled.disable = true;

    let (mut controller, _state) = showing(menu(vec![disabled]), Point::new(100, 100));
    controller.place(0, Measurement::exact(Size::new(200, 120)));

    assert_eq!(hover_row(&mut controller, 0, vec![0, 0]), HoverResult::Leaf);
    assert_eq!(controller.deepest(), 0);
}

#[test]
fn adopting_a_new_window_at_a_depth_closes_the_old_one() {
    // Reactor creates a window per menu, so re-showing a level hands back a new
    // handle. The previous window must be closed: it is no longer in the
    // registry, so `hide_all` could never reach it and it would stay visible.
    let mut controller = MenuController::new(MockHost::new(), MenuMetrics::default());
    let request = controller.show_root(menu(vec![leaf("a")]), Point::new(0, 0));
    controller.adopt(request.depth, 1);

    let state = Rc::clone(&controller.host().state);
    assert_eq!(state.borrow().closed.len(), 0, "nothing closed yet");

    // A second adoption at the same depth displaces the first.
    assert!(controller.adopt(request.depth, 2));
    assert_eq!(
        state.borrow().closed.as_slice(),
        &[1],
        "the displaced window was closed"
    );
    assert_eq!(controller.placed_rect(0), None, "the level was reset");

    // The new window is the one that gets placed.
    controller.place(0, Measurement::exact(Size::new(80, 40)));
    let placed = state.borrow().placed.last().copied().expect("placed");
    assert_eq!(placed.0, 2, "the live window is the new one");
}

#[test]
fn re_adopting_the_same_handle_closes_nothing() {
    // A host that reuses one window per depth (Tauri's fixed labels) must not
    // have its window closed out from under it.
    let mut controller = MenuController::new(MockHost::new(), MenuMetrics::default());
    let request = controller.show_root(menu(vec![leaf("a")]), Point::new(0, 0));

    let state = Rc::clone(&controller.host().state);
    assert!(controller.adopt(request.depth, 7));
    assert!(controller.adopt(request.depth, 7));

    assert!(
        state.borrow().closed.is_empty(),
        "the same handle is a re-render, not a replacement"
    );
}

#[test]
fn swooping_down_a_menu_leaves_no_orphan_windows() {
    // Reproduces the reported leak: sweeping the pointer down the root opens a
    // submenu for each parent row in turn. Every superseded submenu window must
    // be closed, or the screen accumulates them.
    let (mut controller, state) = showing(
        menu(vec![
            parent("p1", vec![leaf("p1a")]),
            leaf("plain"),
            parent("p2", vec![leaf("p2a")]),
            parent("p3", vec![leaf("p3a")]),
        ]),
        Point::new(100, 100),
    );
    controller.place(0, Measurement::exact(Size::new(200, 200)));

    // Row 0 is a parent: opens a submenu.
    let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
        panic!("row 0 should open a submenu");
    };
    controller.open(&child);
    let first = controller
        .state()
        .window(1)
        .expect("a submenu window is open at depth 1");

    // Row 1 is a leaf: the submenu closes.
    assert_eq!(hover_row(&mut controller, 0, vec![0, 1]), HoverResult::Leaf);
    assert!(
        controller.state().window(1).is_none(),
        "no window at depth 1"
    );

    // Row 2 is a parent again: a fresh submenu opens.
    let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 2]) else {
        panic!("row 2 should open a submenu");
    };
    controller.open(&child);
    let second = controller
        .state()
        .window(1)
        .expect("a submenu window is open at depth 1");
    assert_ne!(second, first, "a different window was created");

    // Exactly one window per level, and every superseded one was closed.
    assert_eq!(state.borrow().closed.len(), 1, "the first submenu closed");
    assert_eq!(
        controller.state().depths(),
        vec![0, 1],
        "one window per open level, no strays"
    );

    // And `hide_all` reaches everything that is still on screen.
    let closed_before = state.borrow().closed.len();
    controller.hide_all();
    assert_eq!(
        state.borrow().closed.len(),
        closed_before + 2,
        "both the root and the live submenu were closed"
    );
    assert!(!controller.has_levels());
}

#[test]
fn hide_all_reaches_every_window_it_ever_opened() {
    let (mut controller, state) = showing_parent(Point::new(0, 0));

    // Open and supersede a few submenus, as sweeping the pointer would.
    for _ in 0..3 {
        let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
            panic!("expected a child level");
        };
        controller.open(&child);
        controller.place(1, Measurement::exact(Size::new(200, 60)));
        // Move onto the leaf, collapsing back to the root.
        assert_eq!(hover_row(&mut controller, 0, vec![0, 1]), HoverResult::Leaf);
        controller.place(0, Measurement::exact(Size::new(200, 120)));
    }

    controller.hide_all();

    // Two open windows (root + submenu) plus the closed ones; after `hide_all`
    // nothing may remain registered.
    assert!(!controller.has_levels());
    assert!(controller.state().depths().is_empty());
    assert!(
        state.borrow().closed.len() >= 4,
        "every superseded and open window was closed"
    );
}

#[test]
fn closing_a_submenu_hands_focus_back_to_its_parent() {
    // Closing the focused window makes the OS choose a new foreground window,
    // which is not guaranteed to be one of ours. Without handing focus back, the
    // menu reports "focus left" and dismisses itself while the pointer is still
    // on it — the reported symptom was the root vanishing near the end of a sweep.
    let (mut controller, state) = showing_parent(Point::new(100, 100));

    let parent_window = controller.state().window(0).expect("root window");

    let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
        panic!("expected a child level");
    };
    controller.open(&child);
    controller.place(1, Measurement::exact(Size::new(200, 60)));
    let submenu_window = controller.state().window(1).expect("submenu window");

    // Moving onto a leaf closes the submenu.
    assert_eq!(hover_row(&mut controller, 0, vec![0, 1]), HoverResult::Leaf);

    assert_eq!(
        state.borrow().focused.as_slice(),
        &[parent_window],
        "the parent regained focus after the submenu closed"
    );
    assert_ne!(parent_window, submenu_window);
    assert!(controller.has_levels(), "the root is still open");
}

#[test]
fn closing_nothing_does_not_steal_focus() {
    // Repeated hovers on leaves must not re-focus the parent every time.
    let (mut controller, state) = showing_parent(Point::new(100, 100));

    assert_eq!(hover_row(&mut controller, 0, vec![0, 1]), HoverResult::Leaf);
    assert_eq!(hover_row(&mut controller, 0, vec![0, 1]), HoverResult::Leaf);

    assert!(
        state.borrow().focused.is_empty(),
        "no window was closed, so focus was left alone"
    );
}

#[test]
fn hovering_a_parent_positions_the_child_from_the_parents_rect() {
    let (mut controller, _state) = showing_parent(Point::new(100, 100));
    let metrics = MenuMetrics::default();

    let result = controller.hover(&HoverInfo {
        depth: 0,
        path: vec![0, 0],
        index: 0,
        item_y: Some(28),
    });

    let HoverResult::Show(request) = result else {
        panic!("expected a child level, got {result:?}");
    };
    assert_eq!(request.depth, 1);
    assert_eq!(request.path, vec![0, 0]);
    // Right of the parent content, aligned with the hovered row.
    assert_eq!(request.position.x, 100 + 200 + metrics.submenu_gap);
    assert_eq!(request.position.y, 100 + 28);
    assert_eq!(request.parent_left, Some(100), "for left-edge flipping");
}

#[test]
fn hover_falls_back_to_the_hosts_row_estimate() {
    let (mut controller, state) = showing_parent(Point::new(100, 100));
    // The mock knows its rows: 4px of padding then 28px each.
    state.borrow_mut().row_estimate = Some((4.0, 28.0));

    let result = controller.hover(&HoverInfo {
        depth: 0,
        path: vec![0, 0],
        index: 1,
        item_y: None,
    });

    let HoverResult::Show(request) = result else {
        panic!("expected a child level");
    };
    // The parent was placed unclamped at y = 100; row 1 sits 4 + 28px down.
    assert_eq!(request.position.y, 100 + 4 + 28);
}

#[test]
fn hover_without_a_row_estimate_aligns_with_the_parents_top() {
    let (mut controller, _state) = showing_parent(Point::new(100, 100));

    let result = controller.hover(&HoverInfo {
        depth: 0,
        path: vec![0, 0],
        index: 0,
        item_y: None,
    });

    let HoverResult::Show(request) = result else {
        panic!("expected a child level");
    };
    // No measurement and no estimate: the child lines up with the parent's top.
    assert_eq!(request.position.y, 100);
}

#[test]
fn hovering_before_the_parent_is_placed_is_ignored() {
    // Deliberately no `place` call: the controller has no rectangle to anchor to.
    let (mut controller, _state) = showing(
        menu(vec![parent("b", vec![leaf("b1")])]),
        Point::new(100, 100),
    );

    assert_eq!(
        hover_row(&mut controller, 0, vec![0, 0]),
        HoverResult::Ignored
    );
    assert!(controller.has_levels(), "the root is still open");
    assert_eq!(controller.deepest(), 0, "but no child was opened");
}

#[test]
fn hovering_an_unknown_path_is_ignored() {
    let (mut controller, _state) = showing_parent(Point::new(0, 0));

    assert_eq!(
        hover_row(&mut controller, 0, vec![9, 9]),
        HoverResult::Ignored
    );
}

#[test]
fn submenu_flips_left_of_its_parent_at_the_right_edge() {
    // x = 1600: the 200-wide parent still fits (right edge 1800 < 1912), so it
    // is placed where asked. Its child would then overflow, and must flip.
    let (mut controller, _state) = showing_parent(Point::new(1600, 100));
    let metrics = MenuMetrics::default();

    let parent = controller.placed_rect(0).expect("parent was placed");
    assert_eq!(parent.x, 1600, "the parent itself did not need to flip");

    let HoverResult::Show(request) = hover_row(&mut controller, 0, vec![0, 0]) else {
        panic!("expected a child level");
    };
    // Ideal position is right of the parent, which overflows.
    assert!(request.position.x + 200 > 1912);
    controller.open(&request);

    let rect = controller
        .place(1, Measurement::exact(Size::new(200, 120)))
        .expect("placed");

    // Flipped: the child's right edge sits a gap left of the parent's left edge.
    assert_eq!(rect.right(), parent.x - metrics.submenu_gap);
    assert!(rect.x < parent.x, "the child is left of its parent");
}

#[test]
fn nesting_stops_at_the_metrics_limit() {
    let metrics = MenuMetrics::default();

    // Build a chain one level deeper than the limit.
    let mut deepest = leaf("leaf");
    for level in 0..=metrics.max_submenu_depth {
        deepest = parent(&format!("l{level}"), vec![deepest]);
    }

    let mut controller = MenuController::new(MockHost::new(), metrics);
    let request = controller.show_root(menu(vec![deepest]), Point::new(0, 0));
    controller.open(&request);

    // Walk down one row per level.
    let mut path = vec![0, 0];
    for depth in 0..metrics.max_submenu_depth {
        controller.place(depth, Measurement::exact(Size::new(200, 60)));
        let result = hover_row(&mut controller, depth, path.clone());
        let HoverResult::Show(request) = result else {
            panic!("depth {depth} should still open a child, got {result:?}");
        };
        controller.open(&request);
        path.push(0);
    }
    assert_eq!(controller.deepest(), metrics.max_submenu_depth);

    // One more hover must be refused rather than exceeding the limit.
    controller.place(
        metrics.max_submenu_depth,
        Measurement::exact(Size::new(200, 60)),
    );
    assert_eq!(
        hover_row(&mut controller, metrics.max_submenu_depth, path),
        HoverResult::Ignored
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Adopting (the Reactor-shaped host)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn adopt_registers_a_window_without_placing_it() {
    let mut controller = MenuController::new(MockHost::new(), MenuMetrics::default());
    let request = controller.show_root(menu(vec![leaf("a")]), Point::new(10, 10));

    assert!(controller.adopt(request.depth, 42), "the level is open");
    assert!(controller.has_levels());

    let state = Rc::clone(&controller.host().state);
    let state = state.borrow();
    assert!(
        state.opened.is_empty(),
        "adopt does not ask the host to open"
    );
    assert!(state.placed.is_empty(), "adopt does not place");
}

#[test]
fn adopt_refuses_a_dismissed_level() {
    let mut controller = MenuController::new(MockHost::new(), MenuMetrics::default());
    controller.show_root(menu(vec![leaf("a")]), Point::new(10, 10));
    controller.hide_all();

    assert!(
        !controller.adopt(0, 42),
        "a dismissed level must not be re-registered"
    );
    assert!(!controller.has_levels());
}

#[test]
fn adopt_then_place_positions_the_window() {
    let mut controller = MenuController::new(MockHost::new(), MenuMetrics::default());
    let request = controller.show_root(menu(vec![leaf("a")]), Point::new(300, 200));
    controller.adopt(request.depth, 42);

    let rect = controller
        .place(0, Measurement::exact(Size::new(80, 40)))
        .expect("placed");
    assert_eq!(rect, Rect::new(300, 200, 80, 40));
}

// ═══════════════════════════════════════════════════════════════════════════
// Hiding, blur and idle
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hide_deeper_than_closes_only_deeper_windows() {
    let (mut controller, state) = showing_parent(Point::new(0, 0));

    let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
        panic!("expected a child level");
    };
    controller.open(&child);
    controller.place(1, Measurement::exact(Size::new(200, 60)));
    assert_eq!(controller.deepest(), 1);

    controller.hide_deeper_than(0);

    assert_eq!(state.borrow().closed.len(), 1, "the child window closed");
    assert_eq!(controller.deepest(), 0, "the root survives");
    assert!(controller.has_levels());
}

#[test]
fn blur_only_dismisses_when_focus_left_the_menu_entirely() {
    let (mut controller, _state) = showing_parent(Point::new(0, 0));

    let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
        panic!("expected a child level");
    };
    controller.open(&child);
    controller.place(1, Measurement::exact(Size::new(200, 60)));

    // The submenu now holds focus, which is the normal state while it is open.
    assert!(!controller.handle_idle(true), "a menu window still has focus");
    assert!(controller.has_levels());

    // Focus left every menu window: the first miss is still tolerated, because
    // the OS may just be reassigning focus after a level closed.
    assert!(!controller.handle_idle(false), "the first miss is tolerated");
    assert!(controller.has_levels());

    assert!(controller.handle_idle(false), "the second miss dismisses");
    assert!(!controller.has_levels());
}

#[test]
fn a_blur_storm_while_the_pointer_moves_through_the_menu_does_not_dismiss() {
    // Reproduces the reported bug: hovering along the root closes the submenu,
    // which makes the OS emit blurs for windows that are already gone. Those
    // must not dismiss the menu while the user keeps interacting.
    let (mut controller, _state) = showing_parent(Point::new(0, 0));

    let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
        panic!("expected a child level");
    };
    controller.open(&child);
    controller.place(1, Measurement::exact(Size::new(200, 60)));

    // Hover a leaf row: this closes the submenu and hides its window.
    assert_eq!(hover_row(&mut controller, 0, vec![0, 1]), HoverResult::Leaf);

    // The hidden submenu's blur now arrives. Focus has not left the menu.
    assert!(!controller.handle_idle(false), "a stale blur must not dismiss");
    assert!(controller.has_levels(), "the root menu is still open");
    assert_eq!(controller.deepest(), 0);

    // The next hover keeps it alive even if another stale blur lands.
    assert_eq!(hover_row(&mut controller, 0, vec![0, 1]), HoverResult::Leaf);
    assert!(
        !controller.handle_idle(false),
        "interaction restarted the miss counter"
    );
    assert!(controller.has_levels());
}

#[test]
fn the_root_menu_survives_opening_and_closing_submenus() {
    let (mut controller, _state) = showing_parent(Point::new(100, 100));

    for round in 0..3 {
        // Open the submenu.
        let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
            panic!("round {round}: expected a child level");
        };
        controller.open(&child);
        controller.place(1, Measurement::exact(Size::new(200, 60)));
        assert!(controller.has_levels(), "round {round}: submenu opened");

        // Move onto a leaf: the submenu closes, the root stays.
        assert_eq!(hover_row(&mut controller, 0, vec![0, 1]), HoverResult::Leaf);
        assert!(controller.has_levels(), "round {round}: root survived");
        assert_eq!(controller.deepest(), 0, "round {round}: back to the root");
    }
}

#[test]
fn a_submenu_never_dismisses_its_parent() {
    let (mut controller, _state) = showing_parent(Point::new(100, 100));

    let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
        panic!("expected a child level");
    };
    controller.open(&child);
    controller.place(1, Measurement::exact(Size::new(200, 60)));

    // A blur while the parent hands focus to the child, and the child reporting
    // focus, must both leave the menu standing.
    assert!(!controller.handle_idle(false), "the parent's blur is tolerated");
    assert!(!controller.handle_idle(true), "the child holds focus");
    assert!(controller.has_levels());
}

#[test]
fn hide_all_closes_every_window_and_clears_rects() {
    let (mut controller, state) = showing(menu(vec![leaf("a")]), Point::new(0, 0));

    controller.hide_all();

    assert_eq!(state.borrow().closed.len(), 1);
    assert!(!controller.has_levels());
    assert_eq!(controller.deepest(), 0);
    assert!(controller.placed_rect(0).is_none());
}

#[test]
fn idle_dismisses_only_after_focus_was_held_then_lost_twice() {
    let (mut controller, _state) = showing(menu(vec![leaf("a")]), Point::new(0, 0));
    controller.place(0, Measurement::exact(Size::new(200, 60)));

    // Holding focus keeps the menu open and arms the dismiss.
    assert!(!controller.handle_idle(true));
    assert!(controller.has_levels());

    // A single miss is tolerated: a parent hands focus to its submenu.
    assert!(
        !controller.handle_idle(false),
        "the first miss is tolerated"
    );
    assert!(controller.has_levels());

    // A second consecutive miss dismisses.
    assert!(controller.handle_idle(false), "the second miss dismisses");
    assert!(!controller.has_levels());
}

#[test]
fn idle_never_fires_without_a_prior_interaction() {
    let mut controller = MenuController::new(MockHost::new(), MenuMetrics::default());
    assert!(!controller.handle_idle(false));
    assert!(!controller.has_levels());
}

#[test]
fn dev_mode_keeps_the_menu_open() {
    let mut controller = MenuController::new(MockHost::new(), MenuMetrics::default());
    controller.set_dev_mode(true);

    let request = controller.show_root(menu(vec![leaf("a")]), Point::new(0, 0));
    controller.open(&request);
    controller.place(0, Measurement::exact(Size::new(200, 60)));

    assert!(!controller.handle_idle(false));
    assert!(controller.has_levels(), "dev mode suppresses auto-hide");
}

// ═══════════════════════════════════════════════════════════════════════════
// Execute policy
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn execute_closes_the_menu_but_dev_mode_does_not() {
    let command = CommandPayload {
        cmd: "cmd".into(),
        args: Vec::new(),
        cwd: String::new(),
        admin: false,
        window: WindowMode::default(),
    };

    let (mut controller, _state) = showing(menu(vec![leaf("a")]), Point::new(0, 0));
    assert!(controller.finish_execute(&command), "closed");
    assert!(!controller.has_levels());

    controller.set_dev_mode(true);
    let request = controller.show_root(menu(vec![leaf("a")]), Point::new(0, 0));
    controller.open(&request);

    assert!(
        !controller.finish_execute(&command),
        "left open in dev mode"
    );
    assert!(controller.has_levels());
}

// ═══════════════════════════════════════════════════════════════════════════
// Menu lifecycle
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn show_root_replaces_the_previous_menu() {
    let (mut controller, state) = showing_parent(Point::new(0, 0));

    let HoverResult::Show(child) = hover_row(&mut controller, 0, vec![0, 0]) else {
        panic!("expected a child level");
    };
    controller.open(&child);
    controller.place(1, Measurement::exact(Size::new(200, 60)));

    // A second right-click resets everything.
    let request = controller.show_root(menu(vec![leaf("z")]), Point::new(50, 50));
    controller.open(&request);

    assert_eq!(controller.deepest(), 0, "the submenu was closed");
    assert_eq!(state.borrow().closed.len(), 2, "submenu + previous root");
}

#[test]
fn show_root_closes_the_previous_root_window() {
    // Reactor gives every menu its own window, so a new right-click must close
    // the old *root* too. Closing only the deeper levels leaked one window per
    // right-click, and the orphans stayed visible with a title bar.
    let (mut controller, state) = showing(menu(vec![leaf("a")]), Point::new(0, 0));
    assert_eq!(state.borrow().closed.len(), 0, "nothing closed yet");

    let request = controller.show_root(menu(vec![leaf("b")]), Point::new(10, 10));

    assert_eq!(
        state.borrow().closed.len(),
        1,
        "the previous root window was closed"
    );
    assert_eq!(state.borrow().opened.len(), 1, "only the first was opened");

    // The new menu is the only one open.
    controller.adopt(request.depth, 99);
    controller.place(0, Measurement::exact(Size::new(80, 40)));
    assert_eq!(controller.deepest(), 0);
    assert!(controller.has_levels());
}

#[test]
fn the_flattened_level_is_available_without_opening_a_window() {
    let (controller, _state) = showing(menu(vec![leaf("a"), leaf("b")]), Point::new(0, 0));

    let level = controller.level(0).expect("root level");
    assert_eq!(level.rows.len(), 2);
}
