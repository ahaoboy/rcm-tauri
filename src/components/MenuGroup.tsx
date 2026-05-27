import React from "react";
import type { MenuItem, IndexPath } from "../types/menu";
import { MenuItemRow } from "./MenuItemRow";

/* ── Props ──────────────────────────────────────────────────────────── */

interface MenuGroupProps {
  /** The group wrapper — its `items` array holds the actual menu entries. */
  group: MenuItem;
  /** Window depth. */
  depth: number;
  /** Index path to the parent (root = []). */
  indexPath: IndexPath;
  /** Index of this group within groups[]. */
  groupIndex: number;
  showIcons?: boolean;
}

/* ═══════════════════════════════════════════════════════════════════════
   MenuGroup — renders a vertical list of items within a group
   ═══════════════════════════════════════════════════════════════════════ */

export const MenuGroup: React.FC<MenuGroupProps> = ({
  group,
  depth,
  indexPath,
  groupIndex,
  showIcons,
}) => {
  const entries = group.items;
  if (!entries || entries.length === 0) return null;

  return (
    <div className="rcm-group" role="group">
      {entries.map((item, idx) => (
        <MenuItemRow
          key={item.key || `item-${idx}`}
          item={item}
          depth={depth}
          indexPath={[...indexPath, groupIndex, idx]}
          showIcons={showIcons}
        />
      ))}
    </div>
  );
};
