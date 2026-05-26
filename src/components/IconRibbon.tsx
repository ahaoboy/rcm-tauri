import React from "react";
import type { MenuItem } from "../types/menu";

/* ── Props ──────────────────────────────────────────────────────────── */

interface IconRibbonProps {
  items: MenuItem[];
  onItemClick: (item: MenuItem) => void;
}

/* ═══════════════════════════════════════════════════════════════════════
   IconRibbon — top icon bar in Win11-style context menus
   ═══════════════════════════════════════════════════════════════════════ */

export const IconRibbon: React.FC<IconRibbonProps> = ({ items, onItemClick }) => {
  if (!items || items.length === 0) return null;

  return (
    <div className="rcm-ribbon" role="toolbar" aria-label="Quick actions">
      {items.map((item, idx) => (
        <button
          key={item.key || `ribbon-${idx}`}
          className="rcm-ribbon-btn"
          aria-disabled={item.disable}
          title={item.label || item.key}
          tabIndex={item.disable ? -1 : 0}
          onClick={(e) => {
            e.stopPropagation();
            if (!item.disable) onItemClick(item);
          }}
        >
          {item.icon || item.key}
        </button>
      ))}
    </div>
  );
};
