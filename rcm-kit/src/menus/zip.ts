import { ZIP } from "../consts"
import { t } from "../i18n"
import { isZip } from "../tool"
import type { MenuItem } from "../types"

/**
 * "Zip" — the everyday quick-compress action (defaults to `.zip`).
 *
 * If files are selected, they are archived into `<firstName>.zip`.
 * On background click (no selection), the entire current directory is
 * archived into `<dirname>.zip`. Name collisions are handled by the Rust
 * backend. Power users who need other formats should use "compress".
 */
export function zip(): MenuItem {
  return {
    key: "zip",
    label: t("zip"),
    icon: "🗜️",
    // Only hide when a single archive file is selected — pointless to re-archive it.
    match: ({ files }) => !(files.length === 1 && !files[0].isDir && isZip(files[0].path)),
    action: (props) => ({
      cmd: ZIP,
      args: [".zip", ...props.files.map((f) => f.path)],
      cwd: props.cwd,
      window: "Hidden",
    }),
  }
}
