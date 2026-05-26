import React from "react";
import type { MenuItem } from "../types/menu";
import { MenuItemRow } from "./MenuItemRow";

/* ── Props ──────────────────────────────────────────────────────────── */

interface MenuGroupProps {
  /** The group wrapper — its `items` array holds the actual menu entries. */
  group: MenuItem;
  onItemClick: (item: MenuItem) => void;
}

/* ═══════════════════════════════════════════════════════════════════════
   MenuGroup — renders a vertical list of items within a group
   ═══════════════════════════════════════════════════════════════════════ */

export const MenuGroup: React.FC<MenuGroupProps> = ({ group, onItemClick }) => {
  const entries = group.items;
  if (!entries || entries.length === 0) return null;

  return (
    <div className="rcm-group" role="group">
      {entries.map((item, idx) => (
        <MenuItemRow key={item.key || `item-${idx}`} item={item} onItemClick={onItemClick} />
      ))}
    </div>
  );
};
