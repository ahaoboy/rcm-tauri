import type { MenuItem, InvokeProps } from '../types';
import { t } from '../i18n';

/**
 * Win11 "Create shortcut" — creates a .lnk shortcut to the selected file.
 */
export function createShortcut(): MenuItem {
  return {
    key: 'create-shortcut',
    label: t('create.shortcut'),
    icon: '🔗',
    match: (props: InvokeProps) => props.files.length > 0,
    action: (props: InvokeProps) => {
      const file = props.files[0];
      return {
        exe: 'powershell',
        args: [
          '-Command',
          `$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('${file.path}.lnk'); $s.TargetPath = '${file.path}'; $s.Save()`,
        ],
        cwd: props.cwd,
        window: 'Hidden',
      };
    },
  };
}
