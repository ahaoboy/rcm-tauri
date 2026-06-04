import type { MenuItem, InvokeProps } from '../types';
import { t } from '../i18n';
import { GROUP_BY } from '../consts';

const GROUP_ITEMS: [string, string, string][] = [
  ['name',          'group.by.name',          '📋'],
  ['date-modified', 'group.by.date.modified', '📅'],
  ['type',          'group.by.type',          '📁'],
  ['size',          'group.by.size',          '📊'],
  ['date-created',  'group.by.date.created',  '📆'],
  ['none',          'group.by.none',          '🚫'],
];

/**
 * Win11 "Group by" submenu — change how files are grouped in the current folder.
 * Only shown when no files are selected (background right-click).
 */
export function groupBy(): MenuItem {
  return {
    key: 'group-by',
    label: t('group.by'),
    icon: '📑',
    match: (props: InvokeProps) => props.files.length === 0,
    items: GROUP_ITEMS.map(([key, labelKey, icon]) => ({
      key: `group-by-${key}`,
      label: t(labelKey),
      icon,
      action: (props: InvokeProps) => ({
        exe: GROUP_BY,
        args: [key],
        cwd: props.cwd,
        window: 'Hidden',
      }),
    })),
  };
}
