// ── Core types ──────────────────────────────────────────────────────

/** File info passed to menu handlers */
export interface FileInfo {
  name: string;
  path: string;
  isDir: boolean;
}

/** Environment variables / context */
export type Env = Record<string, unknown>;

/** Snapshot of clipboard state at right-click time. */
export interface ClipboardInfo {
  has_text: boolean;
  has_image: boolean;
  has_files: boolean;
}

/** Props passed to match/action callbacks during menu evaluation */
export interface InvokeProps {
  files: FileInfo[];
  cwd: string;
  env: Env;
  admin: boolean;
  type: string;
  /** Current i18n language (e.g. 'en', 'zh'). Falls back to 'en' if unsupported. */
  lang: string;
  /** Snapshot of clipboard state at the time of the right-click. */
  clipboard?: ClipboardInfo;
}

/** Executable command descriptor */
export interface Command {
  exe: string;
  args?: string[];
  cwd?: string;
  admin?: boolean;
  window?: 'Hidden' | 'Show' | 'Visible' | 'Minimized' | 'Maximized';
}

/** Callback signatures */
export type MatchFn = (props: InvokeProps) => boolean;
export type ActionFn = (props: InvokeProps) => Command | undefined;

/** A single item in the right-click menu */
export interface MenuItem {
  key?: string;
  icon?: string;
  label?: string;
  disable?: boolean;
  admin?: boolean;
  window?: 'Hidden' | 'Show' | 'Visible' | 'Minimized' | 'Maximized';
  items?: MenuItem[];
  match?: MatchFn;
  action?: ActionFn;
  /** Serializable command payload produced by action() — survives JSON.stringify. */
  command?: Command;
}
