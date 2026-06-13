import type { MenuItem, InvokeProps } from '../types';
import { OPEN_FILE_LOCATION } from '../consts';
import { t } from '../i18n';

/**
 * "Open file location" — opens Explorer with the file selected.
 *
 * For shortcut (.lnk) files, the Rust backend resolves the target
 * and opens that location instead of the .lnk itself.
 */
export function openFileLocation(): MenuItem {
  return {
    key: 'open-file-location',
    label: t('open.file.location'),
    icon: '📂',
    match: (props: InvokeProps) => {
      const file = props.files[0];
      return props.files.length === 1 && !file.isDir && file.path.endsWith('.lnk');
    },
    action: (props: InvokeProps) => ({
      exe: OPEN_FILE_LOCATION,
      args: [props.files[0].path],
      cwd: props.cwd,
      window: 'Hidden',
    }),
  };
}
