import React, { useCallback, useRef, useState } from "react";
import type { MenuItem } from "../types/menu";
import { SubMenu } from "./SubMenu";

/* ── Props ──────────────────────────────────────────────────────────── */

interface MenuItemRowProps {
  item: MenuItem;
  onItemClick: (item: MenuItem) => void;
  showIcons?: boolean;
}

/* ═══════════════════════════════════════════════════════════════════════
   MenuItemRow — single row in the menu
   Renders icon, label, optional shortcut, submenu arrow.
   ═══════════════════════════════════════════════════════════════════════ */

export const MenuItemRow: React.FC<MenuItemRowProps> = ({ item, onItemClick, showIcons }) => {
  const rowRef = useRef<HTMLDivElement>(null);
  const [submenuOpen, setSubmenuOpen] = useState(false);
  const [submenuFlip, setSubmenuFlip] = useState(false);

  const hasChildren = item.items && item.items.length > 0;

  // Check if submenu would overflow viewport → flip to left side
  const handleMouseEnter = useCallback(() => {
    if (!hasChildren || !rowRef.current) return;

    setSubmenuOpen(true);

    // Estimate submenu position
    const rowRect = rowRef.current.getBoundingClientRect();
    const estimatedWidth = 220;
    const spaceOnRight = window.innerWidth - rowRect.right - 8;

    if (spaceOnRight < estimatedWidth) {
      setSubmenuFlip(true);
    } else {
      setSubmenuFlip(false);
    }
  }, [hasChildren]);

  const handleMouseLeave = useCallback(() => {
    setSubmenuOpen(false);
  }, []);

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      // If this item has children, do not fire the action — just toggle submenu
      if (hasChildren) {
        e.stopPropagation();
        return;
      }
      onItemClick(item);
    },
    [item, hasChildren, onItemClick],
  );

  return (
    <div
      ref={rowRef}
      className="rcm-item"
      role="menuitem"
      aria-disabled={item.disable}
      tabIndex={item.disable ? -1 : 0}
      onClick={handleClick}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* Icon */}
      {showIcons !== false && item.icon && <span className="rcm-item-icon">{item.icon}</span>}

      {/* Label */}
      <span className="rcm-item-label">{item.label || item.key}</span>

      {/* Shortcut hint — reserved for future use */}
      {/* <span className="rcm-item-shortcut">{item.shortcut}</span> */}

      {/* Submenu arrow */}
      {hasChildren && <span className="rcm-item-arrow">▶</span>}

      {/* Nested submenu */}
      {hasChildren && submenuOpen && (
        <SubMenu items={item.items!} flip={submenuFlip} onItemClick={onItemClick} showIcons={showIcons} />
      )}
    </div>
  );
};
