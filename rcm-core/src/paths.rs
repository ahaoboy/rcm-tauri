//! Common Windows user folder locations (HOME, Desktop, Documents, …).
//!
//! Resolved via the [`dirs`] crate (`SHGetKnownFolderPath` under the hood),
//! which follows OneDrive / folder redirections instead of guessing
//! `%USERPROFILE%\…`. Used to enrich `InvokeProps.env` so menu JS can
//! reference `props.env.HOME`, etc.

use std::collections::HashMap;
use std::path::PathBuf;

/// One known user folder: env key name + a `dirs`-style resolver.
struct Folder {
    key: &'static str,
    get: fn() -> Option<PathBuf>,
}

const FOLDERS: &[Folder] = &[
    Folder { key: "HOME", get: dirs::home_dir },
    Folder { key: "DESKTOP", get: dirs::desktop_dir },
    Folder { key: "DOCUMENTS", get: dirs::document_dir },
    Folder { key: "DOWNLOADS", get: dirs::download_dir },
    Folder { key: "PICTURES", get: dirs::picture_dir },
    Folder { key: "MUSIC", get: dirs::audio_dir },
    Folder { key: "VIDEOS", get: dirs::video_dir },
];

/// Build the map of common user locations for `props.env`.
///
/// Keys are upper-case: `HOME`, `DESKTOP`, `DOCUMENTS`, `DOWNLOADS`,
/// `PICTURES`, `MUSIC`, `VIDEOS`.
pub fn common() -> HashMap<String, String> {
    let mut out = HashMap::with_capacity(FOLDERS.len());
    for folder in FOLDERS {
        if let Some(path) = (folder.get)() {
            out.insert(folder.key.to_string(), path.to_string_lossy().into_owned());
        }
    }
    out
}

