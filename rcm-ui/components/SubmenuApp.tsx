/**
 * SubmenuApp — renders a single level of submenu items in its own window.
 *
 * Windows are pre-created by Rust, labeled `submenu-0` … `submenu-3`.
 * Each window receives `menu-show` events from Rust with the full menu
 * data and an index path telling it which submenu to render.
 *
 * All hover/click coordination happens through Rust via events.
 * Positioning is handled by the frontend: ContextMenu measures the DOM
 * and computes the final window position.
 */

import { useEffect } from "react"

import { useMenuWindow } from "../hooks/useMenuWindow"
import { useTheme } from "../hooks/useTheme"
import { ContextMenu } from "./ContextMenu"

export function SubmenuApp() {
  const theme = useTheme()

  // Depth from URL hash: "submenu-0" → depth 1, "submenu-1" → depth 2, etc.
  const myLevel = parseInt(window.location.hash.replace("#submenu-", ""), 10) || 0
  const depth = myLevel + 1

  const { menu, indexPath, menuActive, pendingPos } = useMenuWindow({
    depth,
    tag: `App:submenu-${myLevel}`,
  })

  useEffect(() => {
    document.documentElement.classList.remove("rcm-light", "rcm-dark")
    document.documentElement.classList.add(`rcm-${theme}`)
  }, [theme])

  if (!menu) {
    return <div className="rcm-root" />
  }

  return (
    <ContextMenu
      depth={depth}
      indexPath={indexPath}
      menu={menu}
      showIcons={false}
      menuActiveRef={menuActive}
      pendingPosRef={pendingPos}
    />
  )
}
