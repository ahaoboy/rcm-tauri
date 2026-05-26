import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/**
 * "Open in Windows Terminal" menu item — opens WT at the selected location.
 */
export function terminal(labelKey = 'open.in.wt'): MenuItem {
  return {
    key: 'terminal',
    label: t(labelKey),
    icon: '>_',
    match: (props: InvokeProps) => props.files.length > 0,
    action: (props: InvokeProps) => {
      const dir = props.files[0].isDir ? props.files[0].path : props.cwd;
      return { exe: 'wt', args: ['-d', dir], cwd: props.cwd, window: 'Show' };
    },
  };
}
