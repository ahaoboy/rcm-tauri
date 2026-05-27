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

/** Win11 "Delete" — recycle bin for single, fast permanent for multi-select */
export function deleteItem(): MenuItem {
  return {
    key: 'delete',
    label: t('delete'),
    icon: '🗑️',
    action: (props: InvokeProps) => {
      const paths = props.files.map(f => f.path);
      if (paths.length === 0) return;
      // Multi-file: fast parallel permanent deletion
      if (paths.length > 1) {
        return { exe: '@delete', args: paths };
      }
      // Single file: move to recycle bin
      return { exe: '@trash', args: paths };
    },
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
