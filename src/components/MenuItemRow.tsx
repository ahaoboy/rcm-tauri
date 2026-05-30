import React, { useCallback, useRef } from "react";
import type { MenuItem, IndexPath } from "../types/menu";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { feLog } from "../feLog";

/* ── Props ──────────────────────────────────────────────────────────── */

interface MenuItemRowProps {
  item: MenuItem;
  /** Window depth (0 = root). */
  depth: number;
  /** Index path to this item in the full menu tree. */
  indexPath: IndexPath;
  showIcons?: boolean;
}

/* ═══════════════════════════════════════════════════════════════════════
   MenuItemRow — single row in the menu
   Sends hover/click events to Rust for centralized window management.
   ═══════════════════════════════════════════════════════════════════════ */

export const MenuItemRow: React.FC<MenuItemRowProps> = ({
  item,
  depth,
  indexPath,
  showIcons,
}) => {
  const rowRef = useRef<HTMLDivElement>(null);
  const hasChildren = item.items && item.items.length > 0;

  // ── Hover: tell Rust about this item ────────────────────────────
  const handleMouseEnter = useCallback(async () => {
    if (!rowRef.current) return;

    const rect = rowRef.current.getBoundingClientRect();
    const win = getCurrentWindow();
    const pos = await win.outerPosition();   // physical pixels
    const size = await win.outerSize();       // physical pixels
    const innerSize = await win.innerSize();  // physical pixels

    // DPI scale factor: getBoundingClientRect returns CSS pixels,
    // but outerPosition/outerSize are in physical pixels.
    const dpi = window.devicePixelRatio || 1;

    // Absolute screen X of the viewport's right edge (content boundary, no shadow)
    const contentRight = pos.x + innerSize.width;

    // Parent menu content dimensions (.rcm-root element's rendered size)
    const parentRoot = rowRef.current.closest('.rcm-root');
    const parentContentHeight = (parentRoot
      ? parentRoot.getBoundingClientRect().height
      : innerSize.height) * dpi;
    const parentContentWidth = (parentRoot
      ? parentRoot.getBoundingClientRect().width
      : innerSize.width) * dpi;

    feLog.eventSend("menu-hover", `depth=${depth} path=[${indexPath}] label='${item.label}' hasChildren=${hasChildren}`);

    await emit("menu-hover", {
      depth,
      path: indexPath,
      parentX: pos.x,
      parentY: pos.y,
      parentW: size.width,
      parentH: size.height,
      parentContentHeight,
      parentContentWidth,
      itemX: rect.left * dpi,
      itemY: rect.top * dpi,
      itemW: rect.width * dpi,
      itemH: rect.height * dpi,
      contentRight,
    });
  }, [depth, indexPath, item.label, hasChildren]);

  // ── Hover out: tell Rust mouse left this item ───────────────────
  const handleMouseLeave = useCallback(async () => {
    await emit("menu-hover-out", { depth });
  }, [depth]);

  // ── Click: select (if has children) or execute ─────────────────
  const handleClick = useCallback(
    async (e: React.MouseEvent) => {
      if (item.disable) return;

      if (hasChildren) {
        // Has children → "select": same as hover (show submenu)
        e.stopPropagation();
        await handleMouseEnter();
        return;
      }

      // No children → "execute"
      if (item.command) {
        feLog.eventSend("menu-execute", `path=[${indexPath}] exe='${item.command.exe}'`);
        await emit("menu-execute", {
          path: indexPath,
          command: item.command,
        });
      } else {
        feLog.warn("MenuItemRow", `click dead item path=[${indexPath}]`);
      }
    },
    [item, hasChildren, indexPath, handleMouseEnter],
  );

  return (
    <div
      ref={rowRef}
      className="rcm-item"
      role="menuitem"
      aria-disabled={item.disable}
      aria-haspopup={hasChildren}
      tabIndex={item.disable ? -1 : 0}
      onClick={handleClick}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* Icon */}
      {showIcons !== false && item.icon && (
        <span className="rcm-item-icon">{item.icon}</span>
      )}

      {/* Label */}
      <span className="rcm-item-label">{item.label || item.key}</span>

      {/* Submenu arrow */}
      {hasChildren && <span className="rcm-item-arrow">▶</span>}
    </div>
  );
};
