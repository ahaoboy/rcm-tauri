/**
 * Submenu utilities.
 *
 * In the refactored architecture, Rust manages all window positioning
 * and submenu lifecycle. This module retains only leftover helpers
 * that may still be needed. New code should use the event-based system:
 *
 *   Frontend → emit("menu-hover", ...)     → Rust shows submenu
 *   Frontend → emit("menu-execute", ...)   → Rust executes + hides all
 *   Rust     → emit("menu-show", ...)      → Frontend renders
 *   Rust     → emit("menu-hide-all", ...)  → Frontend hides
 */

import type { MenuItem } from "./types/menu"

/** Legacy payload type kept for compatibility. */
export interface SubmenuPayload {
  items: MenuItem[]
  level: number
  x: number
  y: number
}
