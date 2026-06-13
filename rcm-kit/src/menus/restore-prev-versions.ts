import type { MenuItem } from "../types"
import { t } from "../i18n"

/**
 * Win11 "Restore previous versions" — opens the Previous Versions tab in file properties.
 */
export function restorePreviousVersions(): MenuItem {
  return {
    key: "restore-prev-versions",
    label: t("restore.prev.versions"),
    icon: "⏪",
    action: () => ({
      exe: "control",
      args: ["/name", "Microsoft.System"],
      window: "Show",
    }),
  }
}
