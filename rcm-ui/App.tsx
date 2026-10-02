import { useEffect } from "react"

import { ContextMenu } from "./components"
import { useMenuWindow } from "./hooks/useMenuWindow"
import { useTheme } from "./hooks/useTheme"
import { useConfigStore, useMenuStore } from "./stores"

function App() {
  useTheme()

  const menu = useMenuStore((s) => s.menu)
  const showIcons = useConfigStore((s) => s.icons)
  const { hide } = useMenuWindow({ depth: 0, tag: "App:root" })

  // Disable the browser's native right-click menu.
  useEffect(() => {
    const handler = (e: MouseEvent) => e.preventDefault()
    document.addEventListener("contextmenu", handler)
    return () => document.removeEventListener("contextmenu", handler)
  }, [])

  // Close on Escape (unless dev mode keeps the menu pinned).
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !useConfigStore.getState().dev) hide()
    }
    document.addEventListener("keydown", handler)
    return () => document.removeEventListener("keydown", handler)
  }, [hide])

  if (!menu) {
    return <div className="rcm-root" />
  }

  return (
    <ContextMenu
      key={showIcons ? "icons" : "no-icons"}
      depth={0}
      indexPath={[]}
      menu={menu}
      showIcons={showIcons}
    />
  )
}

export default App
