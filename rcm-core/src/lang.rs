/// Detect the system's default display language.
///
/// Returns a short language code:
/// - `"zh"` for Chinese (Simplified & Traditional),
/// - `"en"` for English (or anything else, which the i18n module falls back to).
///
/// ## Detection strategy (Windows)
/// Reads `HKCU\Control Panel\International\LocaleName` via the `winreg` crate.
///
/// ## Detection strategy (other platforms)
/// Reads the `LANG` environment variable.
pub fn system_lang() -> String {
    #[cfg(target_os = "windows")]
    {
        windows_lang()
    }
    #[cfg(not(target_os = "windows"))]
    {
        unix_lang()
    }
}

#[cfg(target_os = "windows")]
fn windows_lang() -> String {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let hkcu = match RegKey::predef(HKEY_CURRENT_USER).open_subkey("Control Panel\\International") {
        Ok(k) => k,
        Err(_) => return "en".to_string(),
    };

    let locale: String = match hkcu.get_value("LocaleName") {
        Ok(v) => v,
        Err(_) => return "en".to_string(),
    };

    locale_to_lang(&locale)
}

#[cfg(not(target_os = "windows"))]
fn unix_lang() -> String {
    let locale = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_else(|_| "en_US.UTF-8".to_string());

    locale_to_lang(&locale)
}

/// Map a raw locale string (e.g. "zh-CN", "en-US", "zh_TW") to a short
/// language code that the i18n module understands ("en", "zh").
fn locale_to_lang(locale: &str) -> String {
    let lower = locale.to_lowercase();

    if lower.starts_with("zh")
        || lower.starts_with("chinese")
        || lower.contains("zh-")
        || lower.contains("zh_")
    {
        return "zh".to_string();
    }

    // Everything else falls back to English
    "en".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_mapping() {
        assert_eq!(locale_to_lang("zh-CN"), "zh");
        assert_eq!(locale_to_lang("zh_TW"), "zh");
        assert_eq!(locale_to_lang("zh-Hans"), "zh");
        assert_eq!(locale_to_lang("Chinese"), "zh");
        assert_eq!(locale_to_lang("en-US"), "en");
        assert_eq!(locale_to_lang("ja-JP"), "en");
        assert_eq!(locale_to_lang("de-DE"), "en");
        assert_eq!(locale_to_lang("fr-FR"), "en");
        assert_eq!(locale_to_lang(""), "en");
    }
}
