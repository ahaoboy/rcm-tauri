import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/**
 * "Open with VS Code" menu item.
 */
export function vscode(labelKey = 'open.with.vscode'): MenuItem {
  return {
    key: 'vscode',
    label: t(labelKey),
    icon: '💻',
    action: (props: InvokeProps) => {
      const targets = props.files.length
        ? props.files.map(f => f.path)
        : ['.'];
      return { exe: 'code', args: targets, cwd: props.cwd, window: 'Hidden' };
    },
  };
}
