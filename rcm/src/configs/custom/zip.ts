import type { MenuItem, InvokeProps } from '../../types';
import { ZIP } from '../../system-commands';
import { isZip } from '../../tool';

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
    match: ({ files }) => {
      // Only hide when a single archive file is selected — pointless to re-archive it.
      return !(files.length === 1 && !files[0].isDir && isZip(files[0].name));
    },
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
