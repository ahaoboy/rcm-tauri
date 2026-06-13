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
