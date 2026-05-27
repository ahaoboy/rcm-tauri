use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeProps {
    pub files: Vec<FileInfo>,
    pub cwd: String,
    pub env: HashMap<String, String>,
    pub admin: bool,
    #[serde(rename = "type")]
    pub type_name: String,
    /// Current i18n language (e.g. 'en', 'zh'). Falls back to 'en' if unsupported.
    pub lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub exe: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub window: String,
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
    pub window: String,
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

pub fn invoke(props: InvokeProps) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    crate::vm::invoke(&props)
}

pub fn rcm() -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());
    let props = InvokeProps {
        files: vec![],
        cwd: "C:\\".to_string(),
        env,
        admin: false,
        type_name: "Desktop".to_string(),
        lang: crate::lang::system_lang(),
    };

    invoke(props)
}

/// Build a menu from real right-click context data received via the pipe.
pub fn rcm_from_info(info: &rcm_com::ContextMenuInfo) -> std::result::Result<Menu, Box<dyn std::error::Error>> {
    let mut env = HashMap::new();
    env.insert("OS".to_string(), "Windows".to_string());

    let files: Vec<FileInfo> = info.files.iter().map(|path| {
        let p = std::path::Path::new(path);
        FileInfo {
            name: p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            path: path.clone(),
            is_dir: p.is_dir(),
        }
    }).collect();

    let props = InvokeProps {
        files,
        cwd: info.dir.clone(),
        env,
        admin: false,
        type_name: if info.bg { "Background".to_string() } else { "File".to_string() },
        lang: crate::lang::system_lang(),
    };

    invoke(props)
}
