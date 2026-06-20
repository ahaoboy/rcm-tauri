# RCM — Right Click Menu

<p align="center">
  <img src="public/icon-tray.ico" alt="RCM Icon" width="128" />
</p>

A customizable Windows right-click context menu, built with Tauri + React

[rcm-tauri.webm](https://github.com/user-attachments/assets/1b6f95bf-3b49-49bc-95ae-3c2663c3c90c)

## ⚠️ Important Notice

**This project is under active development and is NOT production-ready.**

- Breaking changes may occur at any time without prior notice.
- APIs, configuration formats, and behavior are subject to change.
- **Do NOT use this software in production environments.**

## Features

- Replace or enhance the standard Windows Explorer context menu.
- Switch between Windows 11 (compact) and Windows 10 (classic) menu styles.
- Custom menu scripts with JavaScript — define your own commands, conditions, and submenus.
- Remote sync — fetch your `rcm.js` from a URL to share across machines.
- Optional icon ribbon in the menu.
- Auto-start with Windows.
- System tray icon for quick settings access.

## Installation

1. Go to the [Releases](https://github.com/ahaoboy/rcm-tauri/releases) page.
2. Download the latest installer (`.msi` or `.exe`) for your system.
3. Run the installer and follow the on-screen instructions.

## Usage

After installation, the RCM icon appears in your system tray. Follow these steps to enable the custom context menu:

### Step-by-step Setup

1. **Switch to Classic style**
   - Right-click the RCM tray icon, then click **Classic**.
   - This enables the classic Windows 10 context menu style (required for the shell extension to work on Windows 11).

2. **Register the shell extension**
   - In the tray menu, click **Register**.
   - This registers `rcm_com.dll` — the shell extension that intercepts right-click events.

3. **Enable auto-start (optional)**
   - Click **Startup** in the tray menu to add RCM to your Windows startup programs.
   - A check mark (✓) indicates it is enabled.

4. **Apply the changes**
   - Click **Apply** to restart Windows Explorer and activate the shell extension.
   - Your right-click menu should now use the RCM style.
     [rcm-tauri.webm](https://github.com/user-attachments/assets/2c2c3a88-bb35-46d6-bea3-8d28bab55541)

## Custom Menu

RCM loads its menu from `rcm.js` located next to the executable. You can edit this file to tailor the right-click menu to your workflow.

### How it works

The menu is a tree of **groups** (sections) containing **items**. Each item can be:

- A **built-in action** (e.g. `open()`, `copy()`, `vscode()`, `terminal()`)
- A **submenu** (`{ label: "...", items: [...] }`)
- A **custom command** defined with `action` / `match` callbacks

Example — a minimal `rcm.js`:

```js
import { newMenu, copy, paste, terminal, Menu } from "rcm-kit"

export default new Menu(
  [
    {
      items: [newMenu(), terminal(), { label: "Clipboard", items: [copy(), paste()] }],
    },
  ],
  [],
)
```

### Defining custom commands

Use the `action` callback to define your own commands:

```js
function fsv() {
  return {
    key: "fsv",
    label: "Browse with fsv",
    action: (props) => {
      const targets = props.files.length ? props.files.map((f) => f.path) : ["."]
      return { cmd: "fsv", args: targets, cwd: props.cwd, window: "Visible" }
    },
  }
}
```

The `props` object provides:

| Field       | Type                                 | Description                                      |
| ----------- | ------------------------------------ | ------------------------------------------------ |
| `files`     | `{ path: string, isDir: boolean }[]` | Selected files (empty = background click)        |
| `cwd`       | `string`                             | Directory where the right-click occurred         |
| `env`       | `Record<string, string>`             | Environment variables (e.g. `{ OS: "Windows" }`) |
| `admin`     | `boolean`                            | Whether the process is running as admin          |
| `lang`      | `string`                             | System language (e.g. `"zh"`, `"en"`)            |
| `clipboard` | `{ has_text, has_image, has_files }` | Clipboard state snapshot                         |

The `action` return value:

| Field    | Type                                                     | Description             |
| -------- | -------------------------------------------------------- | ----------------------- |
| `cmd`    | `string`                                                 | Executable name or path |
| `args`   | `string[]`                                               | Command-line arguments  |
| `cwd`    | `string`                                                 | Working directory       |
| `window` | `"Hidden"` / `"Visible"` / `"Minimized"` / `"Maximized"` | Window mode             |

Use `match` to conditionally show items:

```js
{
  key: "only-for-images",
  label: "Convert to WebP",
  match: ({ files }) => files.some(f => /\.(png|jpg)$/i.test(f.path)),
  action: (props) => ({ cmd: "magick", args: [...], window: "Hidden" }),
}
```

### Syncing your menu remotely

RCM can fetch your `rcm.js` from a remote URL. Configure the URL in `rcm.config.json`:

```jsonc
{
  "url": "https://example.com/my-rcm.js",
}
```

Then click **Sync** in the tray menu to download the latest version. This is useful for:

- Keeping your menu in sync across multiple machines
- Sharing your menu configuration with a team
- Version-controlling your menu setup (e.g. in a GitHub Gist)

> **Note:** The `Sync` menu item only appears when `url` is configured.

## Community Menus

Here are some example configurations from the community. Feel free to submit yours via PR!

| Menu                                                  | Sync URL                                                                 | Preview                                                                                                                      |
| ----------------------------------------------------- | ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| [rcm-tauri](https://github.com/ahaoboy/rcm-tauri)       | `https://github.com/ahaoboy/rcm-tauri/releases/latest/download/rcm.js`   | <img width="400" alt="default menu" src="https://github.com/user-attachments/assets/9840dfb6-d74b-417c-ab35-2e8401745791" /> |
| [rcm-ahaoboy](https://github.com/ahaoboy/rcm-ahaoboy) | `https://github.com/ahaoboy/rcm-ahaoboy/releases/latest/download/rcm.js` | [rcm-tauri.webm](https://github.com/user-attachments/assets/1b6f95bf-3b49-49bc-95ae-3c2663c3c90c)                            |

> **Want to share your setup?** Open a PR adding your `rcm.js` to this section!
