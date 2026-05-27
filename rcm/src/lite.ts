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
        win11.deleteItem(),
      ],
    },
    {
      items: [
        custom.terminal(),
        custom.unzip(),
        custom.zip(),
        custom.mpv(),
        win11.properties(),
      ],
    },
  ],
  // No icon ribbon
  [],
);
