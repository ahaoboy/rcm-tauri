const DEFAULT_MODULE: &str = include_str!("../../rcm-kit/dist/full.js");
const MENU_FILE: &str = "rcm.js";
const STYLE_FILE: &str = "style.css";
const CONFIG_FILE: &str = "rcm.config.json";

/// Write the embedded default menu JS file next to the exe.
pub fn write_menu_defaults() {
    let path = crate::exe_dir().join(MENU_FILE);
    if let Err(e) = std::fs::write(&path, DEFAULT_MODULE) {
        eprintln!("write_menu_defaults: write {} failed: {e}", path.display());
    } else {
        println!("write_menu_defaults: wrote {}", path.display());
    }
}

/// Load the menu module from disk next to the executable.
/// If it exists it is used as-is (allowing user customisation);
/// otherwise the embedded default is written to disk and returned.
pub fn load_menu_module() -> String {
    let file_path = crate::exe_dir().join(MENU_FILE);

    // Already on disk — use it
    if file_path.exists() {
        match std::fs::read_to_string(&file_path) {
            Ok(src) => {
                println!("load_menu_module: using rcm.js from disk");
                return src;
            }
            Err(e) => eprintln!("load_menu_module: read {} failed: {e}", file_path.display()),
        }
    }

    // Not on disk — write the embedded default
    if let Err(e) = std::fs::write(&file_path, DEFAULT_MODULE) {
        eprintln!(
            "load_menu_module: write {} failed: {e}",
            file_path.display()
        );
    } else {
        println!("load_menu_module: wrote default rcm.js");
    }

    DEFAULT_MODULE.to_string()
}

/// Download a JS menu file from `url` and save it next to the executable.
///
/// Returns the path that was written on success, or an error string.
pub fn download_menu(url: &str) -> Result<String, String> {
    download_file(url, MENU_FILE)
}

/// Download style.css from `url` and save it next to the executable.
pub fn download_style(url: &str) -> Result<String, String> {
    download_file(url, STYLE_FILE)
}

/// Download rcm.config.json from `url` and save it next to the executable.
pub fn download_config(url: &str) -> Result<String, String> {
    download_file(url, CONFIG_FILE)
}

/// Download a file from `url` and save it as `filename` next to the executable.
fn download_file(url: &str, filename: &str) -> Result<String, String> {
    let body = ureq::get(url)
        .call()
        .map_err(|e| format!("download {url} failed: {e}"))?
        .into_body()
        .read_to_string()
        .map_err(|e| format!("read {url} failed: {e}"))?;

    let file_path = crate::exe_dir().join(filename);

    std::fs::write(&file_path, &body)
        .map_err(|e| format!("write {} failed: {e}", file_path.display()))?;

    let disp = file_path.display().to_string();
    println!("download: saved {} bytes to {disp}", body.len());
    Ok(disp)
}
