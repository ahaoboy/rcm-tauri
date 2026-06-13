/**
 * Frontend type definitions for the RCM menu system.
 * Mirrors the Rust `rcm::Item` and `rcm::CommandPayload` structs.
 */

/** Serializable command payload sent to the Tauri `execute` command. */
export interface CommandPayload {
  exe: string
  args?: string[]
  cwd?: string
  admin?: boolean
  window?: string
}

/** A single item in the right-click menu (as received from the backend). */
export interface MenuItem {
  key: string
  icon: string
  label: string
  disable: boolean
  admin: boolean
  window: string
  items: MenuItem[]
  command: CommandPayload | null
}

/** The full menu structure emitted by the backend. */
export interface MenuData {
  iconItems: MenuItem[]
  groups: MenuItem[]
}

/** Button type from input events. */
export type ButtonType = "Left" | "Right"

/** ── New event system types ─────────────────────────────────────────── */

/**
 * Index path navigating the menu tree.
 * - Empty `[]` = root (shows iconItems + groups)
 * - `[-1, i, ...]` = icon ribbon item i, then deeper into its children
 * - `[g, i, ...]` = groups[g].items[i], then deeper into its children
 */
export type IndexPath = number[]

/**
 * Rust → Frontend: show menu at a specific depth window.
 */
export interface MenuShowPayload {
  /** Full menu data — every window has the complete tree. */
  menu: MenuData
  /** Index path to the submenu to render.
   *  `[]` for root, e.g. `[0, 2]` for groups[0].items[2].items */
  path: IndexPath
  /** Ideal screen position (frontend will clamp after measuring DOM). */
  x: number
  y: number
  /** Parent window info for submenu flip logic (undefined for root). */
  parent_x?: number
  parent_y?: number
  parent_w?: number
}

/**
 * Frontend → Rust: user hovered over a menu item.
 */
export interface MenuHoverPayload {
  /** Depth level of the emitting window (0 = root). */
  depth: number
  /** Index path to the hovered item. */
  path: IndexPath
  /** Parent window's absolute screen position. */
  parentX: number
  parentY: number
  /** Parent window's size. */
  parentW: number
  parentH: number
  /** Hovered item's position relative to the parent window. */
  itemX: number
  itemY: number
  /** Hovered item's size. */
  itemW: number
  itemH: number
}

/**
 * Frontend → Rust: user clicked a leaf item (no children) → execute.
 */
export interface MenuExecutePayload {
  /** Index path to the clicked item. */
  path: IndexPath
  /** Command to execute. */
  command: CommandPayload
}

/**
 * Rust → Frontend: hide all menu windows.
 */
export interface MenuHideAllPayload {
  /** If true, force hide even in dev mode. */
  force?: boolean
}

/** Payload received via the `input-event` Tauri event (kept for compatibility). */
export interface InputEventPayload {
  event: string
  button: string
  menu: MenuData
}
