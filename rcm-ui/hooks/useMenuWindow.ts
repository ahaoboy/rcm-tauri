/**
 * useMenuWindow — unified hook for menu window event handling.
 *
 * Encapsulates ALL Tauri event listening and emission for a menu window
 * (root or submenu). Components no longer import `@tauri-apps/api/event`
 * directly — everything goes through this hook and the `api/menuEvents`
 * module.
 *
 * Responsibilities:
 *   - Fetch initial config (dev mode, icons)
 *   - Listen for `menu-show` (filtered by depth), store pending position
 *   - Listen for `menu-hide-all`, `dev-mode`, `icons-changed`
 *   - Emit `menu-blur` on focus loss (when menu is active)
 *   - Prevent window close → hide instead
 *   - Position window off-screen on mount (prevent startup flicker)
 *
 * Layout positioning is handled by the frontend: ContextMenu measures the
 * DOM, computes the final position via `utils/layout`, and shows the window.
 */

import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window"
import { useCallback, useEffect, useRef, useState } from "react"

import {
  emitMenuBlur,
  getConfig,
  onDevMode,
  onIconsChanged,
  onMenuHideAll,
  onMenuShow,
} from "../api/menuEvents"
import { feLog } from "../feLog"
import type { MenuData, IndexPath } from "../types/menu"
import type { PendingPos } from "../utils/layout"

const OFF_SCREEN = new PhysicalPosition(-9999, -9999)

export interface MenuWindowState {
  menu: MenuData | null
  indexPath: IndexPath
  devMode: React.RefObject<boolean>
  showIcons: boolean
  /** Hide the window and clear React state. No-op in dev mode. */
  hide: () => Promise<void>
  /** Whether the menu is currently shown (armed for blur detection). */
  menuActive: React.RefObject<boolean>
  /** Ideal position from `menu-show` — consumed by ContextMenu after measuring. */
  pendingPos: React.RefObject<PendingPos | null>
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
  const menuActive = useRef(false)
  const pendingPos = useRef<PendingPos | null>(null)

  useEffect(() => {
    const win = getCurrentWindow()
    const cleanups: (() => void)[] = []

    // Start off-screen to prevent startup flicker
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

      // ── Blur → Rust decides whether to hide all ──────────────
      const unlistenFocus = await win.onFocusChanged(({ payload: focused }) => {
        feLog.info(tag, `onFocusChanged focused=${focused} menuActive=${menuActive.current}`)
        if (!focused && !devMode.current && menuActive.current) {
          feLog.eventSend("menu-blur", `depth=${depth}`)
          emitMenuBlur(depth)
        }
      })
      cleanups.push(unlistenFocus)

      // ── Rust → Frontend: show menu at this depth ─────────────
      const unlistenShow = await onMenuShow((payload) => {
        const { menu: menuData, path, x, y, parentRootX } = payload

        // Depth filter: path.length-1 = event depth (0 for root)
        const eventDepth = path.length === 0 ? 0 : path.length - 1
        if (eventDepth !== depth) {
          feLog.info(tag, `menu-show SKIP (eventDepth=${eventDepth} != depth=${depth})`)
          return
        }

        feLog.eventRecv("menu-show", `pos=(${x.toFixed(0)},${y.toFixed(0)}) path=[${path}]`)

        // Store position for ContextMenu to consume after measuring
        pendingPos.current = { x, y, parentRootX }
        setMenu(menuData)
        setIndexPath(path)
      })
      cleanups.push(unlistenShow)

      // ── Rust → Frontend: hide all ──────────────────────────────
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
    menuActive.current = false
    pendingPos.current = null
    const win = getCurrentWindow()
    await win.hide()
    await win.setPosition(OFF_SCREEN)
    setMenu(null)
    setIndexPath([])
  }, [])

  return { menu, indexPath, devMode, showIcons, hide, menuActive, pendingPos }
}
