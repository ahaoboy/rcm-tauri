use std::os::windows::process::CommandExt;
use std::process::Command;
use std::process::Stdio;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;
use winreg::RegKey;
use winreg::enums::*;

const REG_KEY_POLICIES_EXPLORER: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer";
const REG_VAL_NO_VIEW_CONTEXT_MENU: &str = "NoViewContextMenu";

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
