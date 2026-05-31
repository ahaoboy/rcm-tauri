import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/** Win11 "Cut" */
export function cut(): MenuItem {
  return { key: 'cut', label: t('cut'), icon: '✂️' };
}

/** Win11 "Copy" — copy file(s) to clipboard (only shown when files are selected) */
export function copy(): MenuItem {
  return {
    key: 'copy',
    label: t('copy'),
    icon: '📋',
    match: (props: InvokeProps) => props.files.length > 0,
    action: (props: InvokeProps) => ({
      exe: '@copy',
      args: props.files.map(f => f.path),
    }),
  };
}

/** Win11 "Paste" — paste files from clipboard (only when clipboard has files) */
export function paste(): MenuItem {
  return {
    key: 'paste',
    label: t('paste'),
    icon: '📄',
    match: (props: InvokeProps) => props.files.length === 0 && props.clipboard?.has_files === true,
    action: (props: InvokeProps) => ({
      exe: '@paste-files',
      args: [],
      cwd: props.cwd,
    }),
  };
}

/** Win11 "Rename" */
export function rename(): MenuItem {
  return { key: 'rename', label: t('rename'), icon: '✏️' };
}

/** Win11 "Trash" — move to recycle bin (only shown when files are selected) */
export function trash(): MenuItem {
  return {
    key: 'trash',
    label: t('trash'),
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
