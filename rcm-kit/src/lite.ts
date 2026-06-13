/**
 * Lite right-click menu — essentials only, no icon ribbon.
 */
import { Menu } from './index';
import * as menus from './index';

export default new Menu(
  [
    {
      items: [
        menus.newMenu(),
        menus.copyAs(),
        // menus.trash(),
      ],
    },
    {
      items: [
        menus.vscode(),
        menus.terminal(),
        menus.unzip(),
        menus.zip(),
        menus.fsv(),
      ],
    },
    {
      items: [
        // menus.mpv(),
        // menus.openFileLocation(),
        menus.ssh(),
        {
          label: 'More',
          items: [
            menus.copy(),
            menus.paste(),
            menus.openWith(),
            menus.openFileLocation(),
            menus.groupBy(),
            menus.sortBy(),
            menus.properties(),
          ]
        }
      ]
    }
  ],
  // No icon ribbon
  [],
);
