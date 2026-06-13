import { t } from "../i18n"
import type { MenuItem } from "../types"

/**
 * Win11 "Restore previous versions" — opens the Previous Versions tab in file properties.
 */
export function restorePreviousVersions(): MenuItem {
  return {
    key: "restore-prev-versions",
    label: t("restore.prev.versions"),
    icon: "⏪",
    action: () => ({
      cmd: "control",
      args: ["/name", "Microsoft.System"],
    }),
  }
}
