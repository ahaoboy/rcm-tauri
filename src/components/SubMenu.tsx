import React from "react";
import type { MenuItem } from "../types/menu";
import { MenuItemRow } from "./MenuItemRow";

/* ── Props ──────────────────────────────────────────────────────────── */

interface SubMenuProps {
  items: MenuItem[];
  flip: boolean;
  onItemClick: (item: MenuItem) => void;
}

/* ═══════════════════════════════════════════════════════════════════════
   SubMenu — flyout panel shown when hovering a parent item with children
   ═══════════════════════════════════════════════════════════════════════ */

export const SubMenu: React.FC<SubMenuProps> = ({ items, flip, onItemClick }) => {
  if (!items || items.length === 0) return null;

  return (
    <div
      className={`rcm-submenu${flip ? " rcm-flip" : ""}`}
      role="menu"
    >
      {items.map((item, idx) => (
        <MenuItemRow
          key={item.key || `sub-${idx}`}
          item={item}
          onItemClick={onItemClick}
        />
      ))}
    </div>
  );
};
