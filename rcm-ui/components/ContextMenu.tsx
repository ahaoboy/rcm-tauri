import React, { useCallback, useEffect, useRef } from "react"

import { emitMenuMeasured } from "../api/menuEvents"
import { feLog } from "../feLog"
import type { MenuData, MenuItem, IndexPath } from "../types/menu"
import { measureLevel } from "../utils/measure"
import { IconRibbon } from "./IconRibbon"
import { MenuGroup } from "./MenuGroup"
import { MenuSeparator } from "./MenuSeparator"
import { MenuItemRow } from "./MenuItemRow"

interface ContextMenuProps {
  depth: number
  indexPath: IndexPath
  menu: MenuData
  showIcons?: boolean
  menuActiveRef?: React.RefObject<boolean>
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

/**
 * ContextMenu — renders one menu level.
 *
 * It draws the level and, after layout, measures it and reports the size to
 * Rust. Rust owns the position: it clamps/flips the window, resizes, moves and
 * reveals it. Nothing here reads or writes window geometry beyond that single
 * measurement.
 */
export const ContextMenu: React.FC<ContextMenuProps> = ({
  depth,
  indexPath,
  menu,
  showIcons = false,
  menuActiveRef,
}) => {
  const rootRef = useRef<HTMLDivElement>(null)
  const measuredRef = useRef(false)

  const resolved = navigateMenu(menu, indexPath)

  /**
   * Measure `.rcm-root` (plus the container's padding) and report it.
   *
   * Uses `offsetWidth` / `offsetHeight` via `measureLevel` — not
   * `getBoundingClientRect`, which a CSS transform would shrink. The padding is
   * read from computed style, so changing `--rcm-window-pad` still works.
   */
  const measureAndReport = useCallback(() => {
    if (!rootRef.current || measuredRef.current) return
    measuredRef.current = true

    const measurement = measureLevel(rootRef.current)
    feLog.info(
      `ContextMenu:d${depth}`,
      `measured win=${measurement.winW}x${measurement.winH} root=${measurement.rootW}x${measurement.rootH}`,
    )
    emitMenuMeasured({ depth, ...measurement })

    // The menu is now interactive; arm blur detection.
    if (menuActiveRef) {
      menuActiveRef.current = true
    }
  }, [depth, menuActiveRef])

  // Re-measure whenever the rendered level changes.
  useEffect(() => {
    measuredRef.current = false
  }, [menu, indexPath])

  useEffect(() => {
    const raf = requestAnimationFrame(measureAndReport)
    return () => cancelAnimationFrame(raf)
  }, [resolved, measureAndReport])

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
        <MenuItemRow
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
