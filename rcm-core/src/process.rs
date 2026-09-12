//! Single-instance detection.
//!
//! Both frontends refuse to start when another RCM process is already running,
//! because two instances would fight over the `rcm_com` pipe and the tray icon.

/// Executable names that count as "another RCM instance".
const SIBLING_NAMES: &[&str] = &["rcm.exe"];

/// Whether another RCM process (this executable or a sibling build) is running.
///
/// Compares against the current PID so the caller's own process is not counted.
pub fn is_rcm_process_running() -> bool {
    let our_pid = std::process::id();
    let our_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "rcm.exe".into());

    let mut names: Vec<&str> = vec![our_name.as_str()];
    names.extend_from_slice(SIBLING_NAMES);

    for name in names {
        let filter = format!("IMAGENAME eq {name}");
        let Ok(output) = crate::sys_cmd("tasklist")
            .args(["/fo", "csv", "/nh", "/fi", &filter])
            .output()
        else {
            continue;
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pid = our_pid.to_string();
        if stdout
            .lines()
            .any(|line| line.contains(name) && !line.contains(&pid))
        {
            return true;
        }
    }
    false
}
