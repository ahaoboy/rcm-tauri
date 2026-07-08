import { useEffect, useRef } from "react"

import { getStyleCss } from "./api/menuEvents"
import { ContextMenu } from "./components"
import { useMenuWindow } from "./hooks/useMenuWindow"
import { useTheme } from "./hooks/useTheme"

function App() {
  const theme = useTheme()
  const cssLoaded = useRef(false)
  const { menu, devMode, showIcons, hide, menuActive, pendingPos } = useMenuWindow({
    depth: 0,
    listenIcons: true,
    tag: "App:root",
  })

  // Load dynamic CSS once
  useEffect(() => {
    if (cssLoaded.current) return
    cssLoaded.current = true
    getStyleCss()
      .then((css) => {
        const style = document.createElement("style")
        style.textContent = css
        document.head.appendChild(style)
      })
      .catch(console.error)
  }, [])

  useEffect(() => {
    document.documentElement.classList.remove("rcm-light", "rcm-dark")
    document.documentElement.classList.add(`rcm-${theme}`)
  }, [theme])

  // Disable browser native right-click menu in release mode
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      e.preventDefault()
    }
    document.addEventListener("contextmenu", handler)
    return () => document.removeEventListener("contextmenu", handler)
  }, [])

  // Close all on Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !devMode.current) {
        hide()
      }
    }
    document.addEventListener("keydown", handler)
    return () => document.removeEventListener("keydown", handler)
  }, [devMode, hide])

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
      menuActiveRef={menuActive}
      pendingPosRef={pendingPos}
    />
  )
}

export default App
