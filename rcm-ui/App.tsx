import { invoke } from "@tauri-apps/api/core"
import { listen, emit } from "@tauri-apps/api/event"
import { availableMonitors, getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window"
import { useEffect, useRef, useState, useCallback } from "react"

import { ContextMenu } from "./components"
import { chooseMonitorForPoint, clampWindowToMonitor } from "./constants/layout"
import { feLog } from "./feLog"
import { useTheme } from "./hooks/useTheme"
import type { MenuData, MenuShowPayload } from "./types/menu"

/** Off-screen position used to hide the window without flicker. */
const OFF_SCREEN = new PhysicalPosition(-9999, -9999)

/** Depth of this window. Root = 0. */
const MY_DEPTH = 0

function App() {
  const [menu, setMenu] = useState<MenuData | null>(null)
  const [showIcons, setShowIcons] = useState(false)
  const devMode = useRef(false)
  const menuActive = useRef(false)
  const theme = useTheme()
  /** Pending ideal position from menu-show event; consumed by onReady. */
  const pendingPos = useRef<{ x: number; y: number } | null>(null)

  useEffect(() => {
    document.documentElement.classList.remove("rcm-light", "rcm-dark")
    document.documentElement.classList.add(`rcm-${theme}`)
  }, [theme])

  // ── Disable browser native right-click menu in release mode ──
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      e.preventDefault()
    }
    document.addEventListener("contextmenu", handler)
    return () => document.removeEventListener("contextmenu", handler)
  }, [])

  useEffect(() => {
    let cleanupFns: (() => void)[] = []
    const win = getCurrentWindow()

    // Start off-screen
    win.setPosition(OFF_SCREEN).catch(() => {})

    const setup = async () => {
      // ── Fetch initial config ──────────────────────────────────
      try {
        const cfg = await invoke<{ dev: boolean; icons: boolean }>("get_config")
        devMode.current = cfg.dev
        setShowIcons(cfg.icons)
      } catch {
        /* ignore */
      }

      // ── Prevent window close, just hide it ───────────────────
      const unlistenClose = await win.onCloseRequested(async (e) => {
        e.preventDefault()
        await hideRoot(win)
      })
      cleanupFns.push(unlistenClose)

      // ── Dev mode toggle ──────────────────────────────────────
      const unlistenDev = await listen<boolean>("dev-mode", (event) => {
        devMode.current = event.payload
      })
      cleanupFns.push(unlistenDev)

      // ── Blur → Rust decides whether to hide all ──────────────
      const unlistenFocus = await win.onFocusChanged(({ payload: focused }) => {
        feLog.info("App:root", `onFocusChanged focused=${focused} menuActive=${menuActive.current}`)
        if (!focused && !devMode.current && menuActive.current) {
          feLog.eventSend("menu-blur", "depth=0")
          emit("menu-blur", { depth: MY_DEPTH })
        }
      })
      cleanupFns.push(unlistenFocus)

      // ── Icons toggle ─────────────────────────────────────────
      const unlistenIcons = await listen<boolean>("icons-changed", (event) => {
        setShowIcons(event.payload)
      })
      cleanupFns.push(unlistenIcons)

      // ── Rust → Frontend: show menu at root level ─────────────
      const unlistenShow = await listen<MenuShowPayload>("menu-show", (event) => {
        const { menu: menuData, path, x, y } = event.payload
        // Only process root-level events (empty path)
        if (path.length !== 0) return

        feLog.eventRecv(
          "menu-show",
          `pos=(${x.toFixed(0)},${y.toFixed(0)}) groups=${menuData.groups.length}`,
        )

        pendingPos.current = { x, y }
        setMenu(menuData)
        // Window is moved off-screen initially; onReady will position & show
      })
      cleanupFns.push(unlistenShow)

      // ── Rust → Frontend: hide all ────────────────────────────
      const unlistenHide = await listen("menu-hide-all", () => {
        feLog.eventRecv("menu-hide-all", "")
        if (devMode.current) {
          feLog.info("App:root", "dev mode, ignoring hide")
          return
        }
        hideRoot(win)
      })
      cleanupFns.push(unlistenHide)
    }

    setup()

    return () => {
      cleanupFns.forEach((fn) => fn())
    }
  }, [])

  /** Called by ContextMenu after the window has been resized to fit content. */
  const handleReady = useCallback(async () => {
    const pos = pendingPos.current
    if (!pos) return
    pendingPos.current = null

    try {
      const win = getCurrentWindow()
      const outerSize = await win.outerSize()
      const monitors = await availableMonitors()
      const monitor = chooseMonitorForPoint(monitors, pos.x, pos.y)

      let finalX = pos.x
      let finalY = pos.y

      if (monitor) {
        const monRight = monitor.position.x + monitor.size.width
        // If menu overflows right edge, flip to the left of the cursor
        if (finalX + outerSize.width > monRight) {
          finalX = pos.x - outerSize.width
          feLog.info("App:root", `flip: right overflow → left_side=${finalX.toFixed(0)}`)
        }

        const clamped = clampWindowToMonitor(finalX, finalY, outerSize, monitor)
        finalX = clamped.x
        finalY = clamped.y

        feLog.info(
          "App:root",
          `clamp: size=(${outerSize.width}x${outerSize.height}) raw=(${pos.x.toFixed(0)},${pos.y.toFixed(0)}) -> (${finalX.toFixed(0)},${finalY.toFixed(0)})`,
        )
      }

      await win.setPosition(new PhysicalPosition(Math.round(finalX), Math.round(finalY)))
      await win.setAlwaysOnTop(true)
      menuActive.current = true
      feLog.info("App:root", "menuActive armed")
      await win.show()
      await win.setFocus()
    } catch (e) {
      feLog.error("App:root", `handleReady error: ${e}`)
    }
  }, [])

  /** Hide the root window and reset state. */
  async function hideRoot(win: ReturnType<typeof getCurrentWindow>) {
    if (devMode.current) return
    menuActive.current = false
    await win.hide()
    await win.setPosition(OFF_SCREEN)
    setMenu(null)
  }

  if (!menu) {
    return <div className="rcm-root" />
  }

  return (
    <ContextMenu
      key={showIcons ? "icons" : "no-icons"}
      depth={MY_DEPTH}
      indexPath={[]}
      menu={menu}
      showIcons={showIcons}
      onReady={handleReady}
    />
  )
}

export default App
