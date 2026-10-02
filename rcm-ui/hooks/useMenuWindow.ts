/**
 * useMenuWindow — wires the Tauri events for one menu window.
 *
 * This hook does not own state: it writes into `useMenuStore` (what to render)
 * and reads `useConfigStore` (dev mode). Components read what they need from the
 * stores directly, so this hook re-rendering never forces a component that only
 * cares about, say, the theme to re-render too.
 *
 * It never computes a window position and never moves or sizes the window. Rust
 * owns both: the placement algorithm lives in `rcm_core::ui` and is applied by
 * the Rust-side host. The frontend's only geometric responsibility is measuring
 * what it drew and reporting that back — see `ContextMenu` + `utils/measure`.
 *
 * Responsibilities:
 *   - listen for `menu-show` (filtered by depth) → store
 *   - listen for `menu-hide-all` → hide
 *   - prevent window close → hide instead
 *   - park the window off-screen on mount (Rust reveals it after measuring)
 */

import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window"
import { useCallback, useEffect } from "react"

import { onMenuHideAll, onMenuShow } from "../api/menuEvents"
import { feLog } from "../feLog"
import { useConfigStore, useMenuStore } from "../stores"

const OFF_SCREEN = new PhysicalPosition(-9999, -9999)

export interface UseMenuWindowOptions {
  /** Window depth: 0 = root, 1+ = submenu. */
  depth: number
  /** Tag for log messages. */
  tag: string
}

export interface MenuWindowHandle {
  /** Hide the window, park it off-screen and clear the rendered level. */
  hide: () => Promise<void>
}

export function useMenuWindow({ depth, tag }: UseMenuWindowOptions): MenuWindowHandle {
  // Stable for the component's lifetime (empty dependency list), so the event
  // listeners below can capture it without ever going stale.
  const hide = useCallback(async () => {
    if (useConfigStore.getState().dev) return
    const win = getCurrentWindow()
    await win.hide()
    await win.setPosition(OFF_SCREEN)
    useMenuStore.getState().clear()
  }, [])

  useEffect(() => {
    const win = getCurrentWindow()
    const cleanups: (() => void)[] = []

    // Park off-screen so a stale frame is never visible while the next level
    // renders. Rust reveals the window once it has been measured.
    win.setPosition(OFF_SCREEN).catch(() => {})

    const setup = async () => {
      // ── Prevent window close, just hide it ───────────────────
      cleanups.push(
        await win.onCloseRequested(async (e) => {
          e.preventDefault()
          await hide()
        }),
      )

      // ── Rust → Frontend: render this level ───────────────────
      cleanups.push(
        await onMenuShow(({ menu, path }) => {
          // Depth filter: path.length-1 = event depth (0 for root).
          const eventDepth = path.length === 0 ? 0 : path.length - 1
          if (eventDepth !== depth) {
            feLog.info(tag, `menu-show SKIP (eventDepth=${eventDepth} != depth=${depth})`)
            return
          }
          feLog.eventRecv("menu-show", `path=[${path}]`)
          useMenuStore.getState().setLevel(menu, path)
        }),
      )

      // ── Rust → Frontend: hide all ─────────────────────────────
      cleanups.push(
        await onMenuHideAll(() => {
          feLog.eventRecv("menu-hide-all", tag)
          if (useConfigStore.getState().dev) {
            feLog.info(tag, "dev mode, ignoring hide")
            return
          }
          void hide()
        }),
      )

      // Dismissal is not event-driven: Rust polls which window holds focus. A
      // blur cannot tell "focus moved to another menu window" from "focus left
      // the menu", so emitting one only caused spurious dismissals.
    }
    void setup()

    return () => cleanups.forEach((fn) => fn())
  }, [depth, tag, hide])

  return { hide }
}
