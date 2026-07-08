/**
 * Menu event API — centralized Tauri event emit/listen functions.
 *
 * All communication with the Rust backend goes through this module.
 * Components and hooks should NEVER import `@tauri-apps/api/event`
 * directly. Instead, use the functions provided here.
 *
 * Event flow:
 *   Rust  → FE:  menu-show, menu-hide-all, dev-mode, icons-changed
 *   FE    → Rust: menu-hover, menu-hover-out, menu-execute, menu-blur,
 *                 menu-close-all, log-event
 */

import { invoke } from "@tauri-apps/api/core"
import { emit } from "@tauri-apps/api/event"
import type { UnlistenFn } from "@tauri-apps/api/event"

import type { MenuData, IndexPath, CommandPayload } from "../types/menu"

// ═══════════════════════════════════════════════════════════════════════
// Types — payloads sent from frontend to Rust
// ═══════════════════════════════════════════════════════════════════════

export interface MenuHoverData {
  depth: number
  path: IndexPath
  rootX: number
  rootY: number
  rootW: number
  rootH: number
  itemY: number
  itemH: number
}

export interface MenuShowEvent {
  menu: MenuData
  path: IndexPath
  x: number
  y: number
  parentRootX?: number
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

export function emitMenuHoverOut(depth: number): Promise<void> {
  return emit("menu-hover-out", { depth })
}

export function emitMenuExecute(path: IndexPath, command: CommandPayload): Promise<void> {
  return emit("menu-execute", { path, command })
}

export function emitMenuBlur(depth: number): Promise<void> {
  return emit("menu-blur", { depth })
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
