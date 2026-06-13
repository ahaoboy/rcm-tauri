import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Open" — launches the selected file/folder with its default handler.
 */
export function open(): MenuItem {
  return {
    key: "open",
    label: t("open"),
    icon: "📂",
    action: (props: InvokeProps) => {
      if (!props.files.length) return
      const target = props.files[0]
      return {
        exe: "cmd",
        args: ["/c", "start", "", target.path],
        cwd: props.cwd,
        window: "Hidden",
      }
    },
  }
}
