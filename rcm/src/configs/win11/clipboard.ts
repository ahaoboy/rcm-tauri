import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/** Win11 "Cut" */
export function cut(): MenuItem {
  return { key: 'cut', label: t('cut'), icon: '✂️' };
}

/** Win11 "Copy" */
export function copy(): MenuItem {
  return { key: 'copy', label: t('copy'), icon: '📋' };
}

/** Win11 "Paste" */
export function paste(): MenuItem {
  return { key: 'paste', label: t('paste'), icon: '📄' };
}

/** Win11 "Rename" */
export function rename(): MenuItem {
  return { key: 'rename', label: t('rename'), icon: '✏️' };
}

/** Win11 "Delete" — move to recycle bin (only shown when files are selected) */
export function deleteItem(): MenuItem {
  return {
    key: 'delete',
    label: t('delete'),
    icon: '🗑️',
    match: (props: InvokeProps) => props.files.length > 0,
    action: (props: InvokeProps) => ({
      exe: '@trash',
      args: props.files.map(f => f.path),
    }),
  };
}

/** Win11 "Select all" */
export function selectAll(): MenuItem {
  return { key: 'select-all', label: t('select.all'), icon: '🔲' };
}

/** Win11 "Refresh" */
export function refresh(): MenuItem {
  return { key: 'refresh', label: t('refresh'), icon: '🔄' };
}
