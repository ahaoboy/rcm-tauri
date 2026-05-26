import type { MenuItem } from '../../types';
import { t } from '../../i18n';

/**
 * Win11 "New" submenu — create new files/folders.
 */
export function newMenu(): MenuItem {
  return {
    key: 'new',
    label: t('new'),
    icon: '➕',
    items: [
      {
        key: 'new-folder',
        label: t('new.folder'),
        icon: '📁',
        action: () => ({
          exe: 'cmd',
          args: ['/c', 'mkdir', 'New folder'],
          window: 'Hidden',
        }),
      },
      {
        key: 'new-text-document',
        label: t('new.text.document'),
        icon: '📝',
        action: () => ({
          exe: 'cmd',
          args: ['/c', 'echo.', '>', 'New Text Document.txt'],
          window: 'Hidden',
        }),
      },
    ],
  };
}
