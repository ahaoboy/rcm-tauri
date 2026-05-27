/**
 * Default Win11 right-click menu composed from built-in configs.
 * Includes custom app integrations (VS Code, mpv, Terminal).
 */
import { Menu } from './menu';
import * as win11 from './configs/win11';
import * as custom from './configs/custom';

export default new Menu(
  // ── Main vertical menu groups ───────────────────────────────────
  [
    {
      // Primary actions
      items: [
        custom.vscode(),
        win11.open(),
        win11.openWith(),
        custom.terminal(),
        custom.unzip(),
        custom.zip(),
        win11.edit(),
        win11.print(),
        custom.mpv(),
        win11.runAsAdmin(),
      ],
    },
    {
      // Pin & send
      items: [
        win11.pinToStart(),
        win11.pinToTaskbar(),
        win11.sendTo(),
      ],
    },
    {
      // Clipboard & file ops
      items: [
        win11.cut(),
        win11.copy(),
        win11.copyAsPath(),
        win11.paste(),
        win11.createShortcut(),
        win11.deleteItem(),
        win11.rename(),
        win11.selectAll(),
      ],
    },
    {
      // Meta
      items: [
        win11.restorePreviousVersions(),
        win11.properties(),
      ],
    },
  ],
  // ── Icon ribbon (top bar) ───────────────────────────────────────
  [
    win11.cut(),
    win11.copy(),
    win11.paste(),
    win11.rename(),
    win11.share(),
    win11.deleteItem(),
  ],
);
