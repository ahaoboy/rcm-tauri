//! Minimal Win32 helpers used to turn Reactor's plain top-level windows into
//! borderless, always-on-top popup windows positioned at the cursor.
//!
//! Reactor's declarative API has no notion of window position, visibility or
//! chrome, so the few operations the RCM context menu needs are done directly
//! with `user32`. Everything here is a thin, `unsafe` wrapper — the rest of the
//! application never touches raw handles.

use rcm_core::ui::Rect;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GWL_EXSTYLE, GWL_STYLE, GetForegroundWindow, GetWindowLongPtrW,
    GetWindowThreadProcessId, HWND_TOPMOST, PostMessageW, SW_HIDE, SWP_FRAMECHANGED,
    SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOSIZE, SWP_SHOWWINDOW, SetForegroundWindow,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, WM_CLOSE, WS_CAPTION, WS_EX_APPWINDOW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
};
use windows::core::BOOL;

/// Reinterpret a raw `HWND` pointer (as returned by
/// [`windows_reactor::WindowHandle::as_raw`]) as a typed handle.
#[inline]
fn hwnd(raw: *mut core::ffi::c_void) -> HWND {
    HWND(raw)
}

/// Screen-space rectangle of a monitor's usable work area.
#[derive(Debug, Clone, Copy)]
pub struct WorkArea {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl From<WorkArea> for rcm_core::ui::Rect {
    fn from(area: WorkArea) -> Self {
        rcm_core::ui::Rect::new(
            area.left,
            area.top,
            area.right - area.left,
            area.bottom - area.top,
        )
    }
}

/// Work areas of every monitor, in physical pixels.
///
/// Backs [`rcm_core::ui::MenuHost::work_areas`], which the shared clamp/flip
/// algorithm uses to pick a monitor and keep the menu on screen.
pub fn list_work_areas() -> Vec<Rect> {
    let mut areas: Vec<Rect> = Vec::new();
    let collected = &mut areas as *mut Vec<Rect>;

    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(enum_monitor), LPARAM(collected as isize));
    }

    if areas.is_empty() {
        // Never return an empty list: with no monitors the shared algorithm
        // would skip clamping entirely and could place a menu off-screen.
        areas.push(Rect::from(default_work_area()));
    }
    areas
}

/// `EnumDisplayMonitors` callback — collects each monitor's work area.
unsafe extern "system" fn enum_monitor(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    // SAFETY: `data` is the `*mut Vec<Rect>` passed to `EnumDisplayMonitors`,
    // which stays alive for the duration of the call.
    let areas = unsafe { &mut *(data.0 as *mut Vec<Rect>) };
    let area = unsafe { work_area_of(monitor) };
    areas.push(Rect::from(area));
    BOOL(1) // continue enumeration
}

/// Read one monitor's work area.
unsafe fn work_area_of(monitor: HMONITOR) -> WorkArea {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT::default(),
        rcWork: RECT::default(),
        dwFlags: 0,
    };
    if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        WorkArea {
            left: info.rcWork.left,
            top: info.rcWork.top,
            right: info.rcWork.right,
            bottom: info.rcWork.bottom,
        }
    } else {
        default_work_area()
    }
}

/// Primary-monitor fallback used when a query fails.
fn default_work_area() -> WorkArea {
    WorkArea {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    }
}

/// Turn a freshly created window into a borderless, hidden popup.
///
/// Call this **as early as possible** — ideally the moment the handle is known.
/// Reactor shows a window as soon as it exists, at the OS default position and
/// with normal chrome, so every millisecond before this runs is a visible flash
/// of a misplaced window with a title bar.
///
/// - removes the caption / resize chrome and makes it a popup,
/// - marks it as a tool window (no taskbar / Alt-Tab entry),
/// - hides it,
/// - and sets it to `w`×`h` **plus pins it above other windows**.
///
/// The size must be applied *here*, together with the style change. Making the
/// window a `WS_POPUP` increases its client area by the border it just lost, so
/// a window left at its pre-chrome outer size would show the content with a
/// gap along the right and bottom edges. Setting the size in the same
/// `SetWindowPos` call keeps outer == client == the size the content needs.
pub fn prepare_popup(raw: *mut core::ffi::c_void, w: i32, h: i32) {
    unsafe {
        let handle = hwnd(raw);

        let style = GetWindowLongPtrW(handle, GWL_STYLE) as u32;
        let style = (style & !(WS_CAPTION.0 | WS_THICKFRAME.0 | WS_SYSMENU.0)) | WS_POPUP.0;
        SetWindowLongPtrW(handle, GWL_STYLE, style as isize);

        let ex_style = GetWindowLongPtrW(handle, GWL_EXSTYLE) as u32;
        let ex_style = (ex_style & !WS_EX_APPWINDOW.0) | WS_EX_TOOLWINDOW.0 | WS_EX_TOPMOST.0;
        SetWindowLongPtrW(handle, GWL_EXSTYLE, ex_style as isize);

        let _ = SetWindowPos(
            handle,
            Some(HWND_TOPMOST),
            OFF_SCREEN_X,
            OFF_SCREEN_Y,
            w.max(1),
            h.max(1),
            SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_HIDEWINDOW,
        );
    }
}

/// Move a prepared popup to `(x, y)` and reveal it.
///
/// The size is deliberately not touched: [`prepare_popup`] already made the
/// window exactly as large as its content, and the framework re-applies the size
/// when the content is measured.
pub fn place_popup(raw: *mut core::ffi::c_void, x: i32, y: i32) {
    unsafe {
        let _ = SetWindowPos(
            hwnd(raw),
            Some(HWND_TOPMOST),
            x,
            y,
            0,
            0,
            SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOSIZE,
        );
    }
}

/// Off-screen parking position for a popup that is not shown yet.
pub const OFF_SCREEN_X: i32 = -32000;
pub const OFF_SCREEN_Y: i32 = -32000;

/// Force a popup to become the foreground (active) window.
///
/// The right-click is captured inside Explorer by the `rcm_com` shell
/// extension and forwarded to us over a pipe, so our process never received
/// the input event that Windows requires before it will honour a plain
/// `SetForegroundWindow` call. Temporarily attaching our input queue to the
/// current foreground thread lifts that restriction; once active, the popup
/// gets a normal activation/blur cycle, which is what lets the menu dismiss
/// itself when the user clicks somewhere else — exactly like the native menu.
pub fn force_foreground(raw: *mut core::ffi::c_void) {
    let handle = hwnd(raw);
    if handle.0.is_null() {
        return;
    }

    unsafe {
        let foreground = GetForegroundWindow();
        let our_thread = GetCurrentThreadId();

        // Attach to the current foreground thread so this process is treated
        // as an input owner and is allowed to change the foreground window.
        let attach_to = if foreground.0.is_null() {
            None
        } else {
            let target = GetWindowThreadProcessId(foreground, None);
            if target != 0 && target != our_thread {
                Some(target)
            } else {
                None
            }
        };

        if let Some(target) = attach_to {
            let _ = AttachThreadInput(target, our_thread, true);
        }

        let _ = BringWindowToTop(handle);
        let _ = SetForegroundWindow(handle);

        if let Some(target) = attach_to {
            let _ = AttachThreadInput(target, our_thread, false);
        }
    }
}

/// Move + resize a window without changing its chrome (used to park the hidden
/// root window off-screen).
pub fn move_window(raw: *mut core::ffi::c_void, x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        let _ = SetWindowPos(hwnd(raw), None, x, y, w, h, SWP_NOACTIVATE);
    }
}

/// Hide a window (`SW_HIDE`).
pub fn hide(raw: *mut core::ffi::c_void) {
    unsafe {
        let _ = ShowWindow(hwnd(raw), SW_HIDE);
    }
}

/// Mark a window as a tool window so it never shows in the taskbar.
pub fn mark_tool_window(raw: *mut core::ffi::c_void) {
    unsafe {
        let handle = hwnd(raw);
        let ex_style = GetWindowLongPtrW(handle, GWL_EXSTYLE) as u32;
        let ex_style = (ex_style & !WS_EX_APPWINDOW.0) | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0;
        SetWindowLongPtrW(handle, GWL_EXSTYLE, ex_style as isize);
    }
}

/// Raw handle of the current foreground window (`0` when unavailable).
pub fn foreground_raw() -> *mut core::ffi::c_void {
    unsafe { GetForegroundWindow().0 }
}

/// Ask a window to close by posting `WM_CLOSE`.
pub fn post_close(raw: *mut core::ffi::c_void) {
    if raw.is_null() {
        return;
    }
    unsafe {
        let _ = PostMessageW(Some(hwnd(raw)), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
}
