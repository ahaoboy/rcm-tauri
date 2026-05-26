import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/**
 * Win11 "Edit" — opens file in Notepad or the default text editor.
 */
export function edit(): MenuItem {
  return {
    key: 'edit',
    label: t('edit'),
    icon: '✏️',
    match: (props: InvokeProps) => {
      if (!props.files.length) return false;
      // editable text-ish files
      const name = props.files[0].name.toLowerCase();
      return /\.(txt|ini|cfg|log|md|xml|json|yml|yaml|toml|bat|cmd|ps1|reg)$/.test(name);
    },
    action: (props: InvokeProps) => ({
      exe: 'notepad',
      args: [props.files[0].path],
      window: 'Show',
    }),
  };
}
