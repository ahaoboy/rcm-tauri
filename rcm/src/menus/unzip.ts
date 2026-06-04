import type { MenuItem, InvokeProps } from '../types';
import { UNZIP } from '../consts';
import { isZip } from '../tool';

/**
 * "Extract here" — visible when all selected files are archives.
 *
 * Each archive is extracted into a subdirectory named after the
 * archive (collision-safe, handled by Rust).
 */
export function unzip(): MenuItem {
  return {
    key: 'unzip',
    label: 'unzip',
    icon: '📦',
    match: (props: InvokeProps) =>
      props.files.length > 0 && props.files.every((f) => isZip(f.path)),
    action: (props: InvokeProps) => ({
      exe: UNZIP,
      args: props.files.map((f) => f.path),
      cwd: props.cwd,
      window: 'Hidden',
    }),
  };
}
