import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, cursorPosition, LogicalSize } from "@tauri-apps/api/window";
import type { MenuData, InputEventPayload } from "./types/menu";
import { useTheme } from "./hooks/useTheme";
import { ContextMenu } from "./components";

function App() {
  const [menu, setMenu] = useState<MenuData | null>(null);
  const theme = useTheme();

  useEffect(() => {
    // Apply theme class to <html> for programmatic override of OS theme
    document.documentElement.classList.remove("rcm-light", "rcm-dark");
    document.documentElement.classList.add(`rcm-${theme}`);
  }, [theme]);

  useEffect(() => {
    let unlistenFn: (() => void) | undefined;

    const setupListener = async () => {
      const win = getCurrentWindow();

      // Set initial window size to accommodate menu + shadow
      await win.setSize(new LogicalSize(300, 450));

      const unlisten = await listen<InputEventPayload>("input-event", async (event) => {
        console.log('event', event)
        if (event.payload.event === "Menu" || event.payload.event === "Shift") {
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
            await win.hide();
            setMenu(null);
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
    };
  }, []);

  // No menu data yet — render empty (window is hidden anyway)
  if (!menu) {
    return <div className="rcm-root" />;
  }

  return (
    <ContextMenu
      iconItems={menu.iconItems}
      groups={menu.groups}
    />
  );
}

export default App;

