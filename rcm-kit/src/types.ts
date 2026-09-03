// ── Core types ──────────────────────────────────────────────────────

/** File info passed to menu handlers */
export interface FileInfo {
  path: string
  isDir: boolean
}

/** Well-known environment / context keys injected by the Rust backend. */
export interface Env {
  /** Running operating system — always "Windows". */
  OS: string
  /** User home directory (e.g. `C:\Users\you`). */
  HOME: string
  /** Desktop folder, follows OneDrive redirection. */
  DESKTOP: string
  /** Documents folder, follows OneDrive redirection. */
  DOCUMENTS: string
  /** Downloads folder. */
  DOWNLOADS: string
  /** Pictures folder. */
  PICTURES: string
  /** Music folder. */
  MUSIC: string
  /** Videos folder. */
  VIDEOS: string
  /** Arbitrary extra variables (future/unknown keys). */
  [key: string]: string
}

/** Snapshot of clipboard state at right-click time. */
export interface ClipboardInfo {
  has_text: boolean
  has_image: boolean
  has_files: boolean
}

/** A Windows startup / autorun entry (from autorun::Entry). */
export interface AutorunEntry {
  name: string
  command: string
  scope: "CurrentUser" | "LocalMachine"
}

/** An entry found in the Start Menu or on the Desktop. May be a `.lnk`
 * shortcut or a plain file. */
export interface Entry {
  /** Full path to the item (e.g. `a.lnk` for a shortcut, `a.exe` for a file). */
  path: string
  /** Command-line arguments when the item is a `.lnk` that carries them. */
  args: string | null
  /** Target the `.lnk` points to; `null` for plain files. */
  target: string | null
}

/** Props passed to match/action callbacks during menu evaluation */
export interface InvokeProps {
  files: FileInfo[]
  cwd: string
  env: Env
  admin: boolean
  /** Current i18n language (e.g. 'en', 'zh'). Falls back to 'en' if unsupported. */
  lang: string
  /** Snapshot of clipboard state at the time of the right-click. */
  clipboard?: ClipboardInfo
  /** Start Menu entries (check path/target against the selected file). */
  startmenu: Entry[]
  /** Paths currently in Quick Access (check with selected file paths). */
  quickAccess: string[]
  /** Startup / autorun entries (name → command pairs). */
  autorun: AutorunEntry[]
  /** Desktop entries — same shape as `startmenu`. */
  desktop: Entry[]
}

/** Window visibility for spawned processes. */
export type WindowMode = "Hidden" | "Visible" | "Minimized" | "Maximized"

/** Executable command descriptor */
export interface Command {
  cmd: string
  args?: string[]
  cwd?: string
  admin?: boolean
  window?: WindowMode
}

/** Callback signatures */
export type MatchFn = (props: InvokeProps) => boolean
export type ActionFn = (props: InvokeProps) => Command | undefined

/** A single item in the right-click menu */
export interface MenuItem {
  key?: string
  icon?: string
  label?: string
  disable?: boolean
  admin?: boolean
  items?: MenuItem[]
  match?: MatchFn
  action?: ActionFn
  /** Serializable command payload produced by action() — survives JSON.stringify. */
  command?: Command
}
