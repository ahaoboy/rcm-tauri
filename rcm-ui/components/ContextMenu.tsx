import React, { useCallback, useEffect, useRef, useState } from "react";
import type { MenuData, MenuItem, IndexPath } from "../types/menu";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { WINDOW_PADDING } from "../constants/layout";

/* ── Sub-components ─────────────────────────────────────────────────── */

import { IconRibbon } from "./IconRibbon";
import { MenuGroup } from "./MenuGroup";
import { MenuSeparator } from "./MenuSeparator";

/* ── Props ──────────────────────────────────────────────────────────── */

interface ContextMenuProps {
  /** Window depth: 0 = root. */
  depth: number;
  /** Index path to the submenu this window renders. Empty = root. */
  indexPath: IndexPath;
  /** Full menu data — every window has the complete tree. */
  menu: MenuData;
  showIcons?: boolean;
  /** Called after the window has been resized to fit content.
   *  The parent can then clamp position and show the window. */
  onReady?: () => void;
}

/* ── Navigation helpers ─────────────────────────────────────────────── */

/**
 * Navigate the menu tree following `path` and return the items to display.
 * - Empty path → root (iconItems + groups)
 * - Non-empty → the `.items` of the MenuItem at that path
 */
function navigateMenu(menu: MenuData, path: IndexPath): {
  type: "root";
  iconItems: MenuItem[];
  groups: MenuItem[];
} | {
  type: "submenu";
  items: MenuItem[];
} | null {
  if (path.length === 0) {
    return { type: "root", iconItems: menu.iconItems, groups: menu.groups };
  }

  // Walk the path to find the target item
  const [first, ...rest] = path;
  let item: MenuItem | undefined;

  if (first === -1) {
    // Icon ribbon path: [-1, iconIdx, ...deeper]
    const idx = rest[0];
    if (idx === undefined) return null;
    item = menu.iconItems[idx];
    if (!item) return null;
    // Walk deeper
    for (let i = 1; i < rest.length; i++) {
      item = item.items[rest[i]];
      if (!item) return null;
    }
  } else {
    // Group path: [groupIdx, itemIdx, ...deeper]
    const groupIdx = first;
    const itemIdx = rest[0];
    if (itemIdx === undefined) return null;
    item = menu.groups[groupIdx]?.items[itemIdx];
    if (!item) return null;
    // Walk deeper
    for (let i = 1; i < rest.length; i++) {
      item = item.items[rest[i]];
      if (!item) return null;
    }
  }

  // Return the item's children as a flat submenu
  return { type: "submenu", items: item.items || [] };
}

/* ═══════════════════════════════════════════════════════════════════════
   ContextMenu — renders root or submenu based on indexPath
   ═══════════════════════════════════════════════════════════════════════ */

export const ContextMenu: React.FC<ContextMenuProps> = ({
  depth,
  indexPath,
  menu,
  showIcons = false,
  onReady,
}) => {
  const rootRef = useRef<HTMLDivElement>(null);
  const [menuSize, setMenuSize] = useState({ width: 280, height: 400 });
  const readyCalled = useRef(false);

  // Resolve what to render
  const resolved = navigateMenu(menu, indexPath);

  // Resize window to fit content, then signal parent
  const resizeWindow = useCallback(async () => {
    if (!rootRef.current) return;
    const rect = rootRef.current.getBoundingClientRect();
    const w = Math.ceil(rect.width) + WINDOW_PADDING;
    const h = Math.ceil(rect.height) + WINDOW_PADDING;
    if (w === menuSize.width && h === menuSize.height) {
      // Already correct size — signal ready
      if (!readyCalled.current) {
        readyCalled.current = true;
        onReady?.();
      }
      return;
    }
    setMenuSize({ width: w, height: h });
    try {
      await getCurrentWindow().setSize(new LogicalSize(w, h));
      // Signal parent that the window is now correctly sized
      if (!readyCalled.current) {
        readyCalled.current = true;
        onReady?.();
      }
    } catch {
      // Window may not be available yet
    }
  }, [menuSize, onReady]);

  // Reset ready flag when menu data changes
  useEffect(() => {
    readyCalled.current = false;
  }, [menu, indexPath]);

  useEffect(() => {
    const raf = requestAnimationFrame(() => {
      resizeWindow();
    });
    return () => cancelAnimationFrame(raf);
  }, [resolved, resizeWindow]);

  if (!resolved) {
    return <div className="rcm-root" />;
  }

  // ── Root rendering ──────────────────────────────────────────────
  if (resolved.type === "root") {
    const { iconItems, groups } = resolved;
    const hasIconItems = iconItems && iconItems.length > 0;
    const visibleGroups = groups.filter((g) => g.items && g.items.length > 0);

    return (
      <div className="rcm-root" ref={rootRef} role="menu">
        {showIcons && hasIconItems && (
          <IconRibbon
            items={iconItems}
            iconBasePath={-1}
          />
        )}

        {visibleGroups.map((group, gi) => (
          <React.Fragment key={gi}>
            {gi > 0 && <MenuSeparator />}
            <MenuGroup
              group={group}
              depth={depth}
              indexPath={indexPath}
              groupIndex={gi}
              showIcons={showIcons}
            />
          </React.Fragment>
        ))}
      </div>
    );
  }

  // ── Submenu rendering ───────────────────────────────────────────
  return (
    <div className="rcm-root" ref={rootRef} role="menu">
      {resolved.items.map((item, idx) => (
        <MenuItemRowWrapper
          key={item.key || `sub-${idx}`}
          item={item}
          depth={depth}
          indexPath={[...indexPath, idx]}
          showIcons={false}
        />
      ))}
    </div>
  );
};

/* ── Small wrapper to avoid circular imports ────────────────────────── */

import { MenuItemRow } from "./MenuItemRow";

const MenuItemRowWrapper: React.FC<{
  item: MenuItem;
  depth: number;
  indexPath: IndexPath;
  showIcons?: boolean;
}> = ({ item, depth, indexPath, showIcons }) => {
  return (
    <MenuItemRow
      item={item}
      depth={depth}
      indexPath={indexPath}
      showIcons={showIcons}
    />
  );
};
