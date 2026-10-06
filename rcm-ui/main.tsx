import { listen } from "@tauri-apps/api/event"
import React from "react"
import ReactDOM from "react-dom/client"

import { getStyleCss } from "./api/menuEvents"
import App from "./App"
import { ConfigEditor } from "./components/ConfigEditor"
import { ErrorPage } from "./components/ErrorPage"
import { ShellExtensionPage } from "./components/ShellExtensionPage"
import { SubmenuApp } from "./components/SubmenuApp"
import { WarmupPage } from "./components/WarmupPage"
import { getRoute, type Route } from "./router"
import { useConfigStore } from "./stores"

/**
 * One page per route. `needsCss` marks the windows that load the user's
 * `style.css`; standalone pages ship their own styles.
 */
const PAGES: Record<Route["name"], { Page: React.FC; needsCss: boolean }> = {
  warmup: { Page: WarmupPage, needsCss: false },
  "menu-root": { Page: App, needsCss: true },
  submenu: { Page: SubmenuApp, needsCss: true },
  config: { Page: ConfigEditor, needsCss: false },
  error: { Page: ErrorPage, needsCss: false },
  "shell-ext": { Page: ShellExtensionPage, needsCss: false },
}

const { Page, needsCss } = PAGES[getRoute().name]

// Load shared config (dev mode, icons, theme) once for this window.
useConfigStore.getState().init()

// Menu windows need the user's style.css; standalone pages use their own styles.
if (needsCss) {
  const applyCss = (css: string) => {
    const existing = document.getElementById("rcm-style") as HTMLStyleElement | null
    if (existing) {
      existing.textContent = css
      return
    }
    const el = document.createElement("style")
    el.id = "rcm-style"
    el.textContent = css
    document.head.appendChild(el)
  }
  getStyleCss().then(applyCss).catch(console.error)
  // Live reload when style.css is saved in the ConfigEditor.
  listen<string>("style-changed", (e) => applyCss(e.payload))
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Page />
  </React.StrictMode>,
)
