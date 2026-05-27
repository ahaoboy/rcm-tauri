use std::os::windows::process::CommandExt;
use std::process::Command;
use std::process::Stdio;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;
use winreg::RegKey;
use winreg::enums::*;

const REG_KEY_POLICIES_EXPLORER: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer";
const REG_VAL_NO_VIEW_CONTEXT_MENU: &str = "NoViewContextMenu";

const REG_KEY_RUN: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
const REG_VAL_RCM: &str = "rcm-tauri";

fn get_hkcu() -> RegKey {
    RegKey::predef(HKEY_CURRENT_USER)
}

pub fn get_context_menu_status() -> bool {
    match get_hkcu().open_subkey(REG_KEY_POLICIES_EXPLORER) {
        Ok(subkey) => {
            let val: u32 = subkey.get_value(REG_VAL_NO_VIEW_CONTEXT_MENU).unwrap_or(0);
            val == 0
        }
        Err(_) => true, // By default, menu is enabled
    }
}

pub fn enable_context_menu() -> Result<(), std::io::Error> {
    let (subkey, _) = get_hkcu().create_subkey(REG_KEY_POLICIES_EXPLORER)?;
    subkey.set_value(REG_VAL_NO_VIEW_CONTEXT_MENU, &0u32)
}

pub fn disable_context_menu() -> Result<(), std::io::Error> {
    let (subkey, _) = get_hkcu().create_subkey(REG_KEY_POLICIES_EXPLORER)?;
    subkey.set_value(REG_VAL_NO_VIEW_CONTEXT_MENU, &1u32)
}

pub fn restart_explorer() {
    std::thread::spawn(|| {
        let _ = Command::new("taskkill")
            .creation_flags(CREATE_NO_WINDOW.0)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("/f")
            .arg("/im")
            .arg("explorer.exe")
            .spawn();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let _ = Command::new("explorer.exe").spawn();
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// Autostart (HKCU\...\Run)
// ═══════════════════════════════════════════════════════════════════════════

/// Check whether the app is registered to run at startup.
pub fn is_autostart_enabled() -> bool {
    get_hkcu()
        .open_subkey(REG_KEY_RUN)
        .ok()
        .and_then(|key| key.get_value::<String, _>(REG_VAL_RCM).ok())
        .is_some()
}

/// Add the current executable to the user's startup registry key.
pub fn enable_autostart() -> Result<(), std::io::Error> {
    let exe = std::env::current_exe()?;
    let (key, _) = get_hkcu().create_subkey(REG_KEY_RUN)?;
    key.set_value(REG_VAL_RCM, &exe.to_string_lossy().to_string())
}

/// Remove the app from the user's startup registry key.
pub fn disable_autostart() -> Result<(), std::io::Error> {
    let key = get_hkcu().open_subkey_with_flags(REG_KEY_RUN, KEY_SET_VALUE)?;
    key.delete_value(REG_VAL_RCM)
}
