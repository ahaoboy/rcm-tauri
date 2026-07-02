import { getCurrentWindow } from "@tauri-apps/api/window"
import React, { useCallback, useRef } from "react"

import { emitMenuExecute, emitMenuHover, emitMenuHoverOut } from "../api/menuEvents"
import { feLog } from "../feLog"
import type { MenuItem, IndexPath } from "../types/menu"

interface MenuItemRowProps {
  item: MenuItem
  /** Window depth (0 = root). */
  depth: number
  /** Index path to this item in the full menu tree. */
  indexPath: IndexPath
  showIcons?: boolean
}

/**
 * MenuItemRow — single row in the menu.
 * Sends hover/click events to Rust for centralized window management.
 *
 * On hover, measures the .rcm-root and item geometry and reports to Rust.
 * Rust uses this to compute the ideal submenu position.
 */
export const MenuItemRow: React.FC<MenuItemRowProps> = ({ item, depth, indexPath, showIcons }) => {
  const rowRef = useRef<HTMLDivElement>(null)
  const hasChildren = item.items && item.items.length > 0

  const handleMouseEnter = useCallback(async () => {
    if (!rowRef.current) return

    const rowEl = rowRef.current
    const rootEl = rowEl.closest<HTMLElement>(".rcm-root")
    if (!rootEl) return

    const dpi = window.devicePixelRatio || 1

    // Parent .rcm-root absolute screen position and size (physical px)
    const win = getCurrentWindow()
    const winPos = await win.outerPosition() // physical px
    const rootRect = rootEl.getBoundingClientRect() // CSS px (positions only)
    const rowRect = rowEl.getBoundingClientRect() // CSS px (positions only)

    // .rcm-root screen position = window outer position + root's viewport offset * dpi
    const rootX = winPos.x + rootRect.left * dpi
    const rootY = winPos.y + rootRect.top * dpi

    // Use offsetWidth/offsetHeight for sizes — not affected by CSS transforms
    // (unlike getBoundingClientRect which would shrink during rcm-fade-in).
    // Hovered item Y offset from .rcm-root top (physical px)
    const itemY = (rowRect.top - rootRect.top) * dpi

    feLog.eventSend(
      "menu-hover",
      `depth=${depth} path=[${indexPath}] label='${item.label}' hasChildren=${hasChildren}`,
    )

    await emitMenuHover({
      depth,
      path: indexPath,
      rootX,
      rootY,
      rootW: rootEl.offsetWidth * dpi,
      rootH: rootEl.offsetHeight * dpi,
      itemY,
      itemH: rowEl.offsetHeight * dpi,
    })
  }, [depth, indexPath, item.label, hasChildren])

  const handleMouseLeave = useCallback(async () => {
    await emitMenuHoverOut(depth)
  }, [depth])

  const handleClick = useCallback(
    async (e: React.MouseEvent) => {
      if (item.disable) return

      if (hasChildren) {
        e.stopPropagation()
        await handleMouseEnter()
        return
      }

      if (item.command) {
        feLog.eventSend("menu-execute", `path=[${indexPath}] exe='${item.command.exe}'`)
        await emitMenuExecute(indexPath, item.command)
      } else {
        feLog.warn("MenuItemRow", `click dead item path=[${indexPath}]`)
      }
    },
    [item, hasChildren, indexPath, handleMouseEnter],
  )

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
      {showIcons !== false && item.icon && <span className="rcm-item-icon">{item.icon}</span>}
      <span className="rcm-item-label">{item.label || item.key}</span>
      {hasChildren && <span className="rcm-item-arrow">▶</span>}
    </div>
  )
}
