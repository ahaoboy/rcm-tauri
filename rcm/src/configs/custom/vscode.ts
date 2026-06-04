import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';
import { isZip } from '../../tool';

/**
 * "Open with VS Code" menu item.
 */
export function vscode(labelKey = 'code'): MenuItem {
  return {
    key: 'vscode',
    label: t(labelKey),
    icon: '💻',
    match: ({ files }) => !files.every(f => isZip(f.path)),
    action: (props: InvokeProps) => {
      const targets = props.files.length
        ? props.files.map(f => f.path)
        : ['.'];
      return { exe: 'code', args: targets, cwd: props.cwd, window: 'Hidden' };
    },
  };
}
