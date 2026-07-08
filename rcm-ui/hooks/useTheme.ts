import { listen } from "@tauri-apps/api/event"
import { useEffect, useState } from "react"

import { getConfig } from "../api/menuEvents"

/** Resolve the effective theme from config + system preference. */
function resolve(cfgTheme: string): "light" | "dark" {
  if (cfgTheme === "light") return "light"
  if (cfgTheme === "dark") return "dark"
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
}

/**
 * Returns the active menu theme.
 * Respects the config preference (system / light / dark),
 * system preference changes, and manual tray toggles.
 */
export function useTheme(): "light" | "dark" {
  const [theme, setTheme] = useState<"light" | "dark">(() => resolve("system"))

  useEffect(() => {
    getConfig()
      .then((cfg) => setTheme(resolve(cfg.theme)))
      .catch(() => {})

    const mq = window.matchMedia("(prefers-color-scheme: dark)")
    const onSysChange = () => {
      getConfig()
        .then((cfg) => setTheme(resolve(cfg.theme)))
        .catch(() => {})
    }
    mq.addEventListener("change", onSysChange)

    const unlisten = listen<string>("theme-changed", (e) => {
      setTheme(resolve(e.payload))
    })

    return () => {
      mq.removeEventListener("change", onSysChange)
      unlisten.then((fn) => fn())
    }
  }, [])

  return theme
}
