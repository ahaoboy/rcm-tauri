import type { MenuItem, InvokeProps } from "../types"
import { t } from "../i18n"

/**
 * "Open with FSV" menu item.
 */
export function fsv(labelKey = "fsv"): MenuItem {
  return {
    key: "fsv",
    label: t(labelKey),
    icon: "💻",
    action: (props: InvokeProps) => {
      const targets = props.files.length ? props.files.map((f) => f.path) : ["."]
      return { exe: "fsv", args: targets, cwd: props.cwd, window: "Show" }
    },
  }
}
