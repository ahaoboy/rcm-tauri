//! Centralised logging with optional file output.
//!
//! Logs always print to stdout. When `FILE_LOGGING` is toggled on
//! (via the tray menu), logs are also appended to `<exe_name>.log`
//! next to the executable.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// When true, all log calls also write to the .log file on disk.
pub static FILE_LOGGING: AtomicBool = AtomicBool::new(false);

/// Guards concurrent writes to the log file.
static FILE_MUTEX: Mutex<()> = Mutex::new(());

fn log_path() -> PathBuf {
    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_os_string()))
        .unwrap_or_else(|| "rcm-tauri".into());
    crate::exe_dir().join(exe_name).with_extension("log")
}

/// `HH:MM:SS.mmm` derived from the Unix epoch (UTC), not from process start.
fn timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let ms = now.subsec_millis();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}

/// Append one already-formatted line to the log file, when file logging is on.
///
/// A poisoned lock is treated as "skip this line" rather than panicking: a
/// logging module must never be the reason the app crashes.
fn write_line(line: &str) {
    if !FILE_LOGGING.load(Ordering::Relaxed) {
        return;
    }
    let Ok(_guard) = FILE_MUTEX.lock() else {
        return;
    };
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let _ = writeln!(f, "{line}");
    }
}

/// Log an info-level message. Always prints to stdout.
pub fn info(tag: &str, msg: &str) {
    let line = format!("{} [{}] {}", timestamp(), tag, msg);
    println!("{line}");
    write_line(&line);
}

/// Log a warning-level message.
pub fn warn(tag: &str, msg: &str) {
    let line = format!("{} [{}] WARN {}", timestamp(), tag, msg);
    eprintln!("{line}");
    write_line(&line);
}

/// Log an error-level message.
pub fn error(tag: &str, msg: &str) {
    let line = format!("{} [{}] ERROR {}", timestamp(), tag, msg);
    eprintln!("{line}");
    write_line(&line);
}

/// Log a message coming from the frontend (JS/TS side).
pub fn frontend(tag: &str, msg: &str) {
    info(&format!("FE:{}", tag), msg);
}

/// Log an event being sent or received.
pub fn event(direction: &str, event_name: &str, detail: &str) {
    info(
        &format!("EVENT:{}", direction),
        &format!("{} | {}", event_name, detail),
    );
}
