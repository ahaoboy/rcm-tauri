/**
 * SubmenuApp — renders a single level of submenu items in its own window.
 *
 * Windows are pre-created by Rust, labeled `submenu-0` … `submenu-3`.
 * Each window receives `menu-show` events from Rust with the full menu
 * data and an index path telling it which submenu to render.
 *
 * All hover/click coordination happens through Rust via events:
 *   menu-hover, menu-hover-out, menu-execute
 */

import { useEffect, useRef, useState, useCallback } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { getCurrentWindow, PhysicalPosition, currentMonitor } from "@tauri-apps/api/window";
import type { MenuShowPayload, MenuData, IndexPath } from "../types/menu";
import { ContextMenu } from "./ContextMenu";
import { useTheme } from "../hooks/useTheme";
import { feLog } from "../feLog";
import { EDGE_GAP, SUBMENU_GAP, winPadPhysical } from "../constants/layout";

const OFF_SCREEN = new PhysicalPosition(-9999, -9999);

export function SubmenuApp() {
  const [menu, setMenu] = useState<MenuData | null>(null);
  const [indexPath, setIndexPath] = useState<IndexPath>([]);
  const devMode = useRef(false);
  const menuActive = useRef(false);
  const theme = useTheme();
  /** Pending show info from menu-show event; consumed by handleReady. */
  const pendingShow = useRef<{
    x: number;
    y: number;
    parent_x?: number;
    parent_w?: number;
  } | null>(null);

  // Depth from URL hash: "submenu-0" → depth 1, "submenu-1" → depth 2, etc.
  const myLevel = parseInt(window.location.hash.replace("#submenu-", ""), 10) || 0;
  const depth = myLevel + 1; // submenu-0 = depth 1, submenu-1 = depth 2, …

  useEffect(() => {
    document.documentElement.classList.remove("rcm-light", "rcm-dark");
    document.documentElement.classList.add(`rcm-${theme}`);
  }, [theme]);

  useEffect(() => {
    const win = getCurrentWindow();
    let cleanupFns: (() => void)[] = [];

    // Start off-screen
    win.setPosition(OFF_SCREEN).catch(() => { });

    const setup = async () => {
      // ── Prevent close ──────────────────────────────────────────
      const unlistenClose = await win.onCloseRequested(async (e) => {
        e.preventDefault();
        await hideSubmenu(win);
      });
      cleanupFns.push(unlistenClose);

      // ── Rust → Frontend: menu-show for this depth ──────────────
      const unlistenShow = await listen<MenuShowPayload>("menu-show", (event) => {
        const { menu: menuData, path, x, y, parent_x, parent_w } = event.payload;

        const eventDepth = path.length === 0 ? 0 : path.length - 1;
        if (eventDepth !== depth) {
          feLog.info(`App:submenu-${myLevel}`, `menu-show SKIP (eventDepth=${eventDepth} != myDepth=${depth})`);
          return;
        }

        feLog.eventRecv("menu-show", `submenu-${myLevel} pos=(${x.toFixed(0)},${y.toFixed(0)}) path=[${path}]`);

        pendingShow.current = { x, y, parent_x, parent_w };
        setMenu(menuData);
        setIndexPath(path);
        // Window is off-screen; handleReady will position & show
      });
      cleanupFns.push(unlistenShow);

      // ── Rust → Frontend: hide all ──────────────────────────────
      const unlistenHide = await listen("menu-hide-all", () => {
        feLog.eventRecv("menu-hide-all", `submenu-${myLevel}`);
        if (devMode.current) {
          feLog.info(`App:submenu-${myLevel}`, "dev mode, ignoring hide");
          return;
        }
        hideSubmenu(win);
      });
      cleanupFns.push(unlistenHide);

      // ── Dev mode toggle ────────────────────────────────────────
      const unlistenDev = await listen<boolean>("dev-mode", (event) => {
        devMode.current = event.payload;
      });
      cleanupFns.push(unlistenDev);

      // ── Blur → Rust decides whether to hide all ─────────────────
      const unlistenFocus = await win.onFocusChanged(({ payload: focused }) => {
        feLog.info(`App:submenu-${myLevel}`, `onFocusChanged focused=${focused} menuActive=${menuActive.current}`);
        if (!focused && !devMode.current && menuActive.current) {
          feLog.eventSend("menu-blur", `depth=${depth}`);
          emit("menu-blur", { depth });
        }
      });
      cleanupFns.push(unlistenFocus);

      // Signal that this submenu window is ready
      // (no longer needed since Rust pre-creates windows)
    };

    setup();
    return () => {
      cleanupFns.forEach((fn) => fn());
    };
  }, [depth]);

  /** Called by ContextMenu after the window has been resized to fit content. */
  const handleReady = useCallback(async () => {
    const info = pendingShow.current;
    if (!info) return;
    pendingShow.current = null;

    try {
      const win = getCurrentWindow();
      const outerSize = await win.outerSize();
      const monitor = await currentMonitor();

      let finalX = info.x;
      let finalY = info.y;

      if (monitor) {
        const monRight = monitor.position.x + monitor.size.width;
        const monBottom = monitor.position.y + monitor.size.height;

        // If submenu overflows right edge, flip to the left of the parent
        if (finalX + outerSize.width > monRight - EDGE_GAP && info.parent_x != null) {
          // Align submenu right edge with parent content left edge, minus gap
          const pad = winPadPhysical();
          const flippedX = info.parent_x + pad - outerSize.width - SUBMENU_GAP;
          feLog.info(`App:submenu-${myLevel}`, `flip: right overflow → left_side=${flippedX.toFixed(0)} (pad=${pad.toFixed(0)})`);
          finalX = flippedX;
        }

        // Clamp to monitor bounds
        finalX = Math.max(
          monitor.position.x + EDGE_GAP,
          Math.min(finalX, monRight - outerSize.width - EDGE_GAP)
        );
        finalY = Math.max(
          monitor.position.y + EDGE_GAP,
          Math.min(finalY, monBottom - outerSize.height - EDGE_GAP)
        );

        feLog.info(`App:submenu-${myLevel}`, `clamp: size=(${outerSize.width}x${outerSize.height}) raw=(${info.x.toFixed(0)},${info.y.toFixed(0)}) -> (${finalX.toFixed(0)},${finalY.toFixed(0)})`);
      }

      await win.setPosition(new PhysicalPosition(Math.round(finalX), Math.round(finalY)));
      await win.setAlwaysOnTop(true);
      await win.show();
      await win.setFocus();

      setTimeout(() => {
        menuActive.current = true;
        feLog.info(`App:submenu-${myLevel}`, "menuActive armed");
      }, 200);
    } catch (e) {
      feLog.error(`App:submenu-${myLevel}`, `handleReady error: ${e}`);
    }
  }, [myLevel]);

  async function hideSubmenu(win: ReturnType<typeof getCurrentWindow>) {
    if (devMode.current) return;
    menuActive.current = false;
    await win.hide();
    await win.setPosition(OFF_SCREEN);
    setMenu(null);
    setIndexPath([]);
  }

  if (!menu) {
    return <div className="rcm-root" />;
  }

  return (
    <ContextMenu
      depth={depth}
      indexPath={indexPath}
      menu={menu}
      showIcons={false}
      onReady={handleReady}
    />
  );
}
