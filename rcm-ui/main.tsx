import React from "react"
import ReactDOM from "react-dom/client"

import App from "./App"
import { ConfigEditor } from "./components/ConfigEditor"
import { ErrorPage } from "./components/ErrorPage"
import { SubmenuApp } from "./components/SubmenuApp"
import { WarmupPage } from "./components/WarmupPage"

const root = document.getElementById("root") as HTMLElement
const hash = window.location.hash

const routes: [string | null, React.FC][] = [
  ["#warmup", WarmupPage],
  ["#submenu-", SubmenuApp],
  ["#config/", ConfigEditor],
  ["#error/", ErrorPage],
  [null, App],
]

const [, Page] = routes.find(([prefix]) => (prefix ? hash.startsWith(prefix) : true))!

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <Page />
  </React.StrictMode>,
)
