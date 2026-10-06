/**
 * Config store — the app settings every window shares.
 *
 * Previously each hook fetched `get_config` and subscribed to its own events
 * (`useTheme`, `useMenuWindow`), which meant several concurrent `get_config`
 * calls and duplicated listeners per window. This store fetches once and is the
 * single subscriber.
 *
 * Read it reactively with a selector (`useConfigStore((s) => s.dev)`) or
 * imperatively outside render with `useConfigStore.getState()` — the latter is
 * what event handlers and async callbacks should use, so they never capture a
 * stale value.
 */

import { create } from "zustand"

import { getConfig, onDevMode, onIconsChanged, onThemeChanged } from "../api/menuEvents"

/** The theme actually applied to the DOM. */
export type Theme = "light" | "dark"

interface ConfigState {
  /** Developer mode keeps menus open so they can be inspected. */
  dev: boolean
  /** Whether menu rows render their icons. */
  icons: boolean
  /** Preference as configured: `"system" | "light" | "dark"`. */
  themePreference: string
  /** Resolved theme, ready to apply to `<html>`. */
  theme: Theme
  /** True once the initial `get_config` has settled (success or failure). */
  ready: boolean
  /** Load config and subscribe to changes. Idempotent — safe to call often. */
  init: () => void
  /**
   * Apply a theme preference locally, without writing it.
   *
   * Used for the optimistic update when the settings tab changes the theme, so
   * this window re-themes at once instead of waiting for Rust to echo the change
   * back through `theme-changed`.
   */
  setTheme: (preference: string) => void
}

/** Resolve a preference (possibly `"system"`) to a concrete theme. */
function resolveTheme(preference: string): Theme {
  if (preference === "light" || preference === "dark") return preference
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
}

/** Guards `init` so its listeners are wired exactly once. */
let started = false

export const useConfigStore = create<ConfigState>()((set) => ({
  dev: false,
  icons: false,
  themePreference: "system",
  theme: resolveTheme("system"),
  ready: false,

  setTheme: (preference) => set({ themePreference: preference, theme: resolveTheme(preference) }),

  init: () => {
    if (started) return
    started = true

    const applyTheme = (preference: string) =>
      set({ themePreference: preference, theme: resolveTheme(preference) })

    getConfig()
      .then((cfg) => {
        set({ dev: cfg.dev, icons: cfg.icons, ready: true })
        applyTheme(cfg.theme)
      })
      .catch(() => set({ ready: true }))

    // Live updates pushed by Rust (tray toggles, config reloads).
    void onDevMode((dev) => set({ dev }))
    void onIconsChanged((icons) => set({ icons }))
    void onThemeChanged(applyTheme)

    // While the preference stays on "system", follow the OS setting.
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      const { themePreference } = useConfigStore.getState()
      if (themePreference !== "light" && themePreference !== "dark") {
        set({ theme: resolveTheme(themePreference) })
      }
    })
  },
}))
