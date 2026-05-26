import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/**
 * Win11 "Copy as path" — copies the full path(s) to clipboard.
 */
export function copyAsPath(): MenuItem {
  return {
    key: 'copy-as-path',
    label: t('copy.as.path'),
    icon: '📎',
    match: (props: InvokeProps) => props.files.length > 0,
    action: (props: InvokeProps) => ({
      exe: 'cmd',
      args: ['/c', 'echo', props.files.map(f => f.path).join('\n'), '|', 'clip'],
      cwd: props.cwd,
      window: 'Hidden',
    }),
  };
}
