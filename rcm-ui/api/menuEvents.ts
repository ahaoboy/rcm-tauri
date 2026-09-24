/**
 * Menu event API — centralized Tauri event emit/listen functions.
 *
 * All communication with the Rust backend goes through this module.
 * Components and hooks should NEVER import `@tauri-apps/api/event`
 * directly. Instead, use the functions provided here.
 *
 * Event flow:
 *   Rust  → FE:  menu-show (what to render), menu-hide-all, dev-mode,
 *                icons-changed, theme-changed, style-changed
 *   FE    → Rust: menu-hover (which row), menu-measured (how big it drew),
 *                 menu-execute, menu-close-all, log-event
 *
 * Rust never sends a position and the frontend never computes one. The only
 * geometry crossing the boundary is a measurement.
 *
 * Dismissal is *not* an event: Rust polls which window holds focus, because a
 * blur cannot distinguish "focus moved to another menu window" from "focus left
 * the menu".
 */

import { invoke } from "@tauri-apps/api/core"
import { emit } from "@tauri-apps/api/event"
import type { UnlistenFn } from "@tauri-apps/api/event"

import type { MenuData, IndexPath, CommandPayload } from "../types/menu"
import type { Measurement } from "../utils/measure"

// ═══════════════════════════════════════════════════════════════════════
// Types — payloads sent from frontend to Rust
// ═══════════════════════════════════════════════════════════════════════

/**
 * The pointer entered a menu row.
 *
 * Only identifies the row: Rust already knows the level's rectangle and its own
 * row metrics, so no window geometry is needed.
 */
export interface MenuHoverData {
  depth: number
  path: IndexPath
  /** Row index within the level. */
  index: number
  /** Measured offset of the row from the content top, in physical px. */
  itemY?: number
}

/** How large the frontend drew a level, in physical pixels. */
export type MenuMeasuredData = Measurement & { depth: number }

/** Render this level. Carries no geometry. */
export interface MenuShowEvent {
  menu: MenuData
  path: IndexPath
}

export interface AppConfig {
  dev: boolean
  icons: boolean
  theme: string
  js_url: string | null
  css_url: string | null
  config_url: string | null
}

// ═══════════════════════════════════════════════════════════════════════
// Emit — Frontend → Rust
// ═══════════════════════════════════════════════════════════════════════

export function emitMenuHover(data: MenuHoverData): Promise<void> {
  return emit("menu-hover", data)
}

/**
 * Report the size of a rendered level.
 *
 * This is what drives placement: Rust clamps/flips using these numbers and then
 * resizes, moves and reveals the window.
 */
export function emitMenuMeasured(data: MenuMeasuredData): Promise<void> {
  return emit("menu-measured", data)
}

export function emitMenuExecute(path: IndexPath, command: CommandPayload): Promise<void> {
  return emit("menu-execute", { path, command })
}

export function emitMenuCloseAll(): Promise<void> {
  return emit("menu-close-all")
}

export function emitLog(tag: string, msg: string): Promise<void> {
  return emit("log-event", { tag, msg }).catch(() => {})
}

// ═══════════════════════════════════════════════════════════════════════
// Invoke — Tauri commands
// ═══════════════════════════════════════════════════════════════════════

export function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config")
}

export function getStyleCss(): Promise<string> {
  return invoke<string>("get_style_css")
}

// ═══════════════════════════════════════════════════════════════════════
// Config editor — file read/write/open
// ═══════════════════════════════════════════════════════════════════════

export function readConfigFile(name: string): Promise<string> {
  return invoke<string>("read_config_file", { name })
}

export function saveConfigFile(name: string, content: string): Promise<void> {
  return invoke("save_config_file", { name, content })
}

export function openInEditor(name: string): Promise<void> {
  return invoke("open_in_editor", { name })
}

export function notifyStyleUpdated(css: string): Promise<void> {
  return invoke("notify_style_updated", { css })
}

// ═══════════════════════════════════════════════════════════════════════
// Environment variables
// ═══════════════════════════════════════════════════════════════════════

/** A single environment variable visible to the RCM process. */
export interface EnvVar {
  key: string
  value: string
}

/** Read every environment variable of the current process. */
export function getEnvVars(): Promise<EnvVar[]> {
  return invoke<EnvVar[]>("get_env_vars")
}

// ═══════════════════════════════════════════════════════════════════════
// Pull — download latest files from configured remote URLs
// ═══════════════════════════════════════════════════════════════════════

export function pullJs(): Promise<string> {
  return invoke<string>("pull_js")
}

export function pullCss(): Promise<string> {
  return invoke<string>("pull_css")
}

export function pullConfig(): Promise<string> {
  return invoke<string>("pull_config")
}

export function showError(message: string): Promise<void> {
  return invoke<void>("show_error", { message })
}

// ═══════════════════════════════════════════════════════════════════════
// Listen — Rust → Frontend
// ═══════════════════════════════════════════════════════════════════════

import { listen } from "@tauri-apps/api/event"

export function onMenuShow(handler: (payload: MenuShowEvent) => void): Promise<UnlistenFn> {
  return listen<MenuShowEvent>("menu-show", (e) => handler(e.payload))
}

export function onMenuHideAll(handler: () => void): Promise<UnlistenFn> {
  return listen("menu-hide-all", () => handler())
}

export function onDevMode(handler: (dev: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>("dev-mode", (e) => handler(e.payload))
}

export function onIconsChanged(handler: (icons: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>("icons-changed", (e) => handler(e.payload))
}
