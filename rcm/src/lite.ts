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
        win11.copyAs(),
        // win11.trash(),
      ],
    },
    {
      items: [
        custom.vscode(),
        custom.terminal(),
        custom.unzip(),
        custom.zip(),
        custom.fsv(),
      ],
    },
    {
      items: [
        // custom.mpv(),
        // custom.openFileLocation(),
        custom.ssh(),
        {
          label: 'More',
          items: [
            win11.copy(),
            win11.paste(),
            win11.openWith(),
            custom.openFileLocation(),
            win11.groupBy(),
            win11.sortBy(),
            win11.properties(),
          ]
        }
      ]
    }
  ],
  // No icon ribbon
  [],
);
