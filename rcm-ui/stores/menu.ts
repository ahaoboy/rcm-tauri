/**
 * Menu store — the level a menu window is currently rendering.
 *
 * One store instance per window (a webview is its own JS realm), so root and
 * submenu windows never share state. `useMenuWindow` writes here from the
 * `menu-show` / `menu-hide-all` events; `ContextMenu` reads it.
 */

import { create } from "zustand"

import type { IndexPath, MenuData } from "../types/menu"

interface MenuState {
  /** The level Rust asked this window to render, or `null` when hidden. */
  menu: MenuData | null
  /** Path of the rendered level within the menu tree. */
  indexPath: IndexPath
  /** Render a level (called on `menu-show`). */
  setLevel: (menu: MenuData, indexPath: IndexPath) => void
  /** Clear the level (called when the window is hidden). */
  clear: () => void
}

export const useMenuStore = create<MenuState>()((set) => ({
  menu: null,
  indexPath: [],
  setLevel: (menu, indexPath) => set({ menu, indexPath }),
  clear: () => set({ menu: null, indexPath: [] }),
}))
