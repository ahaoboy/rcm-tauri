import { useEffect, useRef, useState } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import type { MenuData, MenuShowPayload } from "./types/menu";
import { useTheme } from "./hooks/useTheme";
import { ContextMenu } from "./components";
import { feLog } from "./feLog";

/** Off-screen position used to hide the window without flicker. */
const OFF_SCREEN = new PhysicalPosition(-9999, -9999);

/** Depth of this window. Root = 0. */
const MY_DEPTH = 0;

function App() {
  const [menu, setMenu] = useState<MenuData | null>(null);
  const [showIcons, setShowIcons] = useState(false);
  const devMode = useRef(false);
  const menuActive = useRef(false);
  const theme = useTheme();

  useEffect(() => {
    document.documentElement.classList.remove("rcm-light", "rcm-dark");
    document.documentElement.classList.add(`rcm-${theme}`);
  }, [theme]);

  useEffect(() => {
    let cleanupFns: (() => void)[] = [];
    const win = getCurrentWindow();

    // Start off-screen
    win.setPosition(OFF_SCREEN).catch(() => { });

    const setup = async () => {
      // ── Fetch initial config ──────────────────────────────────
      try {
        const cfg = await invoke<{ dev: boolean; icons: boolean }>("get_config");
        devMode.current = cfg.dev;
        setShowIcons(cfg.icons);
      } catch { /* ignore */ }

      // ── Prevent window close, just hide it ───────────────────
      const unlistenClose = await win.onCloseRequested(async (e) => {
        e.preventDefault();
        await hideRoot(win);
      });
      cleanupFns.push(unlistenClose);

      // ── Dev mode toggle ──────────────────────────────────────
      const unlistenDev = await listen<boolean>("dev-mode", (event) => {
        devMode.current = event.payload;
      });
      cleanupFns.push(unlistenDev);

      // ── Blur → Rust decides whether to hide all ──────────────
      const unlistenFocus = await win.onFocusChanged(({ payload: focused }) => {
        feLog.info("App:root", `onFocusChanged focused=${focused} menuActive=${menuActive.current}`);
        if (!focused && !devMode.current && menuActive.current) {
          feLog.eventSend("menu-blur", "depth=0");
          emit("menu-blur", { depth: MY_DEPTH });
        }
      });
      cleanupFns.push(unlistenFocus);

      // ── Icons toggle ─────────────────────────────────────────
      const unlistenIcons = await listen<boolean>("icons-changed", (event) => {
        setShowIcons(event.payload);
      });
      cleanupFns.push(unlistenIcons);

      // ── Rust → Frontend: show menu at root level ─────────────
      const unlistenShow = await listen<MenuShowPayload>("menu-show", (event) => {
        const { menu: menuData, path, x, y } = event.payload;
        // Only process root-level events (empty path)
        if (path.length !== 0) return;

        feLog.eventRecv("menu-show", `pos=(${x.toFixed(0)},${y.toFixed(0)}) groups=${menuData.groups.length}`);

        setMenu(menuData);

        win.setPosition(new PhysicalPosition(x, y)).catch(() => { });
        win.show().catch(() => { });
        win.setFocus().catch(() => { });

        // Delay arming menuActive to let focus settle after setFocus()
        // (prevents brief focus-lose-then-regain from triggering blur)
        setTimeout(() => {
          menuActive.current = true;
          feLog.info("App:root", "menuActive armed");
        }, 200);
      });
      cleanupFns.push(unlistenShow);

      // ── Rust → Frontend: hide all ────────────────────────────
      const unlistenHide = await listen("menu-hide-all", () => {
        feLog.eventRecv("menu-hide-all", "");
        if (devMode.current) {
          feLog.info("App:root", "dev mode, ignoring hide");
          return;
        }
        hideRoot(win);
      });
      cleanupFns.push(unlistenHide);
    };

    setup();

    return () => {
      cleanupFns.forEach((fn) => fn());
    };
  }, []);

  /** Hide the root window and reset state. */
  async function hideRoot(win: ReturnType<typeof getCurrentWindow>) {
    if (devMode.current) return;
    menuActive.current = false;
    await win.hide();
    await win.setPosition(OFF_SCREEN);
    setMenu(null);
  }

  if (!menu) {
    return <div className="rcm-root" />;
  }

  return (
    <ContextMenu
      key={showIcons ? "icons" : "no-icons"}
      depth={MY_DEPTH}
      indexPath={[]}
      menu={menu}
      showIcons={showIcons}
    />
  );
}

export default App;

