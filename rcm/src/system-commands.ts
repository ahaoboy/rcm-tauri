/**
 * System Command Identifiers
 *
 * All built-in system commands use the `@` prefix convention:
 *   { exe: '@unzip', args: [...], cwd: '...' }
 *
 * The Rust backend intercepts these and routes them to the
 * corresponding `SystemCommand` variant for native execution.
 */

// ── Archive operations ───────────────────────────────────────────────

/** Extract a zip archive to the current or specified directory. */
export const UNZIP = '@unzip';

/** Create a zip archive from selected files/folders. */
export const ZIP = '@zip';

// ── File operations ──────────────────────────────────────────────────

/** Rename a file/folder with automatic collision avoidance. */
export const RENAME = '@rename';

/** Create a new empty file at the target location. */
export const NEW_FILE = '@new-file';

/** Create a new folder at the target location. */
export const NEW_FOLDER = '@new-folder';

/** Move file(s) to the recycle bin. */
export const TRASH = '@trash';

/** Permanently delete file(s)/folder(s) in parallel (fast, for multi-select). */
export const DELETE = '@delete';

/** Open the Windows file/folder properties dialog. */
export const PROPERTIES = '@properties';

/** Copy file(s) to clipboard as file-drop data (like Ctrl+C in Explorer). */
export const COPY = '@copy';

// ── Utilities ────────────────────────────────────────────────────────

/** Open Windows "Open With" dialog for the selected file. */
export const OPEN_WITH = '@open-with';

/** Copy the full path(s) of selected items to clipboard (Linux-style separators). */
export const COPY_PATH = '@copy-path';

/** Copy the file name(s) of selected items to clipboard. */
export const COPY_NAME = '@copy-name';

/** Copy the file content(s) as base64 to clipboard. */
export const COPY_BASE64 = '@copy-base64';

/** Open file location in Explorer, resolving shortcut targets. */
export const OPEN_FILE_LOCATION = '@open-file-location';

/** Paste files from clipboard to the current directory. */
export const PASTE_FILES = '@paste-files';

// ── All command identifiers as a set (for validation) ────────────────

/** Set of all recognized system command identifiers. */
export const ALL: ReadonlySet<string> = new Set([
  UNZIP,
  ZIP,
  RENAME,
  NEW_FILE,
  NEW_FOLDER,
  TRASH,
  DELETE,
  PROPERTIES,
  COPY,
  OPEN_WITH,
  COPY_PATH,
  COPY_NAME,
  COPY_BASE64,
  OPEN_FILE_LOCATION,
  PASTE_FILES,
]);
