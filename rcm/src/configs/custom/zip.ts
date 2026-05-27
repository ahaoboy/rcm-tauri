import type { MenuItem, InvokeProps } from '../../types';
import { ZIP } from '../../system-commands';

/**
 * "Add to archive" — always visible.
 *
 * If files are selected, they are archived into `<firstName>.zip`.
 * On background click (no selection), the entire current directory is
 * archived into `<dirname>.zip`.  Name collisions are handled by the
 * Rust backend.
 */
export function zip(): MenuItem {
  return {
    key: 'zip',
    label: 'zip',
    icon: '🗜️',
    action: (props: InvokeProps) => {
      return {
        exe: ZIP,
        args: props.files.map((f) => f.path),
        cwd: props.cwd,
        window: 'Hidden',
      };
    },
  };
}
