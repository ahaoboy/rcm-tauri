//! `@add-to-quick-access` / `@remove-from-quick-access` — Add or remove
//! a file/folder from the Windows Quick Access pane in File Explorer.
//!
//! Uses PowerShell + Shell.Application COM to invoke the `pintohome` /
//! `unpinfromhome` shell verbs.

use super::SystemCmdResult;
use crate::types::CommandPayload;

/// Run `@add-to-quick-access` — pin a file/folder to Quick Access.
pub fn run_add(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No path specified".into(),
            };
        }
    };

    crate::log::info(
        "Rust::add_to_quick_access",
        &format!("adding '{path}' to Quick Access"),
    );

    // Get-Item → Namespace(parent).ParseName(name) → InvokeVerb.
    // This two-step approach (like open_with.rs) is more robust than
    // Namespace(fullPath) which can throw COMException E_FAIL for
    // paths with non-ASCII characters (e.g. Chinese folder names).
    //
    // For drive roots (C:\) where DirectoryName is null, fall back to
    // Namespace(root).Self — drive letters are ASCII so it won't fail.
    let is_drive_root = path.len() == 3 && path.ends_with(":\\");
    let script = if is_drive_root {
        format!(
            r#"[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;(New-Object -ComObject Shell.Application).Namespace('{path}').Self.InvokeVerb('pintohome')"#
        )
    } else {
        format!(
            r#"[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;$f=gi -LiteralPath '{path}';(New-Object -ComObject Shell.Application).Namespace($f.DirectoryName).ParseName($f.Name).InvokeVerb('pintohome')"#
        )
    };

    match crate::sys_cmd("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
    {
        Ok(output) if output.status.success() => {
            crate::log::info("Rust::add_to_quick_access", "pintohome OK");
            SystemCmdResult {
                success: true,
                message: format!("Added to Quick Access: {path}"),
            }
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let msg = if err.is_empty() {
                "powershell exited non-zero".into()
            } else {
                err
            };
            crate::log::error("Rust::add_to_quick_access", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
        Err(e) => {
            let msg = format!("failed to spawn powershell: {e}");
            crate::log::error("Rust::add_to_quick_access", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}

/// Run `@remove-from-quick-access` — unpin a file/folder from Quick Access.
pub fn run_remove(cmd: &CommandPayload) -> SystemCmdResult {
    let path = match cmd.args.first() {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => {
            return SystemCmdResult {
                success: false,
                message: "No path specified".into(),
            };
        }
    };

    crate::log::info(
        "Rust::remove_from_quick_access",
        &format!("removing '{path}' from Quick Access"),
    );

    // Use Shell.Application → navigate to Quick Access folder →
    // find the item by path → InvokeVerb("unpinfromhome").
    let script = format!(
        r#"[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;$shell=New-Object -ComObject Shell.Application;$qa=$shell.Namespace('shell:::{{679f85cb-0220-4080-b29b-5540cc05aab6}}');$items=$qa.Items();for($i=0;$i -lt $items.Count;$i++){{if($items.Item($i).Path -eq '{path}'){{$items.Item($i).InvokeVerb('unpinfromhome');break}}}}"#
    );

    match crate::sys_cmd("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
    {
        Ok(output) if output.status.success() => {
            crate::log::info("Rust::remove_from_quick_access", "unpinfromhome OK");
            SystemCmdResult {
                success: true,
                message: format!("Removed from Quick Access: {path}"),
            }
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let msg = if err.is_empty() {
                "powershell exited non-zero".into()
            } else {
                err
            };
            crate::log::error("Rust::remove_from_quick_access", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
        Err(e) => {
            let msg = format!("failed to spawn powershell: {e}");
            crate::log::error("Rust::remove_from_quick_access", &msg);
            SystemCmdResult {
                success: false,
                message: msg,
            }
        }
    }
}

/// Check whether a file or folder is currently pinned to Quick Access.
///
/// Uses PowerShell + Shell.Application COM to query the Quick Access
/// namespace. This is synchronous and may take 200-500ms.
pub fn is_in_quick_access(path: &str) -> bool {
    let script = format!(
        r#"[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;$qa=(New-Object -ComObject Shell.Application).Namespace('shell:::{{679f85cb-0220-4080-b29b-5540cc05aab6}}');($qa.Items()|Where-Object{{$_.Path -eq '{path}'}}).Count -gt 0"#
    );

    match crate::sys_cmd("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
            stdout.contains("true")
        }
        _ => false,
    }
}

/// List all paths currently pinned to Quick Access.
///
/// Uses PowerShell + Shell.Application COM to enumerate the Quick Access
/// namespace. Returns the canonical file-system path of each item.
/// This is synchronous and may take 200-500ms.
pub fn list_quick_access() -> Vec<String> {
    let script = r#"[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;$qa=(New-Object -ComObject Shell.Application).Namespace('shell:::{679f85cb-0220-4080-b29b-5540cc05aab6}');$qa.Items()|ForEach-Object{$_.Path}"#;

    match crate::sys_cmd("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect()
        }
        _ => Vec::new(),
    }
}
