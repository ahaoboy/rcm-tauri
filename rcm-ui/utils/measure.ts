/**
 * Measurement utilities.
 *
 * The frontend is responsible for exactly one piece of geometry: how large it
 * drew a menu level. Every position is computed in Rust (`rcm_core::ui`), so the
 * only job here is to turn a rendered DOM subtree into physical-pixel numbers.
 *
 * All values are **physical pixels** (CSS px × devicePixelRatio), matching the
 * unit Rust's placement algorithm works in.
 */

/** Physical pixels per CSS pixel for the current window. */
export function dpi(): number {
  return window.devicePixelRatio || 1
}

/** A rendered menu level, measured in physical pixels. */
export interface Measurement {
  /** Full window size, including the container's CSS padding. */
  winW: number
  winH: number
  /** Size of `.rcm-root` itself. */
  rootW: number
  rootH: number
  /** Offset of `.rcm-root` inside the window (the container's padding). */
  rootOffsetX: number
  rootOffsetY: number
}

type Edge = "Left" | "Top" | "Right" | "Bottom"

/** Read one edge of an element's computed padding, in CSS px. */
function padding(style: CSSStyleDeclaration, edge: Edge): number {
  const value = style.getPropertyValue(`padding-${edge.toLowerCase()}`)
  return parseFloat(value) || 0
}

/**
 * Measure the `.rcm-root` element and its padded container.
 *
 * The padding is read from computed style rather than hardcoded, so a user can
 * change `--rcm-window-pad` in CSS and the window will still be sized to fit.
 *
 * Uses `offsetWidth` / `offsetHeight` — **not** `getBoundingClientRect`, which
 * is affected by CSS transforms (the `rcm-fade-in` animation's `scale(0.94)`
 * would shrink the reported size).
 */
export function measureLevel(root: HTMLElement): Measurement {
  const container = root.parentElement ?? root
  const style = getComputedStyle(container)
  const scale = dpi()

  const padLeft = padding(style, "Left")
  const padTop = padding(style, "Top")
  const padRight = padding(style, "Right")
  const padBottom = padding(style, "Bottom")

  const rootW = root.offsetWidth
  const rootH = root.offsetHeight

  return {
    winW: Math.round((rootW + padLeft + padRight) * scale),
    winH: Math.round((rootH + padTop + padBottom) * scale),
    rootW: Math.round(rootW * scale),
    rootH: Math.round(rootH * scale),
    rootOffsetX: Math.round(padLeft * scale),
    rootOffsetY: Math.round(padTop * scale),
  }
}

/**
 * Offset of `element` from the top of `.rcm-root`, in physical pixels.
 *
 * Reported on hover so Rust can align a submenu with the hovered row even if the
 * CSS row height differs from the metrics Rust assumes.
 */
export function offsetWithinRoot(element: HTMLElement, root: HTMLElement): number {
  const elementRect = element.getBoundingClientRect()
  const rootRect = root.getBoundingClientRect()
  return Math.round((elementRect.top - rootRect.top) * dpi())
}
