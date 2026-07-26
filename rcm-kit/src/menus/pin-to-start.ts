import { PIN_TO_START, UNPIN_FROM_START } from "../consts"
import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Pin to Start" / "Unpin from Start" — pins or unpins a file
 * to/from the Start Menu by creating/removing a shortcut in
 * `%APPDATA%\Microsoft\Windows\Start Menu\Programs`.
 *
 * Only matches single `.exe` / `.lnk` file selections.
 * The Rust backend provides `pinnedToStart` (file-stem list) so the
 * frontend can check whether the current file is already pinned.
 */

/** True when the selected file is an .exe or .lnk (single selection). */
function isExeOrLnk(props: InvokeProps): boolean {
  if (props.files.length !== 1) return false
  const lower = props.files[0].path.toLowerCase()
  return lower.endsWith(".exe") || lower.endsWith(".lnk")
}

/** Extract file stem from a Windows path (name without extension). */
function fileStem(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? path
  const dot = name.lastIndexOf(".")
  return dot > 0 ? name.slice(0, dot) : name
}

function isPinned(props: InvokeProps): boolean {
  const stem = fileStem(props.files[0].path)
  return props.startmenu.some((lnk) => fileStem(lnk.path) === stem)
}

/**
 * "Pin to Start" — shown when the file is NOT already pinned.
 * @param label Custom label, defaults to system i18n text.
 */
export function pinToStart(label = t("pin.to.start")): MenuItem {
  return {
    key: "pin-to-start",
    label,
    icon: "📌",
    match: (props) => isExeOrLnk(props) && !isPinned(props),
    action: (props) => ({
      cmd: PIN_TO_START,
      args: [props.files[0].path],
      window: "Hidden",
    }),
  }
}

/**
 * "Unpin from Start" — shown when the file IS already pinned.
 * @param label Custom label, defaults to system i18n text.
 */
export function unpinFromStart(label = t("unpin.from.start")): MenuItem {
  return {
    key: "unpin-from-start",
    label,
    icon: "📌",
    match: (props) => isExeOrLnk(props) && isPinned(props),
    action: (props) => ({
      cmd: UNPIN_FROM_START,
      args: [props.files[0].path],
      window: "Hidden",
    }),
  }
}
