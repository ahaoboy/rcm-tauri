import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Properties" — opens file/folder properties dialog.
 * Falls back to the current directory when no file is selected (background right-click).
 */
export function properties(): MenuItem {
  return {
    key: "properties",
    label: t("properties"),
    icon: "ℹ️",
    action: (props: InvokeProps) => {
      // File selected → show properties of that file
      const target = props.files[0]
      const path = target ? target.path : props.cwd
      if (!path) return
      return { cmd: "@properties", args: [path] }
    },
  }
}
