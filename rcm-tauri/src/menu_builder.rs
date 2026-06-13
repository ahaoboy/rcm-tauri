// ═══════════════════════════════════════════════════════════════════════════
// Menu builders — generate a Menu from context data (desktop or file info).
// ═══════════════════════════════════════════════════════════════════════════

use rcm_core::{FileInfo, InvokeProps, Menu};
use rcm_core::{clipboard, lang};

/// Build a menu from a blank desktop context (no files selected).
pub fn rcm() -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());
    let props = InvokeProps {
        files: vec![],
        cwd: "C:\\".to_string(),
        env,
        admin: false,
        type_name: "Desktop".to_string(),
        lang: lang::system_lang(),
        clipboard: clipboard::detect(),
    };

    rcm_vm::invoke(&props)
}

/// Build a menu from real right-click context data received via the pipe.
pub fn rcm_from_info(
    info: &rcm_com::ContextMenuInfo,
) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = std::collections::HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());

    let files: Vec<FileInfo> = info
        .files
        .iter()
        .map(|path| {
            let p = std::path::Path::new(path);
            FileInfo {
                name: p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                path: path.clone(),
                is_dir: p.is_dir(),
            }
        })
        .collect();

    let props = InvokeProps {
        files,
        cwd: info.dir.clone(),
        env,
        admin: false,
        type_name: if info.bg {
            "Background".to_string()
        } else {
            "File".to_string()
        },
        lang: lang::system_lang(),
        clipboard: clipboard::detect(),
    };

    rcm_vm::invoke(&props)
}
