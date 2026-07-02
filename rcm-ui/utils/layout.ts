/**
 * Layout utilities — monitor selection and window position computation.
 *
 * The frontend measures the DOM (see ContextMenu.tsx), then uses these
 * functions to compute the final window position before showing it.
 * Rust is NOT involved in positioning — it only sends the ideal position.
 *
 * All values are in physical pixels.
 */

/** Gap between submenu and parent window edge (physical px). */
export const SUBMENU_GAP = 8

/** Minimum gap from screen/monitor edges (physical px). */
export const EDGE_GAP = 8

/** Position info from Rust's `menu-show` event, consumed after measurement. */
export interface PendingPos {
  /** Ideal .rcm-root left X on screen. */
  x: number
  /** Ideal .rcm-root top Y on screen. */
  y: number
  /** Parent's .rcm-root left X for submenu flip (undefined for root). */
  parentRootX?: number
}

/** Minimal monitor shape — compatible with Tauri's `Monitor` type. */
export interface MonitorLike {
  position: { x: number; y: number }
  size: { width: number; height: number }
}

/** Find the monitor containing the point, or the closest one. */
export function chooseMonitorForPoint<T extends MonitorLike>(
  monitors: T[],
  x: number,
  y: number,
): T | null {
  if (monitors.length === 0) return null

  for (const m of monitors) {
    const left = m.position.x
    const top = m.position.y
    const right = left + m.size.width
    const bottom = top + m.size.height
    if (x >= left && x <= right && y >= top && y <= bottom) {
      return m
    }
  }

  return monitors.reduce((best, m) => {
    const db = distToMonitor(best, x, y)
    const dm = distToMonitor(m, x, y)
    return dm < db ? m : best
  })
}

function distToMonitor(monitor: MonitorLike, x: number, y: number): number {
  const left = monitor.position.x
  const top = monitor.position.y
  const right = left + monitor.size.width
  const bottom = top + monitor.size.height
  const cx = Math.max(left, Math.min(x, right))
  const cy = Math.max(top, Math.min(y, bottom))
  return (x - cx) ** 2 + (y - cy) ** 2
}

export interface PositionInfo {
  idealX: number
  idealY: number
  parentRootX?: number
  winW: number
  winH: number
  rootOffsetX: number
  rootOffsetY: number
  rootW: number
}

/**
 * Compute the final window position (physical px) from the ideal .rcm-root
 * position and measured window geometry.
 *
 * Handles:
 *   - Right-edge overflow → flip (root flips to left of cursor,
 *     submenu flips to left of parent)
 *   - Monitor clamping for both X and Y
 */
export function computeWindowPosition(
  info: PositionInfo,
  monitors: MonitorLike[],
): { x: number; y: number } {
  const { idealX, idealY, parentRootX, winW, winH, rootOffsetX, rootOffsetY, rootW } = info

  const monitor = chooseMonitorForPoint(monitors, idealX, idealY)

  // Normal: window position = ideal root position - root offset within window
  let winX = idealX - rootOffsetX
  let winY = idealY - rootOffsetY

  // Flip on right-edge overflow
  if (monitor) {
    const monRight = monitor.position.x + monitor.size.width
    if (winX + winW > monRight - EDGE_GAP) {
      if (parentRootX !== undefined) {
        // Submenu: flip so root right edge = parent root left - gap
        winX = parentRootX - SUBMENU_GAP - rootW - rootOffsetX
      } else {
        // Root: flip so root right edge is at cursor
        winX = idealX - rootW - rootOffsetX
      }
    }
  }

  // Clamp to monitor bounds
  if (monitor) {
    const monLeft = monitor.position.x + EDGE_GAP
    const monTop = monitor.position.y + EDGE_GAP
    const monRight = monitor.position.x + monitor.size.width - EDGE_GAP
    const monBottom = monitor.position.y + monitor.size.height - EDGE_GAP

    winX = Math.max(monLeft, Math.min(winX, Math.max(monLeft, monRight - winW)))
    winY = Math.max(monTop, Math.min(winY, Math.max(monTop, monBottom - winH)))
  }

  return { x: winX, y: winY }
}
