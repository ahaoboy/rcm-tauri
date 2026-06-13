import { t } from "../i18n"
import { isExecutable } from "../tool"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Run as administrator" — for executables, scripts, and installers.
 */
export function runAsAdmin(): MenuItem {
  return {
    key: "run-as-admin",
    label: t("run.as.admin"),
    icon: "🛡️",
    admin: true,
    match: ({ files }: InvokeProps) => files.length > 0 && isExecutable(files[0].path),
    action: (props: InvokeProps) => ({
      cmd: props.files[0].path,
      admin: true,
    }),
  }
}
