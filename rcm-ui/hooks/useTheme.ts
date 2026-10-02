/**
 * useTheme — the resolved theme for this window.
 *
 * Reads from the config store (which owns fetching and live updates) and
 * applies the theme class to `<html>` whenever it changes, so callers only
 * need `useTheme()` for the side effect to run.
 */

import { useEffect } from "react"

import { useConfigStore, type Theme } from "../stores"

export function useTheme(): Theme {
  const theme = useConfigStore((s) => s.theme)

  useEffect(() => {
    const root = document.documentElement
    root.classList.remove("rcm-light", "rcm-dark")
    root.classList.add(`rcm-${theme}`)
  }, [theme])

  return theme
}
