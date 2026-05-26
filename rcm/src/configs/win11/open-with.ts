import type { MenuItem } from '../../types';
import { t } from '../../i18n';

/**
 * Win11 "Open with" submenu — pick a program to open the selected file.
 */
export function openWith(): MenuItem {
  return {
    key: 'open-with',
    label: t('open.with'),
    icon: '🔽',
    items: [
      {
        key: 'open-with-choose',
        label: t('open.with'),
        icon: '📎',
        action: () => ({
          exe: 'OpenWith',
          args: [],
          window: 'Show',
        }),
      },
    ],
  };
}
