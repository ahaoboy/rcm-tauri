import type { MenuItem, InvokeProps } from '../../types';
import { ZIP } from '../../system-commands';

/**
 * Archive extensions supported by the Rust backend's `Fmt` enum.
 * Compound extensions (`.tar.gz`) are listed first so they are matched
 * before single-segment ones.
 */
export const ARCHIVE_EXTS: string[] = [
  // TarGz
  '.tar.gz', '.tgz',
  // TarXz
  '.tar.xz', '.txz',
  // TarBz
  '.tar.bz2', '.tbz2', '.tbz',
  // TarZstd
  '.tar.zst', '.tzst', '.tzstd',
  // Tar (plain)
  '.tar',
  // Zip
  '.zip',
  // 7z
  '.7z',
];

/** Match a path against the backend-supported archive extensions. */
export function isZip(path: string): boolean {
  const lower = path.toLowerCase();
  return ARCHIVE_EXTS.some((ext) => lower.endsWith(ext));
}

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
