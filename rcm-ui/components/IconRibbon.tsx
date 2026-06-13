import { emit } from "@tauri-apps/api/event"
import React from "react"

import type { MenuItem } from "../types/menu"

/* ── Props ──────────────────────────────────────────────────────────── */

interface IconRibbonProps {
  items: MenuItem[]
  /** Base path value for icon items (use -1 to differentiate from groups). */
  iconBasePath: number
}

/* ═══════════════════════════════════════════════════════════════════════
   IconRibbon — top icon bar in Win11-style context menus
   ═══════════════════════════════════════════════════════════════════════ */

export const IconRibbon: React.FC<IconRibbonProps> = ({ items, iconBasePath }) => {
  if (!items || items.length === 0) return null

  return (
    <div className="rcm-ribbon" role="toolbar" aria-label="Quick actions">
      {items.map((item, idx) => (
        <button
          key={item.key || `ribbon-${idx}`}
          className="rcm-ribbon-btn"
          aria-disabled={item.disable}
          title={item.label || item.key}
          tabIndex={item.disable ? -1 : 0}
          onClick={async (e) => {
            e.stopPropagation()
            if (item.disable) return

            if (item.command) {
              await emit("menu-execute", {
                path: [iconBasePath, idx],
                command: item.command,
              })
            }
          }}
        >
          {item.icon || item.key}
        </button>
      ))}
    </div>
  )
}
