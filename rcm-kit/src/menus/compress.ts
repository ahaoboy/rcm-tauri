import { ZIP } from "../consts"
import { t } from "../i18n"
import { isZip } from "../tool"
import type { MenuItem, InvokeProps } from "../types"

/** Archive formats offered by the "compress" submenu (Rust `Fmt::guess`). */
export const ARCHIVE_FORMATS = [".zip", ".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst", ".7z"]

/**
 * "Compress" submenu — one item per archive format, for power users.
 *
 * If files are selected, they are archived into `<firstName><format>`.
 * On background click (no selection), the entire current directory is
 * archived into `<dirname><format>`.  Name collisions are handled by the
 * Rust backend. Casual users should prefer the plain "zip" action.
 */
export function compress(): MenuItem {
  return {
    key: "compress",
    label: t("compress"),
    icon: "🗜️",
    // Only hide when a single archive file is selected — pointless to re-archive it.
    match: ({ files }) => !(files.length === 1 && !files[0].isDir && isZip(files[0].path)),
    items: ARCHIVE_FORMATS.map((ext) => ({
      key: `compress-${ext}`,
      label: ext,
      action: (props: InvokeProps) => ({
        cmd: ZIP,
        args: [ext, ...props.files.map((f) => f.path)],
        cwd: props.cwd,
        window: "Hidden",
      }),
    })),
  }
}
