import { t } from "../i18n"
import { isPrintable } from "../tool"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Print" — sends the file to the default printer.
 */
export function print(): MenuItem {
  return {
    key: "print",
    label: t("print"),
    icon: "🖨️",
    match: ({ files }: InvokeProps) => files.length > 0 && isPrintable(files[0].path),
    action: (props: InvokeProps) => ({
      cmd: "print",
      args: [props.files[0].path],
    }),
  }
}
