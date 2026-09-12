//! Flattening a menu subtree into renderable rows.
//!
//! The renderer-agnostic half of "typesetting": given the full [`Menu`] tree,
//! the index path of the level to display, and whether icons are enabled,
//! produce the ordered list of rows a frontend can draw.
//!
//! This reproduces the DOM structure the Tauri frontend renders in
//! `rcm-ui/components/ContextMenu.tsx`:
//!
//! - the **root** level shows the icon ribbon followed by every group, with a
//!   separator *between* groups (never before the first or after the last);
//! - a **submenu** level shows the children of the item at `path`, with no
//!   separators.
//!
//! Item paths are emitted exactly as the Tauri frontend computes them, so both
//! frontends agree on what `[]`, `[group, item, ...]` and `[-1, ribbon, ...]`
//! mean — see [`IndexPath`].

use crate::types::{IndexPath, Item, Menu, NavigateResult};

/// Structural equality for two menu items.
///
/// `Item` does not implement `PartialEq` (its `command` payload is not
/// comparable), so rows are compared on the fields the layout actually depends
/// on: identity, presentation, enablement and nesting.
fn item_eq(a: &Item, b: &Item) -> bool {
    a.key == b.key
        && a.icon == b.icon
        && a.label == b.label
        && a.disable == b.disable
        && a.items.len() == b.items.len()
        && a.items.iter().zip(&b.items).all(|(x, y)| item_eq(x, y))
}

/// Whether a level renders an icon gutter and/or the ribbon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlattenOptions {
    /// The user's `icons` preference from `rcm.config.json`.
    pub icons_enabled: bool,
}

impl FlattenOptions {
    pub const fn new(icons_enabled: bool) -> Self {
        Self { icons_enabled }
    }
}

/// One rendered row of a menu level.
#[derive(Debug, Clone)]
pub enum MenuRow {
    /// A clickable entry. `path` is the full index path to it in the tree.
    Item { item: Box<Item>, path: IndexPath },
    /// A horizontal separator between groups.
    Separator,
}

impl MenuRow {
    /// The item for this row, if it is not a separator.
    pub fn item(&self) -> Option<&Item> {
        match self {
            Self::Item { item, .. } => Some(item),
            Self::Separator => None,
        }
    }
    /// The index path for this row, if it is not a separator.
    pub fn path(&self) -> Option<&IndexPath> {
        match self {
            Self::Item { path, .. } => Some(path),
            Self::Separator => None,
        }
    }

    /// Whether this row is a separator.
    pub const fn is_separator(&self) -> bool {
        matches!(self, Self::Separator)
    }
}

impl PartialEq for MenuRow {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Separator, Self::Separator) => true,
            (Self::Item { item: a, path: pa }, Self::Item { item: b, path: pb }) => {
                pa == pb && item_eq(a, b)
            }
            _ => false,
        }
    }
}

/// A single displayed menu level: the ribbon (root only) plus its rows.
#[derive(Debug, Clone)]
pub struct MenuLevel {
    /// Nesting depth: 0 = root, 1.. = submenus.
    pub depth: usize,
    /// Index path that produced this level (empty for the root).
    pub path: IndexPath,
    /// Icon-ribbon entries (root only, empty elsewhere).
    pub ribbon: Vec<Item>,
    /// Ordered rows to render.
    pub rows: Vec<MenuRow>,
    /// Whether the icon ribbon should be drawn.
    pub ribbon_visible: bool,
    /// Whether rows should reserve an icon gutter.
    ///
    /// Decided per level rather than per row so labels stay aligned within a
    /// level, and only when icons are enabled *and* at least one row has an
    /// icon — otherwise the gutter would be dead space.
    pub icons: bool,
}

impl PartialEq for MenuLevel {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth
            && self.path == other.path
            && self.rows == other.rows
            && self.ribbon_visible == other.ribbon_visible
            && self.icons == other.icons
            && self.ribbon.len() == other.ribbon.len()
            && self
                .ribbon
                .iter()
                .zip(&other.ribbon)
                .all(|(a, b)| item_eq(a, b))
    }
}

impl MenuLevel {
    /// Flatten the level at `path` into renderable rows.
    pub fn flatten(menu: &Menu, path: &[i32], options: FlattenOptions) -> Self {
        let (ribbon, rows) = if path.is_empty() {
            Self::flatten_root(menu)
        } else {
            Self::flatten_submenu(menu, path)
        };

        let icons = options.icons_enabled
            && rows
                .iter()
                .any(|row| row.item().is_some_and(|item| !item.icon.is_empty()));
        let ribbon_visible = options.icons_enabled && !ribbon.is_empty();

        Self {
            depth: Self::depth_of(path),
            path: path.to_vec(),
            ribbon,
            rows,
            ribbon_visible,
            icons,
        }
    }

    /// Root level: every group's items, separated by separators.
    fn flatten_root(menu: &Menu) -> (Vec<Item>, Vec<MenuRow>) {
        let mut rows = Vec::new();
        let mut is_first_group = true;

        for (group_index, group) in menu.groups.iter().enumerate() {
            if group.items.is_empty() {
                continue;
            }
            // A separator between groups, but never leading or trailing.
            if !is_first_group {
                rows.push(MenuRow::Separator);
            }
            is_first_group = false;

            for (item_index, item) in group.items.iter().enumerate() {
                rows.push(MenuRow::Item {
                    item: Box::new(item.clone()),
                    path: vec![group_index as i32, item_index as i32],
                });
            }
        }

        (menu.icon_items.clone(), rows)
    }

    /// Submenu level: the children of the item at `path`.
    fn flatten_submenu(menu: &Menu, path: &[i32]) -> (Vec<Item>, Vec<MenuRow>) {
        let owned: IndexPath = path.to_vec();
        let items: &[Item] = match menu.navigate(&owned) {
            Some(NavigateResult::Submenu(items)) => items,
            _ => &[],
        };

        let rows = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let mut item_path = path.to_vec();
                item_path.push(index as i32);
                MenuRow::Item {
                    item: Box::new(item.clone()),
                    path: item_path,
                }
            })
            .collect();

        (Vec::new(), rows)
    }

    /// Display depth for an index path.
    ///
    /// Matches the Tauri frontend: the path is `[selector, first_index, ...]`,
    /// so depth is `path.len().saturating_sub(1)`.
    pub fn depth_of(path: &[i32]) -> usize {
        path.len().saturating_sub(1)
    }

    /// The row at `index`, if any.
    pub fn row(&self, index: usize) -> Option<&MenuRow> {
        self.rows.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Menu;

    fn item(key: &str) -> Item {
        Item {
            key: key.to_string(),
            icon: String::new(),
            label: key.to_string(),
            disable: false,
            admin: false,
            window: Default::default(),
            items: Vec::new(),
            command: None,
        }
    }

    fn menu_with_groups(groups: Vec<Vec<Item>>) -> Menu {
        Menu {
            icon_items: Vec::new(),
            groups: groups
                .into_iter()
                .map(|items| Item {
                    items,
                    ..item("group")
                })
                .collect(),
        }
    }

    #[test]
    fn root_separates_groups_but_not_edges() {
        let menu = menu_with_groups(vec![vec![item("a")], vec![item("b")], vec![item("c")]]);
        let level = MenuLevel::flatten(&menu, &[], FlattenOptions::new(false));

        // Rows are: [a, SEP, b, SEP, c] — separators between groups only.
        assert_eq!(level.rows.len(), 5);
        assert!(!level.rows[0].is_separator());
        assert!(level.rows[1].is_separator());
        assert!(!level.rows[2].is_separator());
        assert!(level.rows[3].is_separator());
        assert!(!level.rows[4].is_separator(), "no trailing separator");
    }

    #[test]
    fn root_skips_empty_groups() {
        let menu = menu_with_groups(vec![vec![item("a")], Vec::new(), vec![item("b")]]);
        let level = MenuLevel::flatten(&menu, &[], FlattenOptions::new(false));

        // Empty group contributes no items and no extra separator.
        assert_eq!(level.rows.len(), 3);
        assert!(level.rows[1].is_separator());
    }

    #[test]
    fn root_paths_are_group_then_item() {
        let menu = menu_with_groups(vec![vec![item("a"), item("b")]]);
        let level = MenuLevel::flatten(&menu, &[], FlattenOptions::new(false));
        assert_eq!(level.rows[0].path().unwrap(), &vec![0, 0]);
        assert_eq!(level.rows[1].path().unwrap(), &vec![0, 1]);
    }

    #[test]
    fn submenu_paths_extend_parent() {
        let mut parent = item("open");
        parent.items = vec![item("with"), item("in")];
        let menu = menu_with_groups(vec![vec![parent]]);

        let level = MenuLevel::flatten(&menu, &[0, 0], FlattenOptions::new(false));
        assert_eq!(level.depth, 1);
        assert_eq!(level.rows[0].path().unwrap(), &vec![0, 0, 0]);
        assert_eq!(level.rows[1].path().unwrap(), &vec![0, 0, 1]);
        // Submenus never show the ribbon.
        assert!(level.ribbon.is_empty());
        assert!(!level.ribbon_visible);
    }

    #[test]
    fn icon_gutter_requires_icons_enabled_and_a_row_icon() {
        let mut with_icon = item("a");
        with_icon.icon = "\u{1F4C1}".into();
        let menu = menu_with_groups(vec![vec![with_icon]]);

        assert!(
            MenuLevel::flatten(&menu, &[], FlattenOptions::new(true)).icons,
            "icons on + a row with an icon => gutter"
        );
        assert!(
            !MenuLevel::flatten(&menu, &[], FlattenOptions::new(false)).icons,
            "icons off => no gutter"
        );

        let plain = menu_with_groups(vec![vec![item("a")]]);
        assert!(
            !MenuLevel::flatten(&plain, &[], FlattenOptions::new(true)).icons,
            "icons on but no row has an icon => no dead gutter"
        );
    }

    #[test]
    fn depth_of_matches_index_path_encoding() {
        assert_eq!(MenuLevel::depth_of(&[]), 0);
        assert_eq!(MenuLevel::depth_of(&[0, 1]), 1);
        assert_eq!(MenuLevel::depth_of(&[-1, 0]), 1);
        assert_eq!(MenuLevel::depth_of(&[0, 1, 2]), 2);
    }
}
