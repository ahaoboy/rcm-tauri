# RCM — Right Click Menu

<p align="center">
  <img src="public/icon-tray.ico" alt="RCM Icon" width="128" />
</p>

A customizable Windows right-click context menu, built with Tauri + React (with an experimental [Slint](https://slint.dev/) backend).

## ⚠️ Important Notice

**This project is under active development and is NOT production-ready.**

- Breaking changes may occur at any time without prior notice.
- APIs, configuration formats, and behavior are subject to change.
- **Do NOT use this software in production environments.**

## Features

- Replace or enhance the standard Windows Explorer context menu.
- Switch between Windows 11 (compact) and Windows 10 (classic) menu styles.
- Custom menu scripts with JavaScript (lite / full mode).
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

### Tray Menu Reference

| Menu Item            | Type   | Description                                     |
| -------------------- | ------ | ----------------------------------------------- |
| **Win11**            | Radio  | Windows 11 compact menu style                   |
| **Classic**          | Radio  | Windows 10 classic expanded menu style          |
| **Enable / Disable** | Check  | Toggle the custom context menu on/off           |
| **Register**         | Button | Register `rcm_com.dll` shell extension          |
| **Unregister**       | Button | Unregister the shell extension                  |
| **DumpEnv**          | Button | Dump environment variables to `<exe>.env`       |
| **Lite**             | Radio  | Lite menu mode (minimal items)                  |
| **Full**             | Radio  | Full menu mode (all items)                      |
| **Icons**            | Check  | Show/hide the icon ribbon in the menu           |
| **Dev**              | Check  | Toggle dev mode (menu stays open on focus loss) |
| **Log**              | Check  | Toggle file logging to `<exe>.log`              |
| **Startup**          | Check  | Toggle auto-start with Windows                  |
| **Apply**            | Button | Restart Windows Explorer to apply changes       |
| **Reset**            | Button | Reset all config and menu files to defaults     |
| **Quit**             | Button | Exit the application                            |
