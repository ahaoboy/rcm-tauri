import { OPEN_WITH } from "../consts"
import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Open with" — opens the Windows "Open With → Choose another app"
 * dialog for the selected file via the native `@open-with` system command.
 *
 * Falls back to the current directory when no file is selected
 * (background right-click).
 */
export function openWith(): MenuItem {
  return {
    key: "open-with",
    label: t("open.with"),
    icon: "🔽",
    match: ({ files }) => files.length === 1,
    action: (props: InvokeProps) => {
      const target = props.files[0]
      const path = target ? target.path : props.cwd
      if (!path || props.files.length > 1) return
      return {
        cmd: OPEN_WITH,
        args: [path],
        window: "Hidden",
      }
    },
  }
}
