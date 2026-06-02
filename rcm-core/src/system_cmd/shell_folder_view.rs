//! Shared helpers for commands that update a Windows Explorer folder view.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    IServiceProvider,
};
use windows::Win32::System::Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4};
use windows::Win32::UI::Shell::{
    IFolderView2, IShellBrowser, IShellWindows, IWebBrowserApp, SID_STopLevelBrowser, ShellWindows,
};
use windows::core::{GUID, Interface};

const EXPLORER_WAIT_MS: u64 = 2500;

pub(crate) const PKEY_NULL: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x00000000_0000_0000_0000_000000000000),
    pid: 0,
};

pub(crate) const PKEY_ITEM_NAME_DISPLAY: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xb725f130_47ef_101a_a5f1_02608c9eebac),
    pid: 10,
};

pub(crate) const PKEY_ITEM_TYPE_TEXT: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xb725f130_47ef_101a_a5f1_02608c9eebac),
    pid: 4,
};

pub(crate) const PKEY_SIZE: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xb725f130_47ef_101a_a5f1_02608c9eebac),
    pid: 12,
};

pub(crate) const PKEY_DATE_MODIFIED: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xb725f130_47ef_101a_a5f1_02608c9eebac),
    pid: 14,
};

pub(crate) const PKEY_DATE_CREATED: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xb725f130_47ef_101a_a5f1_02608c9eebac),
    pid: 15,
};

struct ComApartment {
    initialized: bool,
}

impl ComApartment {
    fn init() -> Self {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self {
            initialized: hr.is_ok(),
        }
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}

pub(crate) fn property_key_from_arg(arg: &str) -> Option<PROPERTYKEY> {
    match arg {
        "name" => Some(PKEY_ITEM_NAME_DISPLAY),
        "date-modified" => Some(PKEY_DATE_MODIFIED),
        "type" => Some(PKEY_ITEM_TYPE_TEXT),
        "size" => Some(PKEY_SIZE),
        "date-created" => Some(PKEY_DATE_CREATED),
        _ => None,
    }
}

pub(crate) fn target_dir(cwd: &str) -> Result<PathBuf, String> {
    if cwd.trim().is_empty() {
        return Err("No directory specified".into());
    }

    let path = PathBuf::from(cwd);
    if !path.is_dir() {
        return Err(format!("Target is not a directory: {cwd}"));
    }

    path.canonicalize()
        .map_err(|e| format!("Failed to resolve directory '{cwd}': {e}"))
}

pub(crate) fn with_folder_view<F>(dir: &Path, f: F) -> Result<(), String>
where
    F: FnOnce(&IFolderView2) -> windows::core::Result<()>,
{
    let _apartment = ComApartment::init();
    let normalized = normalize_path(dir);

    let shell_windows = unsafe {
        CoCreateInstance::<_, IShellWindows>(&ShellWindows, None, CLSCTX_ALL)
            .map_err(|e| format!("Failed to create ShellWindows COM object: {e}"))?
    };

    let mut opened_by_us = false;
    let browser = match find_browser_for_dir(&shell_windows, &normalized)? {
        Some(browser) => browser,
        None => {
            open_explorer(dir)?;
            opened_by_us = true;
            wait_for_browser(&shell_windows, &normalized)?
                .ok_or_else(|| format!("Explorer did not open folder: {}", dir.display()))?
        }
    };

    let result = folder_view_from_browser(&browser).and_then(|view| f(&view));

    if opened_by_us {
        let _ = unsafe { browser.Quit() };
    }

    result.map_err(|e| format!("Failed to update Explorer folder view: {e}"))
}

fn open_explorer(dir: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    std::process::Command::new("explorer")
        .arg(dir)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to launch Explorer: {e}"))
}

fn wait_for_browser(
    shell_windows: &IShellWindows,
    normalized: &str,
) -> Result<Option<IWebBrowserApp>, String> {
    let deadline = Instant::now() + Duration::from_millis(EXPLORER_WAIT_MS);

    while Instant::now() < deadline {
        if let Some(browser) = find_browser_for_dir(shell_windows, normalized)? {
            return Ok(Some(browser));
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(None)
}

fn find_browser_for_dir(
    shell_windows: &IShellWindows,
    normalized: &str,
) -> Result<Option<IWebBrowserApp>, String> {
    let count = unsafe { shell_windows.Count() }
        .map_err(|e| format!("Failed to enumerate Explorer windows: {e}"))?;

    for i in 0..count {
        let item = unsafe { shell_windows.Item(&variant_i4(i)) };
        let Ok(dispatch) = item else {
            continue;
        };
        let Ok(browser) = dispatch.cast::<IWebBrowserApp>() else {
            continue;
        };
        let Ok(url) = (unsafe { browser.LocationURL() }) else {
            continue;
        };

        if location_url_matches(&url.to_string(), normalized) {
            return Ok(Some(browser));
        }
    }

    Ok(None)
}

fn folder_view_from_browser(browser: &IWebBrowserApp) -> windows::core::Result<IFolderView2> {
    let provider = browser.cast::<IServiceProvider>()?;
    let shell_browser: IShellBrowser = unsafe { provider.QueryService(&SID_STopLevelBrowser) }?;
    let shell_view = unsafe { shell_browser.QueryActiveShellView() }?;
    shell_view.cast::<IFolderView2>()
}

fn variant_i4(value: i32) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: std::mem::ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_I4,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 { lVal: value },
            }),
        },
    }
}

fn location_url_matches(location_url: &str, normalized: &str) -> bool {
    let Some(path) = file_url_to_path(location_url) else {
        return false;
    };

    normalize_path(Path::new(&path)) == normalized
}

fn file_url_to_path(url: &str) -> Option<PathBuf> {
    let lower = url.to_ascii_lowercase();
    let raw = if lower.starts_with("file:///") {
        &url[8..]
    } else if lower.starts_with("file://") {
        &url[7..]
    } else {
        return None;
    };

    let decoded = percent_decode(raw).replace('/', "\\");
    let path = if decoded.starts_with("\\\\") {
        decoded
    } else {
        decoded.trim_start_matches('\\').to_string()
    };

    Some(PathBuf::from(path))
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }

        out.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn normalize_path(path: &Path) -> String {
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    resolved
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_ascii_lowercase()
}
