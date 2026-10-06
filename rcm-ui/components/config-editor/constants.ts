/**
 * Shared constants & routing helpers for the config editor tabs.
 */

import { css } from "@codemirror/lang-css"
import { javascript } from "@codemirror/lang-javascript"
import { json } from "@codemirror/lang-json"
import type { Extension } from "@codemirror/state"

import { getRoute } from "../../router"

/** Editable config files shown as tabs. */
export const FILES = [
  { key: "rcm.config.json", label: "rcm.config.json", lang: "json" },
  { key: "rcm.js", label: "rcm.js", lang: "javascript" },
  { key: "style.css", label: "style.css", lang: "css" },
] as const

export type FileKey = (typeof FILES)[number]["key"]

/** Special tab: read-only environment variable inspector (not a file). */
export const ENV_TAB = "env"

/** Special tab: the tray's settings as a form (not a file). */
export const SETTINGS_TAB = "settings"

/** Special tab: application identity, version and paths (not a file). */
export const ABOUT_TAB = "about"

export type TabKey = FileKey | typeof ENV_TAB | typeof SETTINGS_TAB | typeof ABOUT_TAB

/** Tab bar entries — the editable files plus the non-file tabs. */
export const TABS: { key: TabKey; label: string }[] = [
  ...FILES.map((f) => ({ key: f.key, label: f.label })),
  { key: SETTINGS_TAB, label: SETTINGS_TAB },
  { key: ENV_TAB, label: ENV_TAB },
  { key: ABOUT_TAB, label: ABOUT_TAB },
]

/** Every valid tab key, derived from {@link TABS} so a new tab needs no second edit. */
const TAB_KEYS = new Set<string>(TABS.map((t) => t.key))

/** CodeMirror language extension per file `lang` id. */
export const LANG: Record<string, () => Extension> = {
  javascript,
  json,
  css,
}

export const FILE_BY_KEY: Record<FileKey, (typeof FILES)[number]> = Object.fromEntries(
  FILES.map((f) => [f.key, f]),
) as Record<FileKey, (typeof FILES)[number]>

/** Resolve the active tab from `#config/<key>`; falls back to the config JSON. */
export function tabFromHash(): TabKey {
  const route = getRoute()
  const raw = route.name === "config" ? route.tab : ""
  return TAB_KEYS.has(raw) ? (raw as TabKey) : "rcm.config.json"
}
