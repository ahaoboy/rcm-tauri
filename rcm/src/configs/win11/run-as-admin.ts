import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/**
 * Win11 "Run as administrator" — for executables, scripts, and MSI installers.
 */
export function runAsAdmin(): MenuItem {
  return {
    key: 'run-as-admin',
    label: t('run.as.admin'),
    icon: '🛡️',
    admin: true,
    match: (props: InvokeProps) => {
      if (!props.files.length) return false;
      const name = props.files[0].name.toLowerCase();
      return /\.(exe|bat|cmd|ps1|msi|vbs)$/.test(name);
    },
    action: (props: InvokeProps) => ({
      exe: props.files[0].path,
      admin: true,
      window: 'Show',
    }),
  };
}
