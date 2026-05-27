import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/**
 * Win11 "Copy as path" — copies path(s) to clipboard (slash-separated).
 * Without a selection, copies the current folder path.
 */
export function copyAsPath(): MenuItem {
  return {
    key: 'copy-as-path',
    label: t('copy.as.path'),
    icon: '📎',
    action: (props: InvokeProps) => {
      const paths = props.files.map(f => f.path);
      if (paths.length === 0) {
        // No files selected → copy the current directory path
        return { exe: '@copy-path', args: [] };
      }
      return { exe: '@copy-path', args: paths };
    },
  };
}
