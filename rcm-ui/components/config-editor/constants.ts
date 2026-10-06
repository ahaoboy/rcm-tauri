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

export type TabKey = FileKey | typeof ENV_TAB | typeof SETTINGS_TAB

/** Tab bar entries — the editable files plus the non-file tabs. */
export const TABS: { key: TabKey; label: string }[] = [
  ...FILES.map((f) => ({ key: f.key as TabKey, label: f.label })),
  { key: SETTINGS_TAB, label: SETTINGS_TAB },
  { key: ENV_TAB, label: ENV_TAB },
]

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
  const raw = route.name === "config" ? route.tab : "rcm.config.json"
  if (raw === ENV_TAB) return ENV_TAB
  if (raw === SETTINGS_TAB) return SETTINGS_TAB
  return FILE_BY_KEY[raw as FileKey] ? (raw as FileKey) : "rcm.config.json"
}
