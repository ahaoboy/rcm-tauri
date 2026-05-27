import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, cursorPosition, LogicalSize, PhysicalPosition } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import type { MenuData, InputEventPayload } from "./types/menu";
import { useTheme } from "./hooks/useTheme";
import { ContextMenu } from "./components";

/** Off-screen position used to hide the window without flicker. */
const OFF_SCREEN = new PhysicalPosition(-9999, -9999);

function App() {
  const [menu, setMenu] = useState<MenuData | null>(null);
  const [showIcons, setShowIcons] = useState(false);
  const devMode = useRef(false);
  const theme = useTheme();

  useEffect(() => {
    // Apply theme class to <html> for programmatic override of OS theme
    document.documentElement.classList.remove("rcm-light", "rcm-dark");
    document.documentElement.classList.add(`rcm-${theme}`);
  }, [theme]);

  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    let unlistenFocus: (() => void) | undefined;
    let unlistenDev: (() => void) | undefined;
    let unlistenIcons: (() => void) | undefined;

    const setupListener = async () => {
      const win = getCurrentWindow();

      // Start off-screen
      await win.setPosition(OFF_SCREEN);
      // Set initial window size to accommodate menu + shadow
      await win.setSize(new LogicalSize(300, 450));

      // ── Fetch initial config from backend ─────────────────────
      try {
        const cfg = await invoke<{ dev: boolean; icons: boolean }>("get_config");
        devMode.current = cfg.dev;
        setShowIcons(cfg.icons);
      } catch { /* ignore */ }

      // ── Hide menu when window loses focus (unless dev mode) ────
      unlistenFocus = await win.onFocusChanged(({ payload: focused }) => {
        if (!focused && !devMode.current) {
          hideMenu(win);
        }
      });

      // ── Listen for dev-mode toggle from tray ──────────────────
      unlistenDev = await listen<boolean>("dev-mode", (event) => {
        devMode.current = event.payload;
      });

      // ── Listen for icons toggle from tray ────────────────────
      unlistenIcons = await listen<boolean>("icons-changed", (event) => {
        console.log('icons-changed', event.payload)
        setShowIcons(event.payload);
      });

      // ── Listen for right-click events from the backend ───────────
      const unlisten = await listen<InputEventPayload>("input-event", async (event) => {
        console.log('event', event)
        if (event.payload.event === "Menu") {
          // Update menu data FIRST so React renders before window is shown
          if (event.payload.menu) {
            setMenu(event.payload.menu);
          }

          // Position window at cursor
          const pos = await cursorPosition();

          await win.setPosition(pos);
          await win.show();
          await win.setFocus();
        } else if (event.payload.event === "Click") {
          // Delay hide to allow click processing
          setTimeout(async () => {
            hideMenu(win);
          }, 150);
        }
      });
      return unlisten;
    };

    setupListener().then((fn) => {
      unlistenFn = fn;
    });

    return () => {
      if (unlistenFn) unlistenFn();
      if (unlistenFocus) unlistenFocus();
      if (unlistenDev) unlistenDev();
      if (unlistenIcons) unlistenIcons();
    };
  }, []);

  /** Hide the menu and move off-screen (unless dev mode). */
  async function hideMenu(win: ReturnType<typeof getCurrentWindow>) {
    await win.hide();
    setMenu(null);
    if (!devMode.current) {
      await win.setPosition(OFF_SCREEN);
    }
  }

  // No menu data yet — render empty (window is hidden anyway)
  if (!menu) {
    return <div className="rcm-root" />;
  }

  return (
    <ContextMenu
      key={showIcons ? 'icons' : 'no-icons'}
      iconItems={menu.iconItems}
      groups={menu.groups}
      showIcons={showIcons}
    />
  );
}

export default App;

