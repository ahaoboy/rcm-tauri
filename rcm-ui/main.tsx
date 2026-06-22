import { invoke } from "@tauri-apps/api/core"
import React from "react"
import ReactDOM from "react-dom/client"

import App from "./App"
import { SubmenuApp } from "./components/SubmenuApp"

const root = document.getElementById("root") as HTMLElement
const isSubmenu = window.location.hash.startsWith("#submenu-")

// Load CSS dynamically from the Rust backend:
// - If style.css exists next to the exe, it is used.
// - Otherwise the default style.css is written next to the exe and loaded.
async function loadStyle() {
  try {
    const css = await invoke<string>("get_style_css")
    const style = document.createElement("style")
    style.textContent = css
    document.head.appendChild(style)
  } catch (e) {
    console.error(e)
  }
}

loadStyle()

ReactDOM.createRoot(root).render(
  <React.StrictMode>{isSubmenu ? <SubmenuApp /> : <App />}</React.StrictMode>,
)
