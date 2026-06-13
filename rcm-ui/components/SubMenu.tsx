import React from "react";
import type { MenuItem, IndexPath } from "../types/menu";
import { MenuItemRow } from "./MenuItemRow";

/* ── Props ──────────────────────────────────────────────────────────── */

interface SubMenuProps {
  items: MenuItem[];
  /** Window depth. */
  depth: number;
  /** Index path pointing to the parent of these items. */
  indexPath: IndexPath;
  showIcons?: boolean;
}

/* ═══════════════════════════════════════════════════════════════════════
   SubMenu — flyout panel shown when hovering a parent item with children
   ═══════════════════════════════════════════════════════════════════════ */

export const SubMenu: React.FC<SubMenuProps> = ({ items, depth, indexPath, showIcons }) => {
  if (!items || items.length === 0) return null;

  return (
    <div
      className="rcm-submenu"
      role="menu"
    >
      {items.map((item, idx) => (
        <MenuItemRow
          key={item.key || `sub-${idx}`}
          item={item}
          depth={depth}
          indexPath={[...indexPath, idx]}
          showIcons={showIcons}
        />
      ))}
    </div>
  );
};
