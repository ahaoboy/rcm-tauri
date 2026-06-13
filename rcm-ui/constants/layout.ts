/**
 * Shared layout constants for menu window positioning.
 *
 * These MUST match the corresponding values in Rust (src-tauri/src/lib.rs).
 *   SUBMENU_GAP → Rust: const SUBMENU_GAP: f64 = 8.0;
 *
 * resizeWindow() in ContextMenu.tsx adds WINDOW_PADDING CSS px to the
 * measured .rcm-root rect. This extra space is centered, giving
 * WINDOW_PAD_PER_SIDE px on each edge.
 */

/** Extra CSS pixels added to content size in resizeWindow (ContextMenu.tsx). */
export const WINDOW_PADDING = 16;

/** Per-side padding in CSS pixels (half of WINDOW_PADDING). */
export const WINDOW_PAD_PER_SIDE = WINDOW_PADDING / 2; // 8

/** Gap between submenu and parent window edges (physical px).  */
export const SUBMENU_GAP = 8;

/** Minimum gap from screen/monitor edges (physical px). */
export const EDGE_GAP = 8;

/** Compute per-side window padding in physical pixels at current DPI. */
export function winPadPhysical(): number {
  return WINDOW_PAD_PER_SIDE * (window.devicePixelRatio || 1);
}
