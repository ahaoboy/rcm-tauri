import { t } from "../i18n"
import type { MenuItem } from "../types"

/**
 * Win11 "Share" — opens the Windows share dialog.
 */
export function share(): MenuItem {
  return {
    key: "share",
    label: t("share"),
    icon: "📤",
    action: () => ({
      cmd: "ms-settings:share",
      args: [],
    }),
  }
}
