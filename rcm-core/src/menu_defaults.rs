const DEFAULT_MODULE: &str = include_str!("../../rcm-kit/dist/default.js");
const LITE_MODULE: &str = include_str!("../../rcm-kit/dist/lite.js");

/// Write embedded default menu JS files to disk next to the exe.
pub fn write_menu_defaults() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    for (name, src) in [
        ("rcm.lite.js", LITE_MODULE),
        ("rcm.full.js", DEFAULT_MODULE),
    ] {
        let path = exe_dir.join(name);
        if let Err(e) = std::fs::write(&path, src) {
            eprintln!("write_menu_defaults: write {} failed: {e}", path.display());
        } else {
            println!("write_menu_defaults: wrote {}", path.display());
        }
    }
}

/// `name` is `"rcm.lite"` or `"rcm.full"`.  The corresponding `.js`
/// file is looked up next to the executable.  If it exists it is used
/// as-is (allowing user customisation); otherwise the embedded default
/// is written to disk and returned.
pub fn load_menu_module(name: &str) -> String {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let file_path = exe_dir.join(format!("{name}.js"));

    // Already on disk — use it
    if file_path.exists() {
        match std::fs::read_to_string(&file_path) {
            Ok(src) => {
                println!("load_menu_module: using {}.js from disk", name);
                return src;
            }
            Err(e) => eprintln!("load_menu_module: read {} failed: {e}", file_path.display()),
        }
    }

    // Not on disk — write the embedded default
    let embedded = match name {
        "rcm.lite" => LITE_MODULE,
        "rcm.full" => DEFAULT_MODULE,
        _ => DEFAULT_MODULE,
    };

    if let Err(e) = std::fs::write(&file_path, embedded) {
        eprintln!(
            "load_menu_module: write {} failed: {e}",
            file_path.display()
        );
    } else {
        println!("load_menu_module: wrote default to {}.js", name);
    }

    embedded.to_string()
}
