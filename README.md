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
- Remote pull — fetch `rcm.js`, `style.css`, and `rcm.config.json` from URLs to share across machines.
- Built-in config editor with syntax highlighting for live editing.
- Optional icon ribbon in the menu.
- Auto-start with Windows.
- System tray icon for quick settings access.
- Dark / light theme support.

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

### Pulling updates remotely

RCM can fetch your configuration files from remote URLs. Configure the URLs in `rcm.config.json`:

```jsonc
{
  // GitHub release URLs (set as defaults when no URLs are configured)
  "js_url": "https://github.com/ahaoboy/rcm-tauri/releases/latest/download/rcm.js",
  "css_url": "https://github.com/ahaoboy/rcm-tauri/releases/latest/download/style.css",
  // config URL is empty by default — set it to pull rcm.config.json updates
  "config_url": "",
}
```

> **Defaults:** `js_url` and `css_url` point to this repo's latest release. `config_url` is empty by default.

**Pull from tray:** Open the tray, click **Pull ▸ JS** / **CSS** / **Config** to download the latest version of each file.

**Pull from editor:** Open the config editor (tray → **Config**), switch to the file tab, and click **⬇️ Pull**. On success the editor reloads automatically; on failure an error window pops up.

> **Note:** The **Pull** submenu only appears in the tray when at least one URL is configured. After changing URLs, restart RCM for the **Pull** menu to update.

### Config Editor

RCM includes a built-in editor for `rcm.js`, `style.css`, and `rcm.config.json` with syntax highlighting.

- Open via tray → **Config**, or run `rcm-tauri.exe config` from the command line.
- **💾 Save** — writes the file and broadcasts CSS changes to all open menus instantly.
- **⬇️ Pull** — downloads the latest version from the configured remote URL.
- **🔄 Reload** — discards unsaved changes and re-reads the file from disk.
- **📂 Open** — opens the file with your system default editor.

## Custom Style

RCM also supports custom CSS through `style.css` located next to the executable.

### How it works

- On first launch, RCM writes the default `style.css` to the executable directory.
- If you edit or replace `style.css`, the menu will use your version.
- Use the config editor (**Config** → `style.css` tab) or **Pull ▸ CSS** to fetch remote updates.
- When you save `style.css` in the config editor, all open menus receive the update instantly.
- Click **Reset** in the tray menu to restore the default `style.css` (along with `rcm.config.json` and `rcm.js`).

### What you can customize

The CSS uses design tokens (custom properties) prefixed with `--rcm-`. You can override any of them to change the menu appearance without touching the layout rules:

```css
:root {
  /* Colors */
  --rcm-bg: #fff5f7; /* Menu background */
  --rcm-border: #f2c4d0; /* Menu border */
  --rcm-text: #b84664; /* Text color */
  --rcm-text-disabled: rgba(…); /* Disabled item text */

  /* Items */
  --rcm-item-hover: #ffe8ef; /* Hover background */
  --rcm-item-active: #ffd4e0; /* Active/pressed background */
  --rcm-item-height: 32px; /* Row height */
  --rcm-item-radius: 10px; /* Corner roundness */

  /* Layout */
  --rcm-font-family: "Segoe UI Variable", …;
  --rcm-font-size: 13px;
  --rcm-icon-size: 18px;
  --rcm-radius: 12px; /* Menu container roundness */
  --rcm-padding: 3px; /* Inner padding */

  /* Shadows */
  --rcm-shadow: 0 1px 2px rgba(…), …;

  /* Separator & Ribbon */
  --rcm-separator: #f2c4d0;
  --rcm-ribbon-border: #f2c4d0;

  /* Arrows & Focus */
  --rcm-arrow-color: #e0a0b4;
  --rcm-accent: #ff85a2; /* Focus ring color */
}
```

Dark mode overrides use `@media (prefers-color-scheme: dark)` or the class selectors `.rcm-light` / `.rcm-dark`.

## Community Menus

Here are some example configurations from the community. Feel free to submit yours via PR!

| Menu                                                                                                                                       | Preview                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| [rcm-tauri](https://github.com/ahaoboy/rcm-tauri) · [Pull URL](https://github.com/ahaoboy/rcm-tauri/releases/latest/download/rcm.js)       | <img width="400" alt="default menu" src="https://github.com/user-attachments/assets/9840dfb6-d74b-417c-ab35-2e8401745791" /> |
| [rcm-ahaoboy](https://github.com/ahaoboy/rcm-ahaoboy) · [Pull URL](https://github.com/ahaoboy/rcm-ahaoboy/releases/latest/download/rcm.js) | <img width="400" alt="rcm-ahaoboy" src="https://github.com/user-attachments/assets/0fac2734-38e3-4836-82a0-5cfb8684ef7d" />  |

> **Want to share your setup?** Open a PR adding your `rcm.js` to this section!
