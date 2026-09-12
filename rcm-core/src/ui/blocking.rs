//! Native context-menu blocking flag, shared by every frontend.
//!
//! The real state lives inside the `rcm_com` shell extension and is queried over
//! its control pipe. Because that pipe can be momentarily unavailable (Explorer
//! restarting, DLL not yet loaded), the last successful answer is cached and
//! used as a fallback so a tray checkmark never flips spuriously.

use std::sync::atomic::{AtomicBool, Ordering};

/// Last-known blocking state.
///
/// Initialised to `true` because both frontends call `rcm_com::enable()` on
/// startup.
static BLOCKING_FALLBACK: AtomicBool = AtomicBool::new(true);

/// Read the cached blocking state without touching the pipe.
pub fn cached_blocking_enabled() -> bool {
    BLOCKING_FALLBACK.load(Ordering::Relaxed)
}

/// Override the cached state (used after a successful enable/disable).
pub fn set_cached_blocking_enabled(enabled: bool) {
    BLOCKING_FALLBACK.store(enabled, Ordering::Relaxed);
}

/// Whether native context-menu blocking is currently enabled.
///
/// Queries the DLL via the control pipe. On success the answer is cached; on
/// failure the cached value is returned.
pub fn is_blocking_enabled() -> bool {
    match rcm_com::query() {
        Ok(enabled) => {
            set_cached_blocking_enabled(enabled);
            enabled
        }
        Err(_) => cached_blocking_enabled(),
    }
}

/// Enable blocking and update the cache. Returns the DLL's error, if any.
pub fn enable_blocking() -> Result<(), String> {
    rcm_com::enable().map_err(|e| e.to_string())?;
    set_cached_blocking_enabled(true);
    Ok(())
}

/// Disable blocking and update the cache. Returns the DLL's error, if any.
pub fn disable_blocking() -> Result<(), String> {
    rcm_com::disable().map_err(|e| e.to_string())?;
    set_cached_blocking_enabled(false);
    Ok(())
}
