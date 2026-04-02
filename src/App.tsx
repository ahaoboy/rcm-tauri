import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, cursorPosition, LogicalSize, PhysicalPosition } from "@tauri-apps/api/window";
import "./App.css";
import type { Menu } from "rcm"

export type ButtonType = "Left" | "Right";
export type InputEvent = {
  button: ButtonType
  menu: Menu
  x?: number
  y?: number
};

function App() {
  const [menuVisible, setMenuVisible] = useState(false);

  useEffect(() => {
    let unlistenFn: (() => void) | undefined;

    const setupListener = async () => {
      const win = getCurrentWindow();

      // Resize window to dynamically fit the context menu and its ambient shadow.
      await win.setSize(new LogicalSize(280, 400));

      const unlisten = await listen<InputEvent>("input-event", async (event) => {
        console.log('event', event)
        if (event.payload.button === "Right") {
          let pos;
          if (event.payload.x != null && event.payload.y != null) {
            pos = new PhysicalPosition(event.payload.x, event.payload.y);
          } else {
            pos = await cursorPosition();
          }

          await win.setPosition(pos);
          await win.show();
          await win.setFocus();
          setMenuVisible(true);
        } else if (event.payload.button === "Left") {
          // Add a short delay to allow React to process UI clicks before hiding the entire webview
          setTimeout(async () => {
            await win.hide();
            setMenuVisible(false);
          }, 150);
        }
      });
      return unlisten;
    };

    setupListener().then(fn => { unlistenFn = fn; });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  const handleAction = async (actionName: string) => {
    console.log(`Action triggered: ${actionName}`);
    // Replace this logic with actual implementation, e.g. invoke("create_folder") for New.
    // We hide the window after executing the action.
    const win = getCurrentWindow();
    await win.hide();
    setMenuVisible(false);
  };

  return (
    <div className="context-menu" style={{ display: menuVisible ? 'flex' : 'flex' }}>
      <div className="menu-item" onClick={() => handleAction('New')}>
        <span className="icon">📄</span>
        <span>新建 (New)</span>
      </div>
      <div className="menu-separator"></div>
      <div className="menu-item" onClick={() => handleAction('Copy')}>
        <span className="icon">📋</span>
        <span>复制 (Copy)</span>
      </div>
      <div className="menu-item" onClick={() => handleAction('Cut')}>
        <span className="icon">✂️</span>
        <span>剪切 (Cut)</span>
      </div>
      <div className="menu-item" onClick={() => handleAction('Paste')}>
        <span className="icon">📋</span>
        <span>粘贴 (Paste)</span>
      </div>
      <div className="menu-separator"></div>
      <div className="menu-item" onClick={() => handleAction('Compress')}>
        <span className="icon">🗜️</span>
        <span>压缩 (Compress)</span>
      </div>
      <div className="menu-item" onClick={() => handleAction('Delete')}>
        <span className="icon">🗑️</span>
        <span>删除 (Delete)</span>
      </div>
      <div className="menu-separator"></div>
      <div className="menu-item" onClick={() => handleAction('Properties')}>
        <span className="icon">⚙️</span>
        <span>属性 (Properties)</span>
      </div>
    </div>
  );
}

export default App;
