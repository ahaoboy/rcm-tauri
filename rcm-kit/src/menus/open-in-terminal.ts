import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Open in Terminal" — opens Windows Terminal at the selected folder or file location.
 */
export function openInTerminal(): MenuItem {
  return {
    key: "open-in-terminal",
    label: t("open.in.terminal"),
    icon: "🖥️",
    match: (props: InvokeProps) => props.files.length > 0,
    action: (props: InvokeProps) => ({
      cmd: "wt",
      args: ["-d", props.files[0].isDir ? props.files[0].path : props.cwd],
      cwd: props.cwd,
    }),
  }
}
