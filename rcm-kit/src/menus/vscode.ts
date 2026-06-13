import { t } from "../i18n"
import { isExecutable, isZip } from "../tool"
import type { MenuItem, InvokeProps } from "../types"

/**
 * "Open with VS Code" menu item.
 */
export function vscode(labelKey = "code"): MenuItem {
  return {
    key: "vscode",
    label: t(labelKey),
    icon: "💻",
    match: ({ files }) =>
      !files.every((f) => isZip(f.path) || isExecutable(f.path)) || files.length === 0,
    action: (props: InvokeProps) => {
      const targets = props.files.length ? props.files.map((f) => f.path) : ["."]
      return { exe: "code", args: targets, cwd: props.cwd, window: "Hidden" }
    },
  }
}
