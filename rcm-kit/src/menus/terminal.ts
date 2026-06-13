import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * "Open in Windows Terminal" menu item.
 *
 * Visible only on background click (no files selected) or when exactly
 * one folder is selected.  In the folder case, the terminal opens inside
 * that folder rather than the parent directory.
 */
export function terminal(labelKey = "open.in.wt"): MenuItem {
  return {
    key: "terminal",
    label: t(labelKey),
    icon: ">_",
    match: (props: InvokeProps) =>
      props.files.length === 0 || (props.files.length === 1 && props.files[0].isDir),
    action: (props: InvokeProps) => {
      // When a single folder is selected, open inside it; otherwise use cwd.
      const targetDir =
        props.files.length === 1 && props.files[0].isDir ? props.files[0].path : props.cwd
      return {
        cmd: "wt",
        args: ["-d", targetDir],
        cwd: targetDir,
        window: "Hidden",
      }
    },
  }
}
