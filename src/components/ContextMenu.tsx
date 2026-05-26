import React, { useCallback, useEffect, useRef, useState } from "react";
import type { MenuItem } from "../types/menu";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

/* ── Sub-components ─────────────────────────────────────────────────── */

import { IconRibbon } from "./IconRibbon";
import { MenuGroup } from "./MenuGroup";
import { MenuSeparator } from "./MenuSeparator";

/* ── Props ──────────────────────────────────────────────────────────── */

interface ContextMenuProps {
  iconItems: MenuItem[];
  groups: MenuItem[];
}

/* ── Helpers ────────────────────────────────────────────────────────── */

/**
 * Execute a command payload via the Tauri `execute` backend command,
 * then hide the window.
 */
async function handleItemClick(item: MenuItem): Promise<void> {
  if (item.disable) return;

  if (item.command) {
    try {
      console.log('handleItemClick', item)
      await invoke("execute", { cmd: item.command });
    } catch (err) {
      console.error("Command execution failed:", err);
    }
  }

  // Hide window after action
  const win = getCurrentWindow();
  await win.hide();
}

/* ═══════════════════════════════════════════════════════════════════════
   ContextMenu — root menu component
   ═══════════════════════════════════════════════════════════════════════ */

export const ContextMenu: React.FC<ContextMenuProps> = ({
  iconItems,
  groups,
}) => {
  const rootRef = useRef<HTMLDivElement>(null);
  const [menuSize, setMenuSize] = useState({ width: 280, height: 400 });

  // Measure the rendered menu and resize the Tauri window to fit
  const resizeWindow = useCallback(async () => {
    if (!rootRef.current) return;
    const rect = rootRef.current.getBoundingClientRect();
    // Add padding for the window shadow
    const w = Math.ceil(rect.width) + 16;
    const h = Math.ceil(rect.height) + 16;
    if (w === menuSize.width && h === menuSize.height) return;
    setMenuSize({ width: w, height: h });
    try {
      await getCurrentWindow().setSize(new LogicalSize(w, h));
    } catch {
      // Window may not be available yet
    }
  }, [menuSize]);

  useEffect(() => {
    // Resize after initial render
    const raf = requestAnimationFrame(() => {
      resizeWindow();
    });
    return () => cancelAnimationFrame(raf);
  }, [iconItems, groups, resizeWindow]);

  const hasIconItems = iconItems && iconItems.length > 0;

  // Filter out groups that have no visible items
  const visibleGroups = groups.filter(
    (g) => g.items && g.items.length > 0,
  );

  return (
    <div className="rcm-root" ref={rootRef} role="menu">
      {hasIconItems && (
        <IconRibbon items={iconItems} onItemClick={handleItemClick} />
      )}

      {visibleGroups.map((group, gi) => (
        <React.Fragment key={gi}>
          {gi > 0 && <MenuSeparator />}
          <MenuGroup group={group} onItemClick={handleItemClick} />
        </React.Fragment>
      ))}
    </div>
  );
};
