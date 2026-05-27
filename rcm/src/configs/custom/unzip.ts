import type { MenuItem, InvokeProps } from '../../types';
import { UNZIP } from '../../system-commands';

/** Extensions supported by easy-archive's `Fmt::guess`. */
const ARCHIVE_RE = /\.(zip|tar|tar\.gz|tgz|tar\.xz|txz|tar\.bz2|tbz2|tbz|tar\.zst|tzstd|tzst|7z)$/i;

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
      props.files.length > 0 && props.files.every((f) => ARCHIVE_RE.test(f.name)),
    action: (props: InvokeProps) => ({
      exe: UNZIP,
      args: props.files.map((f) => f.path),
      cwd: props.cwd,
      window: 'Hidden',
    }),
  };
}
