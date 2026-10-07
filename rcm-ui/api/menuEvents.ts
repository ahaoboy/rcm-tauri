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
// Shell extension diagnostics
// ═══════════════════════════════════════════════════════════════════════

/** Outcome of a Retry attempt against the shell extension. */
export interface RetryResult {
  /** `true` when the pipe connected — the right-click menu is live. */
  ok: boolean
  /** Fresh diagnostic report when `ok` is `false`; empty on success. */
  report: string
}

/**
 * Re-probe the shell extension after registering it and restarting Explorer.
 * Resolves with the new state so the caller can update its report.
 */
export function retryShellExtension(): Promise<RetryResult> {
  return invoke<RetryResult>("retry_shell_extension")
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

/** Theme preference changed ("system" | "light" | "dark"). */
export function onThemeChanged(handler: (theme: string) => void): Promise<UnlistenFn> {
  return listen<string>("theme-changed", (e) => handler(e.payload))
}

/**
 * Whether the shell extension has shown any sign of life.
 *
 * `true` once it has delivered an event over the pipe it connects to, which is
 * the only proof it is loaded. Reads a global on the Rust side — it costs no
 * pipe traffic — so the diagnostic window can poll it every second.
 */
export function shellExtensionConnected(): Promise<boolean> {
  return invoke<boolean>("shell_extension_connected")
}

// ═══════════════════════════════════════════════════════════════════════
// About
// ═══════════════════════════════════════════════════════════════════════

/**
 * The runtime facts the About page displays.
 *
 * Only what the webview cannot read for itself. The page's own content — name,
 * description, wording and layout — is defined in the frontend.
 */
export interface RuntimePaths {
  version: string
  /** Short git commit, `-dirty` if the tree was modified. Empty outside a git checkout. */
  commit: string
  exe_dir: string
  config_path: string
  log_path: string
}

export function getRuntimePaths(): Promise<RuntimePaths> {
  return invoke<RuntimePaths>("get_runtime_paths")
}

/** Open the folder holding the executable and its config files. */
export function openConfigFolder(): Promise<void> {
  return invoke("open_config_folder")
}

// ═══════════════════════════════════════════════════════════════════════
// Settings (the tray's own options, editable from the settings tab)
// ═══════════════════════════════════════════════════════════════════════

/** Context-menu style. */
export type Style = "win11" | "classic"

/** Menu theme preference. */
export type ThemeName = "system" | "light" | "dark"

/**
 * Every user-facing setting, gathered from wherever it is stored.
 *
 * There is no single settings file: style and autostart live in the registry,
 * registration in the shell extension, and the rest in `rcm.config.json`. Rust
 * reads them all so the UI does not need to know.
 *
 * The remote-sync URLs are not here — the config editor already edits them as
 * JSON, and a second form for the same values would just be a way to disagree.
 */
export interface Settings {
  style: Style
  theme: ThemeName
  icons: boolean
  dev: boolean
  blocking: boolean
  registered: boolean
  /** Path Explorer will load the extension from. */
  extension_dll: string
  autostart: boolean
}

/**
 * Settings to change. Omitted fields are left alone, so a caller can change one
 * setting without echoing back values it never touched.
 */
export type SettingsPatch = Partial<Settings>

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings")
}

/**
 * Apply a change and resolve with the resulting state.
 *
 * The reply is the full state, not the requested values: registration, blocking
 * and autostart can be refused by the OS, so callers should render what comes
 * back rather than what they sent.
 */
export function updateSettings(patch: SettingsPatch): Promise<Settings> {
  return invoke<Settings>("update_settings", { patch })
}

/** Restart Explorer so registration changes take effect. */
export function applyChanges(): Promise<void> {
  return invoke("apply_changes")
}

/** Restore every config file to the embedded defaults. */
export function resetSettings(): Promise<Settings> {
  return invoke<Settings>("reset_settings")
}
