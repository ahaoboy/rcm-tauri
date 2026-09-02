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
export const UNZIP = "@unzip"

/** Create a zip archive from selected files/folders. */
export const ZIP = "@zip"

// ── File operations ──────────────────────────────────────────────────

/** Rename a file/folder with automatic collision avoidance. */
export const RENAME = "@rename"

/** Create a new empty file at the target location. */
export const NEW_FILE = "@new-file"

/** Create a new folder at the target location. */
export const NEW_FOLDER = "@new-folder"

/** Move file(s) to the recycle bin. */
export const TRASH = "@trash"

/** Permanently delete file(s)/folder(s) in parallel (fast, for multi-select). */
export const DELETE = "@delete"

/** Open the Windows file/folder properties dialog. */
export const PROPERTIES = "@properties"

/** Copy file(s) to clipboard as file-drop data (like Ctrl+C in Explorer). */
export const COPY = "@copy"

// ── Utilities ────────────────────────────────────────────────────────

/** Open Windows "Open With" dialog for the selected file. */
export const OPEN_WITH = "@open-with"

/** Copy the full path(s) of selected items to clipboard (Linux-style separators). */
export const COPY_PATH = "@copy-path"

/** Copy the file name(s) of selected items to clipboard. */
export const COPY_NAME = "@copy-name"

/** Copy the file content(s) as base64 to clipboard. */
export const COPY_BASE64 = "@copy-base64"

/** Resolve a .lnk shortcut target path and copy to clipboard. */
export const COPY_TARGET = "@copy-target"

/** Open file location in Explorer, resolving shortcut targets. */
export const OPEN_FILE_LOCATION = "@open-file-location"

/** Paste files from clipboard to the current directory. */
export const PASTE_FILES = "@paste-files"

/** Change the folder's group-by setting (Name, Date, Type, Size, etc.). */
export const GROUP_BY = "@group-by"

/** Change the folder's sort-by setting (Name, Date, Type, Size, etc.). */
export const SORT_BY = "@sort-by"

// ── Disk operations ──────────────────────────────────────────────────

/** Open the Windows "Format" dialog for a drive. */
export const FORMAT = "@format"

/** Eject a removable drive. */
export const EJECT = "@eject"

// ── Start Menu & Quick Access ──────────────────────────────────────

/** Pin a file to the Start Menu. */
export const PIN_TO_START = "@pin-to-start"

/** Unpin a file from the Start Menu. */
export const UNPIN_FROM_START = "@unpin-from-start"

/** Add a file/folder to Quick Access. */
export const ADD_TO_QUICK_ACCESS = "@add-to-quick-access"

/** Remove a file/folder from Quick Access. */
export const REMOVE_FROM_QUICK_ACCESS = "@remove-from-quick-access"

/** Add an .exe to Windows startup (autorun). */
export const ADD_TO_AUTORUN = "@add-to-autorun"

/** Remove an .exe from Windows startup (autorun). */
export const REMOVE_FROM_AUTORUN = "@remove-from-autorun"

/** Add a desktop shortcut for the selected file/folder. */
export const ADD_TO_DESKTOP = "@add-to-desktop"

/** Remove the desktop shortcut(s) pointing to the selected file/folder. */
export const REMOVE_FROM_DESKTOP = "@remove-from-desktop"

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
  COPY_TARGET,
  OPEN_FILE_LOCATION,
  PASTE_FILES,
  GROUP_BY,
  SORT_BY,
  FORMAT,
  EJECT,
  PIN_TO_START,
  UNPIN_FROM_START,
  ADD_TO_QUICK_ACCESS,
  REMOVE_FROM_QUICK_ACCESS,
  ADD_TO_AUTORUN,
  REMOVE_FROM_AUTORUN,
  ADD_TO_DESKTOP,
  REMOVE_FROM_DESKTOP,
])

export const TEXT_EXTS = [
  ".txt",
  ".ini",
  ".cfg",
  ".log",
  ".md",
  ".xml",
  ".json",
  ".yml",
  ".yaml",
  ".toml",
  ".bat",
  ".cmd",
  ".ps1",
  ".reg",
  ".csv",
  ".tsv",
  ".tex",
  ".rst",
  ".org",
  // programming languages
  ".rs",
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".py",
  ".pyi",
  ".pyx",
  ".c",
  ".h",
  ".cpp",
  ".hpp",
  ".cc",
  ".hh",
  ".cxx",
  ".hxx",
  ".cs",
  ".vb",
  ".fs",
  ".fsx",
  ".go",
  ".java",
  ".kt",
  ".kts",
  ".scala",
  ".groovy",
  ".rb",
  ".rake",
  ".gemspec",
  ".php",
  ".phtml",
  ".swift",
  ".lua",
  ".r",
  ".R",
  ".pl",
  ".pm",
  ".sh",
  ".bash",
  ".zsh",
  ".fish",
  ".sql",
  ".psql",
  ".dart",
  ".elm",
  ".erl",
  ".hrl",
  ".ex",
  ".exs",
  ".hs",
  ".lhs",
  ".ml",
  ".mli",
  ".nim",
  ".zig",
  ".vue",
  ".svelte",
  ".html",
  ".htm",
  ".css",
  ".scss",
  ".sass",
  ".less",
  ".styl",
  ".graphql",
  ".gql",
  ".proto",
  ".diff",
  ".patch",
  ".lock",
]

export const VIDEO_EXTS = [
  ".3g2",
  ".3gp",
  ".asf",
  ".avi",
  ".f4v",
  ".flv",
  ".h264",
  ".h265",
  ".m2ts",
  ".m4v",
  ".mkv",
  ".mov",
  ".mp4",
  ".mp4v",
  ".mpeg",
  ".mpg",
  ".ogm",
  ".ogv",
  ".rm",
  ".rmvb",
  ".ts",
  ".vob",
  ".webm",
  ".wmv",
  ".y4m",
  ".m4s",
]
export const AUDIO_EXTS = [
  ".aac",
  ".ac3",
  ".aiff",
  ".ape",
  ".au",
  ".cue",
  ".dsf",
  ".dts",
  ".flac",
  ".m4a",
  ".mid",
  ".midi",
  ".mka",
  ".mp3",
  ".mp4a",
  ".oga",
  ".ogg",
  ".opus",
  ".spx",
  ".tak",
  ".tta",
  ".wav",
  ".weba",
  ".wma",
  ".wv",
]
export const IMAGE_EXTS = [
  ".apng",
  ".avif",
  ".bmp",
  ".gif",
  ".j2k",
  ".jp2",
  ".jfif",
  ".jpeg",
  ".jpg",
  ".jxl",
  ".mj2",
  ".png",
  ".svg",
  ".tga",
  ".tif",
  ".tiff",
  ".webp",
]
export const SUBTITLE_EXTS = [
  ".aqt",
  ".ass",
  ".gsub",
  ".idx",
  ".jss",
  ".lrc",
  ".mks",
  ".pgs",
  ".pjs",
  ".psb",
  ".rt",
  ".sbv",
  ".slt",
  ".smi",
  ".sub",
  ".sup",
  ".srt",
  ".ssa",
  ".ssf",
  ".ttxt",
  ".usf",
  ".vt",
  ".vtt",
]
export const PRINTABLE_EXTS = [
  ".txt",
  ".pdf",
  ".doc",
  ".docx",
  ".xls",
  ".xlsx",
  ".ppt",
  ".pptx",
  ...IMAGE_EXTS,
]
