/// Check if another `rcm-tauri.exe` or `rcm.exe` process is already running.
pub fn is_rcm_process_running() -> bool {
    let our_pid = std::process::id();
    let our_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "rcm-tauri.exe".into());

    for name in &[our_name.as_str(), "rcm.exe"] {
        let filter = format!("IMAGENAME eq {name}");
        if let Ok(output) = rcm_core::sys_cmd("tasklist")
            .args(["/fo", "csv", "/nh", "/fi", &filter])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let pid_str = our_pid.to_string();
            for line in stdout.lines() {
                if line.contains(name) && !line.contains(&pid_str) {
                    return true;
                }
            }
        }
    }
    false
}
