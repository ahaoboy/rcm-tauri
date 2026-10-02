/**
 * Route parsing — the single place the URL hash is read.
 *
 * The app is a set of independent Tauri windows. Each one loads
 * `index.html#<hash>` and is pinned to that route for its whole life, so
 * "routing" is one parse at startup rather than a client-side router: there is
 * no in-window navigation to drive.
 *
 * Each window is pinned to its startup route; the only in-window hash change
 * today is the config editor switching tabs. [`getRoute`] parses the hash and
 * caches it, and the cache is invalidated on `hashchange`, so components can
 * call `getRoute()` instead of touching `window.location` themselves.
 *
 * Route shapes:
 *   `#warmup`                hidden WebView2 warm-up window
 *   _(no hash)_              the root menu window
 *   `#submenu-<n>`           submenu window n
 *   `#config/<tab>`          config editor; tab is a file name or `env`
 *   `#error/<json>`          error window; json is a {@link UiError}
 *   `#shell-ext/<report>`    shell-extension diagnostics
 *   `#about`                 about window
 */

import type { UiError } from "./types/error"

/** A parsed route: a discriminated union over the `name`. */
export type Route =
  | { name: "warmup" }
  | { name: "menu-root" }
  | { name: "submenu"; level: number }
  | { name: "config"; tab: string }
  | { name: "error"; error: UiError }
  | { name: "shell-ext"; report: string }
  | { name: "about" }

/** `decodeURIComponent` that returns the input unchanged when malformed. */
function safeDecode(value: string): string {
  try {
    return decodeURIComponent(value)
  } catch {
    return value
  }
}

/**
 * Decode `#error/<json>` into a {@link UiError}.
 *
 * Anything that is not a tagged JSON object degrades to a plain message, so a
 * hand-typed or legacy hash still renders instead of throwing.
 */
function parseError(raw: string): UiError {
  try {
    const data = JSON.parse(safeDecode(raw))
    if (data && typeof data.kind === "string") return data as UiError
  } catch {
    // Not JSON — treat the whole thing as a message.
  }
  return { kind: "message", message: safeDecode(raw) }
}

/** Parse a URL hash into a {@link Route}. Pure — exposed for tests. */
export function parseRoute(hash: string): Route {
  const path = hash.startsWith("#") ? hash.slice(1) : hash

  if (path === "warmup") return { name: "warmup" }
  if (path === "about") return { name: "about" }
  if (path.startsWith("submenu-")) {
    const level = Number.parseInt(path.slice("submenu-".length), 10)
    return { name: "submenu", level: Number.isFinite(level) ? level : 0 }
  }
  if (path.startsWith("config/")) {
    return { name: "config", tab: safeDecode(path.slice("config/".length)) }
  }
  if (path.startsWith("error/")) {
    return { name: "error", error: parseError(path.slice("error/".length)) }
  }
  if (path.startsWith("shell-ext/")) {
    return { name: "shell-ext", report: safeDecode(path.slice("shell-ext/".length)) }
  }
  return { name: "menu-root" }
}

let cached: Route | null = null

/**
 * This window's current route.
 *
 * Parsed from `window.location.hash` and cached, because most windows never
 * change their hash and re-parsing would hand React a new object identity on
 * every call. The cache is dropped whenever the hash changes (see below), so
 * the result always reflects the current URL.
 */
export function getRoute(): Route {
  return (cached ??= parseRoute(window.location.hash))
}

/**
 * Update the URL hash without navigating.
 *
 * Only used for in-window state worth reflecting in the URL — currently the
 * active config tab. The cache is refreshed in the same tick so callers that
 * read [`getRoute`] from a `hashchange` handler (in any listener order) see the
 * new route immediately.
 */
export function setRoute(hash: string): void {
  const normalized = hash.startsWith("#") ? hash.slice(1) : hash
  window.location.hash = normalized
  cached = parseRoute(normalized)
}

// External hash edits (address bar, other scripts) must not leave a stale cache.
window.addEventListener("hashchange", () => {
  cached = null
})
