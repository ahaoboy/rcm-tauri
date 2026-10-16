import { readdirSync, statSync } from "fs"

import {
  TEXT_EXTS,
  VIDEO_EXTS,
  IMAGE_EXTS,
  PRINTABLE_EXTS,
  AUDIO_EXTS,
  SUBTITLE_EXTS,
} from "./consts"
import type { Entry, InvokeProps } from "./types"

/**
 * Archive extensions supported by the Rust backend's `Fmt` enum.
 * Compound extensions (`.tar.gz`) are listed first so they are matched
 * before single-segment ones.
 */
export const ARCHIVE_EXTS: string[] = [
  // TarGz
  ".tar.gz",
  ".tgz",
  // TarXz
  ".tar.xz",
  ".txz",
  // TarBz
  ".tar.bz2",
  ".tbz2",
  ".tbz",
  // TarZstd
  ".tar.zst",
  ".tzst",
  ".tzstd",
  // Tar (plain)
  ".tar",
  // Zip
  ".zip",
  // 7z
  ".7z",
]

/** Match a path against the backend-supported archive extensions. */
export function isZip(path: string): boolean {
  const lower = path.toLowerCase()
  return ARCHIVE_EXTS.some((ext) => lower.endsWith(ext))
}

// ── Executable detection (Windows) ──────────────────────────────────

/**
 * Windows file extensions considered executable.
 * Covers native binaries (.exe, .com, .scr, .pif), installers (.msi, .msix),
 * and scripts (.bat, .cmd, .ps1, .vbs, .wsf, .psm1, .psd1).
 */
export const EXECUTABLE_EXTS: string[] = [".exe", ".com", ".scr", ".pif", ".msi", ".msix", ".appx"]

/** Check whether the path represents a Windows binary executable. */
export function isExecutable(path: string): boolean {
  const lower = path.toLowerCase()
  return EXECUTABLE_EXTS.some((ext) => lower.endsWith(ext))
}

// ── Path helpers ────────────────────────────────────────────────────

/** Extract the file name (with extension) from a Windows or Unix path. */
export function basename(path: string): string {
  return path.split(/[\\/]/).pop() ?? path
}

/** Check whether `path` ends with any of the given extensions (case-insensitive). */
export function hasExt(path: string, ...exts: string[]): boolean {
  const lower = path.toLowerCase()
  return exts.some((ext) => lower.endsWith(ext.toLowerCase()))
}

export const isText = (path: string) => hasExt(path, ...TEXT_EXTS)
export const isVideo = (path: string) => hasExt(path, ...VIDEO_EXTS)
export const isAudio = (path: string) => hasExt(path, ...AUDIO_EXTS)
export const isImage = (path: string) => hasExt(path, ...IMAGE_EXTS)
export const isSubtitle = (path: string) => hasExt(path, ...SUBTITLE_EXTS)
export const isMedia = (path: string) => isVideo(path) || isAudio(path)
export const isPrintable = (path: string) => hasExt(path, ...PRINTABLE_EXTS)

// ── Shortcut helpers (Start Menu / Desktop) ────────────────────────

/** Extract the file stem (name without extension) from a path. */
export function fileStem(path: string): string {
  const name = basename(path)
  const dot = name.lastIndexOf(".")
  return dot > 0 ? name.slice(0, dot) : name
}

/**
 * True when an entry list (`startmenu` / `desktop`) already has an entry
 * for `path`. Matches by resolved `.lnk` target first, then by same stem.
 */
export function hasShortcut(list: Entry[], path: string): boolean {
  const norm = path.toLowerCase()
  const stem = fileStem(path)
  return list.some(
    (lnk) =>
      (lnk.target !== null && lnk.target.toLowerCase() === norm) || fileStem(lnk.path) === stem,
  )
}

// ── Directory helpers ───────────────────────────────────────────────

/** Check whether `path` exists and is a directory. */
export function isDir(path: string): boolean {
  try {
    return statSync(path).isDirectory()
  } catch {
    return false
  }
}

/**
 * True when `path` is a directory that contains nothing at all.
 *
 * Non-recursive by design: a folder holding only subfolders is *not* empty,
 * because it is not empty from the user's point of view.
 *
 * A path that cannot be read — missing, not a directory, or refused by
 * `readdirSync` — reports `false`. Callers use this to *hide* menu items, and
 * guessing "empty" for a path we cannot inspect would hide actions that might
 * well have worked.
 */
export function isEmptyDir(path: string): boolean {
  try {
    return isDir(path) && readdirSync(path).length === 0
  } catch {
    return false
  }
}

/**
 * True when a right-click is aimed at nothing but empty folders.
 *
 * Covers both ways that happens: an explorer background click inside an empty
 * folder (`files` is empty and `cwd` is that folder), and a selection made up
 * solely of folders that hold nothing. Archiving either one produces an archive
 * with no content, so items that would do exactly that stay hidden.
 *
 * An empty selection **outside** any directory is `false`: nothing to archive is
 * a property of the folder, not of the click.
 */
export function isEmptyDirTarget({ files, cwd }: Pick<InvokeProps, "files" | "cwd">): boolean {
  if (files.length === 0) return isEmptyDir(cwd)
  return files.every((f) => isEmptyDir(f.path))
}
