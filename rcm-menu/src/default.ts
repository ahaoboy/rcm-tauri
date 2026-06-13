/**
 * Default menus right-click menu composed from built-in configs.
 * Includes menus app integrations (VS Code, mpv, Terminal).
 */
import { Menu } from './menu';
import * as menus from './menus';

export default new Menu(
  // ── Main vertical menu groups ───────────────────────────────────
  [
    {
      // Primary actions
      items: [
        menus.vscode(),
        menus.open(),
        menus.openWith(),
        menus.terminal(),
        menus.ssh(),
        menus.unzip(),
        menus.zip(),
        menus.edit(),
        menus.print(),
        // menus.mpv(),
        menus.runAsAdmin(),
        menus.groupBy(),
        menus.sortBy(),
      ],
    },
    {
      // Pin & send
      items: [
        menus.pinToStart(),
        menus.pinToTaskbar(),
        menus.sendTo(),
      ],
    },
    {
      // Clipboard & file ops
      items: [
        menus.cut(),
        menus.copy(),
        menus.copyAs(),
        menus.paste(),
        menus.createShortcut(),
        menus.openFileLocation(),
        menus.trash(),
        menus.rename(),
        menus.selectAll(),
      ],
    },
    {
      // Meta
      items: [
        menus.restorePreviousVersions(),
        menus.properties(),
      ],
    },
  ],
  // ── Icon ribbon (top bar) ───────────────────────────────────────
  [
    menus.cut(),
    menus.copy(),
    menus.paste(),
    menus.rename(),
    menus.share(),
    menus.trash(),
  ],
);
