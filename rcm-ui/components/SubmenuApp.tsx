/**
 * SubmenuApp — renders a single level of submenu items in its own window.
 *
 * Windows are pre-created by Rust, labeled `submenu-0` … `submenu-3`. Each
 * window receives `menu-show` events from Rust with the full menu data and an
 * index path telling it which submenu to render.
 *
 * Positioning is owned by Rust: this component measures the rendered level and
 * reports the size, and Rust clamps/flips, resizes, moves and reveals the window.
 */

import { useMenuWindow } from "../hooks/useMenuWindow"
import { useTheme } from "../hooks/useTheme"
import { getRoute } from "../router"
import { useMenuStore } from "../stores"
import { ContextMenu } from "./ContextMenu"

export function SubmenuApp() {
  useTheme()

  const route = getRoute()
  const level = route.name === "submenu" ? route.level : 0
  const depth = level + 1

  const menu = useMenuStore((s) => s.menu)
  const indexPath = useMenuStore((s) => s.indexPath)
  useMenuWindow({ depth, tag: `App:submenu-${level}` })

  if (!menu) {
    return <div className="rcm-root" />
  }

  return <ContextMenu depth={depth} indexPath={indexPath} menu={menu} showIcons={false} />
}
