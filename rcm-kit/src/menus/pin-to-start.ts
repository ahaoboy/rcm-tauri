import type { MenuItem } from "../types"
import { t } from "../i18n"

/**
 * Win11 "Pin to Start" — pins the item to the Start menu.
 */
export function pinToStart(): MenuItem {
  return {
    key: "pin-to-start",
    label: t("pin.to.start"),
    icon: "📌",
    action: () => ({
      exe: "powershell",
      args: ["-Command", ""],
      window: "Hidden",
    }),
  }
}
