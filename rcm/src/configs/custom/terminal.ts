import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/**
 * "Open in Windows Terminal" menu item — always visible.
 */
export function terminal(labelKey = 'open.in.wt'): MenuItem {
  return {
    key: 'terminal',
    label: t(labelKey),
    icon: '>_',
    action: (props: InvokeProps) => {
      return { exe: 'wt', args: ['-d', props.cwd], cwd: props.cwd, window: 'Show' };
    },
  };
}
