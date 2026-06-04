
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
