/**
 * Lite right-click menu — essentials only, no icon ribbon.
 */
import { Menu } from './menu';
import * as win11 from './configs/win11';
import * as custom from './configs/custom';

export default new Menu(
  [
    {
      items: [
        win11.newMenu(),
        win11.copy(),
        win11.copyAsPath(),
        // win11.trash(),
      ],
    },
    {
      items: [
        custom.vscode(),
        custom.terminal(),
        custom.unzip(),
        custom.zip(),
        // custom.mpv(),
        // custom.openFileLocation(),
        custom.ssh(),
        {
          label: 'More',
          items: [
            win11.openWith(),
            win11.properties(),
          ]
        }
      ],
    },
  ],
  // No icon ribbon
  [],
);
