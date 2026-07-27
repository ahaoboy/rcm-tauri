// ── Core types ──────────────────────────────────────────────────────

/** File info passed to menu handlers */
export interface FileInfo {
  path: string
  isDir: boolean
}

/** Environment variables / context */
export type Env = Record<string, string>

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

/** A Start Menu shortcut entry (from startmenu::Lnk). */
export interface StartmenuEntry {
  /** Full path to the .lnk file. */
  path: string
  /** Optional command-line arguments for the shortcut. */
  args: string | null
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
  /** Start Menu shortcuts (check lnk path stem against selected file). */
  startmenu: StartmenuEntry[]
  /** Paths currently in Quick Access (check with selected file paths). */
  quickAccess: string[]
  /** Startup / autorun entries (name → command pairs). */
  autorun: AutorunEntry[]
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
