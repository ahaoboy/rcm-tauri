/**
 * useMenuWindow — unified Tauri event handling for a menu window.
 *
 * Encapsulates ALL event listening and emission for one menu level (root or
 * submenu). Components never import `@tauri-apps/api/event` directly; everything
 * goes through here and `api/menuEvents`.
 *
 * ## What this hook does *not* do
 *
 * It never computes a window position and never moves or sizes the window. Rust
 * owns both: the placement algorithm lives in `rcm_core::ui` and is applied by
 * the Rust-side `TauriHost`. The frontend's only geometric responsibility is
 * measuring what it drew and reporting that back — see `ContextMenu` +
 * `utils/measure`.
 *
 * Responsibilities:
 *   - fetch initial config (dev mode, icons)
 *   - listen for `menu-show` (filtered by depth)
 *   - listen for `menu-hide-all`, `dev-mode`, `icons-changed`
 *   - emit `menu-blur` on focus loss (when the menu is active)
 *   - prevent window close → hide instead
 *
 * The one exception is parking the window off-screen on mount: Rust reveals the
 * window after measurement, and a stale frame must not be visible in the
 * meantime.
 */

import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window"
import { useCallback, useEffect, useRef, useState } from "react"

import { getConfig, onDevMode, onIconsChanged, onMenuHideAll, onMenuShow } from "../api/menuEvents"
import { feLog } from "../feLog"
import type { MenuData, IndexPath } from "../types/menu"

const OFF_SCREEN = new PhysicalPosition(-9999, -9999)

export interface MenuWindowState {
  menu: MenuData | null
  indexPath: IndexPath
  devMode: React.RefObject<boolean>
  showIcons: boolean
  /** Hide the window and clear React state. No-op in dev mode. */
  hide: () => Promise<void>
}

export interface UseMenuWindowOptions {
  /** Window depth: 0 = root, 1+ = submenu. */
  depth: number
  /** Listen for `icons-changed` events (root only). */
  listenIcons?: boolean
  /** Tag for log messages. */
  tag: string
}

export function useMenuWindow(options: UseMenuWindowOptions): MenuWindowState {
  const { depth, listenIcons = false, tag } = options

  const [menu, setMenu] = useState<MenuData | null>(null)
  const [indexPath, setIndexPath] = useState<IndexPath>([])
  const [showIcons, setShowIcons] = useState(false)
  const devMode = useRef(false)

  useEffect(() => {
    const win = getCurrentWindow()
    const cleanups: (() => void)[] = []

    // Park off-screen so a stale frame is never visible while the next level
    // renders. Rust reveals the window once it has been measured.
    win.setPosition(OFF_SCREEN).catch(() => {})

    const setup = async () => {
      // ── Fetch initial config ──────────────────────────────────
      try {
        const cfg = await getConfig()
        devMode.current = cfg.dev
        if (listenIcons) {
          setShowIcons(cfg.icons)
        }
      } catch {
        /* ignore */
      }

      // ── Prevent window close, just hide it ───────────────────
      const unlistenClose = await win.onCloseRequested(async (e) => {
        e.preventDefault()
        await hide()
      })
      cleanups.push(unlistenClose)

      // ── Dev mode toggle ──────────────────────────────────────
      const unlistenDev = await onDevMode((dev) => {
        devMode.current = dev
      })
      cleanups.push(unlistenDev)

      // ── Icons toggle (root only) ─────────────────────────────
      if (listenIcons) {
        const unlistenIcons = await onIconsChanged((icons) => {
          setShowIcons(icons)
        })
        cleanups.push(unlistenIcons)
      }

      // Dismissal is not event-driven: Rust polls which window holds focus. A
      // blur cannot tell "focus moved to another menu window" from "focus left
      // the menu", so emitting one only caused spurious dismissals.

      // ── Rust → Frontend: render this level ───────────────────
      const unlistenShow = await onMenuShow((payload) => {
        const { menu: menuData, path } = payload

        // Depth filter: path.length-1 = event depth (0 for root)
        const eventDepth = path.length === 0 ? 0 : path.length - 1
        if (eventDepth !== depth) {
          feLog.info(tag, `menu-show SKIP (eventDepth=${eventDepth} != depth=${depth})`)
          return
        }

        feLog.eventRecv("menu-show", `path=[${path}]`)

        // No position is involved. `ContextMenu` measures after render and
        // reports the size; Rust then positions and reveals the window.
        setMenu(menuData)
        setIndexPath(path)
      })
      cleanups.push(unlistenShow)

      // ── Rust → Frontend: hide all ─────────────────────────────
      const unlistenHide = await onMenuHideAll(() => {
        feLog.eventRecv("menu-hide-all", tag)
        if (devMode.current) {
          feLog.info(tag, "dev mode, ignoring hide")
          return
        }
        hide()
      })
      cleanups.push(unlistenHide)
    }

    setup()

    return () => {
      cleanups.forEach((fn) => fn())
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [depth])

  const hide = useCallback(async () => {
    if (devMode.current) return
    const win = getCurrentWindow()
    await win.hide()
    await win.setPosition(OFF_SCREEN)
    setMenu(null)
    setIndexPath([])
  }, [])

  return { menu, indexPath, devMode, showIcons, hide }
}
