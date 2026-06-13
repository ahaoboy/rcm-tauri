import { t } from "../i18n"
import { isText } from "../tool"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Edit" — opens text files in Notepad.
 */
export function edit(): MenuItem {
  return {
    key: "edit",
    label: t("edit"),
    icon: "✏️",
    match: ({ files }: InvokeProps) => files.length > 0 && isText(files[0].path),
    action: (props: InvokeProps) => ({
      cmd: "notepad",
      args: [props.files[0].path],
    }),
  }
}
