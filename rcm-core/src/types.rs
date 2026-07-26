use serde::{Deserialize, Serialize};
use startmenu::Lnk;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
}

/// Snapshot of clipboard state at the time of the right-click.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClipboardInfo {
    /// Clipboard contains text (CF_UNICODETEXT / CF_TEXT).
    #[serde(default)]
    pub has_text: bool,
    /// Clipboard contains an image (CF_DIB / CF_BITMAP).
    #[serde(default)]
    pub has_image: bool,
    /// Clipboard contains file(s) (CF_HDROP).
    #[serde(default)]
    pub has_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeProps {
    pub files: Vec<FileInfo>,
    pub cwd: String,
    pub env: HashMap<String, String>,
    pub admin: bool,
    /// Current i18n language (e.g. 'en', 'zh'). Falls back to 'en' if unsupported.
    pub lang: String,
    /// Snapshot of clipboard state at the time of the right-click.
    #[serde(default)]
    pub clipboard: ClipboardInfo,
    /// Start Menu items (Lnk objects with path + args).
    #[serde(default, rename = "startmenu")]
    pub startmenu: Vec<Lnk>,
    /// Paths currently in Quick Access (checked against selected file paths).
    #[serde(default, rename = "quickAccess")]
    pub quick_access: Vec<String>,
    /// Startup / autorun entries (reuses autorun::StartupEntry).
    #[serde(default, rename = "autorun")]
    pub autorun: Vec<autorun::StartupEntry>,
}

/// Window visibility mode for spawned processes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum WindowMode {
    #[default]
    Hidden,
    Visible,
    Minimized,
    Maximized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub cmd: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub window: WindowMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub disable: bool,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub window: WindowMode,
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub command: Option<CommandPayload>,
}

impl Item {
    /// Whether this item has clickable children (submenu).
    pub fn has_children(&self) -> bool {
        !self.items.is_empty()
    }

    /// Whether this item is a leaf action (has a command to execute).
    pub fn is_action(&self) -> bool {
        self.command.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    #[serde(rename = "iconItems")]
    pub icon_items: Vec<Item>,
    pub groups: Vec<Item>,
}

/// Index path navigating the menu tree.
/// - Empty `[]` = root (iconItems + groups)
/// - `[-1, i, ...]` = icon ribbon item i, then deeper
/// - `[g, i, ...]` = groups[g].items[i], then deeper
pub type IndexPath = Vec<i32>;

/// Result of navigating to a position in the menu tree.
pub enum NavigateResult<'a> {
    /// Root level — show iconItems and groups.
    Root,
    /// Found a submenu — return its items.
    Submenu(&'a [Item]),
}

impl Menu {
    /// Navigate the menu tree following `path` and return what to display.
    pub fn navigate(&self, path: &IndexPath) -> Option<NavigateResult<'_>> {
        if path.is_empty() {
            return Some(NavigateResult::Root);
        }

        let (first, rest) = (path[0], &path[1..]);

        let item = if first == -1 {
            // icon ribbon
            let idx = rest.first().copied()? as usize;
            self.icon_items.get(idx)?
        } else {
            let group_idx = first as usize;
            let item_idx = rest.first().copied()? as usize;
            self.groups.get(group_idx)?.items.get(item_idx)?
        };

        // Walk deeper into nested items
        let mut current = item;
        for &idx in &rest[1..] {
            current = current.items.get(idx as usize)?;
        }

        if current.items.is_empty() {
            None // leaf — no submenu to show
        } else {
            Some(NavigateResult::Submenu(&current.items))
        }
    }

    /// Get a reference to the item at `path`.
    pub fn get_item(&self, path: &IndexPath) -> Option<&Item> {
        if path.is_empty() {
            return None; // root has no single item
        }

        let (first, rest) = (path[0], &path[1..]);

        let first_item = if first == -1 {
            let idx = rest.first().copied()? as usize;
            self.icon_items.get(idx)?
        } else {
            let group_idx = first as usize;
            let item_idx = rest.first().copied()? as usize;
            self.groups.get(group_idx)?.items.get(item_idx)?
        };

        let mut current = first_item;
        for &idx in &rest[1..] {
            current = current.items.get(idx as usize)?;
        }
        Some(current)
    }

    /// Compute the maximum nesting depth of the menu.
    pub fn max_depth(&self) -> usize {
        let mut max = 0usize;
        for item in &self.icon_items {
            max = max.max(Self::item_depth(item, 1));
        }
        for group in &self.groups {
            for item in &group.items {
                max = max.max(Self::item_depth(item, 1));
            }
        }
        max
    }

    fn item_depth(item: &Item, current: usize) -> usize {
        let mut max = current;
        for child in &item.items {
            max = max.max(Self::item_depth(child, current + 1));
        }
        max
    }
}
