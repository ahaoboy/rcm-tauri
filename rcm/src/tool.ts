
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

// ── Executable detection (Windows) ──────────────────────────────────

/**
 * Windows file extensions considered executable.
 * Covers native binaries (.exe, .com, .scr, .pif), installers (.msi, .msix),
 * and scripts (.bat, .cmd, .ps1, .vbs, .wsf, .psm1, .psd1).
 */
export const EXECUTABLE_EXTS: string[] = [
  '.exe', '.com', '.scr', '.pif',
  '.msi', '.msix', '.appx',
];

/** Check whether the path represents a Windows binary executable. */
export function isExecutable(path: string): boolean {
  const lower = path.toLowerCase();
  return EXECUTABLE_EXTS.some((ext) => lower.endsWith(ext));
}
