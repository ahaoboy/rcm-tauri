import { FORMAT, EJECT } from "../consts"
import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Drive" (磁盘) operations — shown when right-clicking a drive root
 * (e.g. C:\, D:\).
 *
 * Items:
 * - Format…  — opens the Windows Format dialog
 * - Eject     — ejects a removable drive
 */

/** True when the selected path is a drive root (e.g. "C:\\"). */
function isDriveRoot(path: string): boolean {
  return /^[A-Z]:\\$/i.test(path)
}

/** "Format…" — opens the Windows Format dialog via the @format system command. */
function format(): MenuItem {
  return {
    key: "format",
    label: `${t("format")}…`,
    icon: "💾",
    admin: true,
    action: (props: InvokeProps) => {
      const path = props.files[0]?.path
      if (!path) return
      return { cmd: FORMAT, args: [path], window: "Hidden" }
    },
  }
}

/** "Eject" — ejects a removable drive via the @eject system command. */
function eject(): MenuItem {
  return {
    key: "eject",
    label: t("eject"),
    icon: "⏏️",
    action: (props: InvokeProps) => {
      const path = props.files[0]?.path
      if (!path) return
      return { cmd: EJECT, args: [path], window: "Hidden" }
    },
  }
}

/**
 * Disk operations group.
 *
 * Matches when the user right-clicks a single item that is a drive root
 * (e.g. `C:\`, `D:\`).
 */
export function disk(): MenuItem {
  return {
    key: "disk",
    label: t("drive.tools"),
    match: ({ files }) => files.length === 1 && isDriveRoot(files[0].path),
    items: [format(), eject()],
  }
}
