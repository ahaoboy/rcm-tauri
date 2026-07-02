import {
  availableMonitors,
  getCurrentWindow,
  LogicalSize,
  PhysicalPosition,
} from "@tauri-apps/api/window"
import React, { useCallback, useEffect, useRef } from "react"

import { feLog } from "../feLog"
import type { MenuData, MenuItem, IndexPath } from "../types/menu"
import { computeWindowPosition } from "../utils/layout"
import type { PendingPos } from "../utils/layout"
import { IconRibbon } from "./IconRibbon"
import { MenuGroup } from "./MenuGroup"
import { MenuSeparator } from "./MenuSeparator"

interface ContextMenuProps {
  depth: number
  indexPath: IndexPath
  menu: MenuData
  showIcons?: boolean
  menuActiveRef?: React.RefObject<boolean>
  pendingPosRef?: React.RefObject<PendingPos | null>
}

/**
 * Navigate the menu tree following `path` and return the items to display.
 * - Empty path → root (iconItems + groups)
 * - Non-empty → the `.items` of the MenuItem at that path
 */
function navigateMenu(
  menu: MenuData,
  path: IndexPath,
):
  | { type: "root"; iconItems: MenuItem[]; groups: MenuItem[] }
  | { type: "submenu"; items: MenuItem[] }
  | null {
  if (path.length === 0) {
    return { type: "root", iconItems: menu.iconItems, groups: menu.groups }
  }

  const [first, ...rest] = path
  let item: MenuItem | undefined

  if (first === -1) {
    const idx = rest[0]
    if (idx === undefined) return null
    item = menu.iconItems[idx]
    if (!item) return null
    for (let i = 1; i < rest.length; i++) {
      item = item.items[rest[i]]
      if (!item) return null
    }
  } else {
    const groupIdx = first
    const itemIdx = rest[0]
    if (itemIdx === undefined) return null
    item = menu.groups[groupIdx]?.items[itemIdx]
    if (!item) return null
    for (let i = 1; i < rest.length; i++) {
      item = item.items[rest[i]]
      if (!item) return null
    }
  }

  return { type: "submenu", items: item.items || [] }
}

export const ContextMenu: React.FC<ContextMenuProps> = ({
  depth,
  indexPath,
  menu,
  showIcons = false,
  menuActiveRef,
  pendingPosRef,
}) => {
  const rootRef = useRef<HTMLDivElement>(null)
  const measuredRef = useRef(false)

  const resolved = navigateMenu(menu, indexPath)

  /**
   * Measure .rcm-root and the #root container's CSS padding, resize the
   * Tauri window to fit, compute the final position (clamp, flip, edge
   * cases), then position and show the window.
   *
   * The padding is read from computed style at runtime — no hardcoded
   * WINDOW_PADDING constant. Users can freely change `--rcm-window-pad`
   * in CSS and the layout will adapt automatically.
   *
   * Uses offsetWidth/offsetHeight — NOT affected by CSS transforms (the
   * rcm-fade-in animation's scale(0.94) would shrink getBoundingClientRect).
   */
  const resizeAndShow = useCallback(async () => {
    if (!rootRef.current || measuredRef.current) return

    const pos = pendingPosRef?.current
    if (!pos) return

    const rootEl = rootRef.current
    const containerEl = rootEl.parentElement ?? rootEl
    const style = getComputedStyle(containerEl)

    const padLeft = parseFloat(style.paddingLeft) || 0
    const padRight = parseFloat(style.paddingRight) || 0
    const padTop = parseFloat(style.paddingTop) || 0
    const padBottom = parseFloat(style.paddingBottom) || 0

    const rootW = rootEl.offsetWidth
    const rootH = rootEl.offsetHeight

    const w = rootW + padLeft + padRight
    const h = rootH + padTop + padBottom

    measuredRef.current = true
    pendingPosRef.current = null

    const win = getCurrentWindow()
    const dpi = window.devicePixelRatio || 1

    try {
      await win.setSize(new LogicalSize(w, h))
    } catch {
      // Window may not be available yet
    }

    // Compute final window position (clamp, flip, edge cases)
    const monitors = await availableMonitors()
    const { x: finalX, y: finalY } = computeWindowPosition(
      {
        idealX: pos.x,
        idealY: pos.y,
        parentRootX: pos.parentRootX,
        winW: w * dpi,
        winH: h * dpi,
        rootOffsetX: padLeft * dpi,
        rootOffsetY: padTop * dpi,
        rootW: rootW * dpi,
      },
      monitors,
    )

    feLog.info(
      `ContextMenu:d${depth}`,
      `win=(${w}x${h}) pos=(${finalX.toFixed(0)},${finalY.toFixed(0)})`,
    )

    await win.setPosition(new PhysicalPosition(Math.round(finalX), Math.round(finalY)))
    await win.setAlwaysOnTop(true)
    if (menuActiveRef) {
      menuActiveRef.current = true
    }
    await win.show()
    await win.setFocus()
  }, [depth, menuActiveRef, pendingPosRef])

  // Reset measured flag when menu data changes
  useEffect(() => {
    measuredRef.current = false
  }, [menu, indexPath])

  useEffect(() => {
    const raf = requestAnimationFrame(() => {
      resizeAndShow()
    })
    return () => cancelAnimationFrame(raf)
  }, [resolved, resizeAndShow])

  if (!resolved) {
    return <div className="rcm-root" />
  }

  if (resolved.type === "root") {
    const { iconItems, groups } = resolved
    const hasIconItems = iconItems && iconItems.length > 0
    const visibleGroups = groups.filter((g) => g.items && g.items.length > 0)

    return (
      <div className="rcm-root" ref={rootRef} role="menu">
        {showIcons && hasIconItems && <IconRibbon items={iconItems} iconBasePath={-1} />}

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
    )
  }

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
  )
}

import { MenuItemRow } from "./MenuItemRow"

const MenuItemRowWrapper: React.FC<{
  item: MenuItem
  depth: number
  indexPath: IndexPath
  showIcons?: boolean
}> = ({ item, depth, indexPath, showIcons }) => {
  return <MenuItemRow item={item} depth={depth} indexPath={indexPath} showIcons={showIcons} />
}
