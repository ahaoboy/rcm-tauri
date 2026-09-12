//! Config-file access shared by the config editor and its Tauri commands.
//!
//! Both frontends expose the same three editable files and the same four
//! operations (list, read, save, open in the OS default program), so the
//! validation and error wording live here once.

use std::path::PathBuf;

/// Files the editor is allowed to touch.
///
/// Anything else is rejected, so a frontend cannot be tricked into reading or
/// writing an arbitrary path next to the executable.
pub const VALID_FILES: &[&str] = &["rcm.js", "style.css", "rcm.config.json"];

/// Default file the editor opens.
pub const DEFAULT_FILE: &str = "rcm.config.json";

/// Whether `name` is one of [`VALID_FILES`].
pub fn is_config_file(name: &str) -> bool {
    VALID_FILES.contains(&name)
}

/// Absolute path of `name`, rejecting anything not in [`VALID_FILES`].
pub fn config_file_path(name: &str) -> Result<PathBuf, String> {
    if !is_config_file(name) {
        return Err(format!("Invalid file: {name}"));
    }
    Ok(crate::exe_dir().join(name))
}

/// Read a config file from the exe directory.
pub fn read_config_file(name: &str) -> Result<String, String> {
    let path = config_file_path(name)?;
    std::fs::read_to_string(&path).map_err(|e| format!("Read failed: {e}"))
}

/// Write `content` to a config file in the exe directory.
pub fn save_config_file(name: &str, content: &str) -> Result<(), String> {
    let path = config_file_path(name)?;
    std::fs::write(&path, content).map_err(|e| format!("Save failed: {e}"))
}

/// Open a config file with the operating system's default program.
pub fn open_config_file(name: &str) -> Result<(), String> {
    let path = config_file_path(name)?;
    crate::sys_cmd("cmd")
        .args(["/c", "start", "", &path.to_string_lossy()])
        .spawn()
        .map_err(|e| format!("Open failed: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_editable_files() {
        for name in VALID_FILES {
            assert!(is_config_file(name), "{name} should be editable");
        }
    }

    #[test]
    fn rejects_paths_outside_the_allow_list() {
        for name in [
            "rcm.exe",
            "../rm",
            "rcm.config.json.bak",
            "",
            "C:\\evil.txt",
        ] {
            assert!(!is_config_file(name), "{name} must be rejected");
            assert!(config_file_path(name).is_err());
            assert!(read_config_file(name).is_err());
            assert!(save_config_file(name, "x").is_err());
            assert!(open_config_file(name).is_err());
        }
    }

    #[test]
    fn rejects_are_reported_as_invalid_file() {
        let err = config_file_path("nope").unwrap_err();
        assert_eq!(err, "Invalid file: nope");
    }
}
