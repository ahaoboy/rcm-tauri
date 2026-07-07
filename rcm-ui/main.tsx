import React from "react"
import ReactDOM from "react-dom/client"

import { getStyleCss } from "./api/menuEvents"
import App from "./App"
import { ConfigEditor } from "./components/ConfigEditor"
import { ErrorPage } from "./components/ErrorPage"
import { SubmenuApp } from "./components/SubmenuApp"

const root = document.getElementById("root") as HTMLElement
const hash = window.location.hash
const isSubmenu = hash.startsWith("#submenu-")
const isConfig = hash.startsWith("#config/")
const isError = hash.startsWith("#error/")

// Load CSS dynamically from the Rust backend (skip for config/error windows).
if (!isConfig && !isError) {
  getStyleCss()
    .then((css) => {
      const style = document.createElement("style")
      style.textContent = css
      document.head.appendChild(style)
    })
    .catch(console.error)
}

function render() {
  if (isError) {
    return <ErrorPage />
  }
  if (isConfig) {
    return <ConfigEditor />
  }
  if (isSubmenu) {
    return <SubmenuApp />
  }
  return <App />
}

ReactDOM.createRoot(root).render(<React.StrictMode>{render()}</React.StrictMode>)
