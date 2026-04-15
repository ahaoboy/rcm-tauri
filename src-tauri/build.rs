fn main() {
    // Copy the compiled rcm_com DLL from deps to binaries/
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "release".to_string());
    let target_dir = std::path::Path::new("target").join(&profile).join("deps");
    let binaries_dir = std::path::Path::new("binaries");
    if !binaries_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&binaries_dir) {
            eprintln!("cargo:warning=Failed to create binaries directory: {e}");
            return;
        }
    }

    // Find rcm_com-*.dll (the hash suffix varies between builds)
    if let Ok(entries) = std::fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("rcm_com-")
                && name_str.ends_with(".dll")
                && !name_str.contains(".dll.")
            {
                let dest = binaries_dir.join("rcm_com.dll");
                if let Err(e) = std::fs::copy(entry.path(), &dest) {
                    eprintln!("cargo:warning=Failed to copy rcm_com DLL: {e}");
                } else {
                    println!(
                        "cargo:warning=Copied {} -> {}",
                        entry.path().display(),
                        dest.display()
                    );
                }
                break;
            }
        }
    }

    tauri_build::build();
}
