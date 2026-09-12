import React, { useCallback, useRef } from "react"

import { emitMenuExecute, emitMenuHover } from "../api/menuEvents"
import { feLog } from "../feLog"
import type { MenuItem, IndexPath } from "../types/menu"
import { offsetWithinRoot } from "../utils/measure"

interface MenuItemRowProps {
  item: MenuItem
  /** Window depth (0 = root). */
  depth: number
  /** Index path to this item in the full menu tree. */
  indexPath: IndexPath
  showIcons?: boolean
}

/**
 * MenuItemRow — a single row in the menu.
 *
 * Hover and click are forwarded to Rust, which decides what to show and where.
 * The only geometry reported is this row's offset from the top of `.rcm-root`,
 * which Rust uses to align a submenu with the hovered row.
 */
export const MenuItemRow: React.FC<MenuItemRowProps> = ({
  item,
  depth,
  indexPath,
  showIcons,
}) => {
  const rowRef = useRef<HTMLDivElement>(null)
  const hasChildren = item.items && item.items.length > 0

  /**
   * Row index within its level.
   *
   * `indexPath` is `[selector, firstIndex, ...deeper]`, so the row index is the
   * last element for a level rendered directly from its parent.
   */
  const rowIndex = indexPath[indexPath.length - 1] ?? 0

  const handleMouseEnter = useCallback(async () => {
    const rowEl = rowRef.current
    if (!rowEl) return

    const rootEl = rowEl.closest<HTMLElement>(".rcm-root")
    if (!rootEl) return

    feLog.eventSend(
      "menu-hover",
      `depth=${depth} path=[${indexPath}] label='${item.label}' hasChildren=${hasChildren}`,
    )

    await emitMenuHover({
      depth,
      path: indexPath,
      index: rowIndex,
      itemY: offsetWithinRoot(rowEl, rootEl),
    })
  }, [depth, indexPath, rowIndex, item.label, hasChildren])

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
    >
      {showIcons !== false && item.icon && <span className="rcm-item-icon">{item.icon}</span>}
      <span className="rcm-item-label">{item.label || item.key}</span>
      {hasChildren && <span className="rcm-item-arrow">▶</span>}
    </div>
  )
}
