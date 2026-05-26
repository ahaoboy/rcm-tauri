import type { MenuItem } from '../../types';
import { t } from '../../i18n';

/**
 * Win11 "Properties" — opens file/folder properties dialog.
 */
export function properties(): MenuItem {
  return {
    key: 'properties',
    label: t('properties'),
    icon: 'ℹ️',
    action: () => ({
      exe: 'properties',
      args: [],
      window: 'Show',
    }),
  };
}
