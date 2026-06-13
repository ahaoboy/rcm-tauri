import { t } from "../i18n"
import type { MenuItem } from "../types"

/**
 * Win11 "Pin to Taskbar" — pins the item to the taskbar.
 */
export function pinToTaskbar(): MenuItem {
  return {
    key: "pin-to-taskbar",
    label: t("pin.to.taskbar"),
    icon: "📌",
    action: () => ({
      exe: "powershell",
      args: ["-Command", ""],
      window: "Hidden",
    }),
  }
}
