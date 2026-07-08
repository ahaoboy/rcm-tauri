import { listen } from "@tauri-apps/api/event"
import React from "react"
import ReactDOM from "react-dom/client"

import { getStyleCss } from "./api/menuEvents"
import App from "./App"
import { ConfigEditor } from "./components/ConfigEditor"
import { ErrorPage } from "./components/ErrorPage"
import { SubmenuApp } from "./components/SubmenuApp"
import { WarmupPage } from "./components/WarmupPage"

const root = document.getElementById("root") as HTMLElement
const hash = window.location.hash

const routes: [string | null, React.FC, boolean][] = [
  ["#warmup", WarmupPage, false],
  ["#submenu-", SubmenuApp, true],
  ["#config/", ConfigEditor, false],
  ["#error/", ErrorPage, false],
  [null, App, true],
]

const [, Page, needsCss] = routes.find(([prefix]) => (prefix ? hash.startsWith(prefix) : true))!

// Menu windows (App, Submenu) need dynamic CSS; standalone pages don't
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
  // Live reload when style.css is saved in ConfigEditor
  listen<string>("style-changed", (e) => applyCss(e.payload))
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <Page />
  </React.StrictMode>,
)
