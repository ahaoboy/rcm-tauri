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
export const WINDOW_PADDING = 16

/** Per-side padding in CSS pixels (half of WINDOW_PADDING). */
export const WINDOW_PAD_PER_SIDE = WINDOW_PADDING / 2 // 8

/** Gap between submenu and parent window edges (physical px).  */
export const SUBMENU_GAP = 8

/** Minimum gap from screen/monitor edges (physical px). */
export const EDGE_GAP = 8

export interface MonitorLike {
  position: {
    x: number
    y: number
  }
  size: {
    width: number
    height: number
  }
}

export interface WindowSizeLike {
  width: number
  height: number
}

/** Compute per-side window padding in physical pixels at current DPI. */
export function winPadPhysical(): number {
  return WINDOW_PAD_PER_SIDE * (window.devicePixelRatio || 1)
}

export function chooseMonitorForPoint<T extends MonitorLike>(
  monitors: T[],
  x: number,
  y: number,
): T | null {
  if (monitors.length === 0) return null

  const containing = monitors.find((monitor) => {
    const left = monitor.position.x
    const top = monitor.position.y
    const right = left + monitor.size.width
    const bottom = top + monitor.size.height
    return x >= left && x <= right && y >= top && y <= bottom
  })
  if (containing) return containing

  return monitors.reduce((best, monitor) => {
    const bestDistance = distanceToMonitor(best, x, y)
    const nextDistance = distanceToMonitor(monitor, x, y)
    return nextDistance < bestDistance ? monitor : best
  })
}

export function clampWindowToMonitor(
  x: number,
  y: number,
  size: WindowSizeLike,
  monitor: MonitorLike,
): { x: number; y: number } {
  const left = monitor.position.x + EDGE_GAP
  const top = monitor.position.y + EDGE_GAP
  const right = monitor.position.x + monitor.size.width - size.width - EDGE_GAP
  const bottom = monitor.position.y + monitor.size.height - size.height - EDGE_GAP

  return {
    x: clamp(x, left, Math.max(left, right)),
    y: clamp(y, top, Math.max(top, bottom)),
  }
}

function distanceToMonitor(monitor: MonitorLike, x: number, y: number): number {
  const left = monitor.position.x
  const top = monitor.position.y
  const right = left + monitor.size.width
  const bottom = top + monitor.size.height

  const closestX = clamp(x, left, right)
  const closestY = clamp(y, top, bottom)
  const dx = x - closestX
  const dy = y - closestY
  return dx * dx + dy * dy
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(value, max))
}
