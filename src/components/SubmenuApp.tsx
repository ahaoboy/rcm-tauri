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

import { useEffect, useRef, useState } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import type { MenuShowPayload, MenuData, IndexPath } from "../types/menu";
import { ContextMenu } from "./ContextMenu";
import { useTheme } from "../hooks/useTheme";
import { feLog } from "../feLog";

const OFF_SCREEN = new PhysicalPosition(-9999, -9999);

export function SubmenuApp() {
  const [menu, setMenu] = useState<MenuData | null>(null);
  const [indexPath, setIndexPath] = useState<IndexPath>([]);
  const devMode = useRef(false);
  const menuActive = useRef(false);
  const theme = useTheme();

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
        const { menu: menuData, path, x, y } = event.payload;

        const eventDepth = path.length === 0 ? 0 : path.length - 1;
        if (eventDepth !== depth) {
          feLog.info(`App:submenu-${myLevel}`, `menu-show SKIP (eventDepth=${eventDepth} != myDepth=${depth})`);
          return;
        }

        feLog.eventRecv("menu-show", `submenu-${myLevel} pos=(${x.toFixed(0)},${y.toFixed(0)}) path=[${path}]`);

        setMenu(menuData);
        setIndexPath(path);

        win.setPosition(new PhysicalPosition(x, y)).catch(() => { });
        win.show().catch(() => { });
        win.setFocus().catch(() => { });

        setTimeout(() => {
          menuActive.current = true;
          feLog.info(`App:submenu-${myLevel}`, "menuActive armed");
        }, 200);
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
    />
  );
}
