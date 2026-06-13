use autorun::StartupScope;

const REG_VAL_RCM: &str = "rcm-tauri";

pub fn is_autostart_enabled() -> bool {
    autorun::exists(REG_VAL_RCM, StartupScope::CurrentUser).unwrap_or(false)
}

/// Add the current executable to the user's startup registry key.
pub fn enable_autostart() -> Result<(), std::io::Error> {
    let exe = std::env::current_exe()?;
    autorun::add(
        REG_VAL_RCM,
        &exe.to_string_lossy(),
        StartupScope::CurrentUser,
    )
    .map_err(std::io::Error::other)
}

/// Remove the app from the user's startup registry key.
pub fn disable_autostart() -> Result<(), std::io::Error> {
    autorun::remove(REG_VAL_RCM, StartupScope::CurrentUser)
        .map_err(std::io::Error::other)
}
