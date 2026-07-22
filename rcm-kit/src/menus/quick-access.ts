import { ADD_TO_QUICK_ACCESS, REMOVE_FROM_QUICK_ACCESS } from "../consts"
import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Add to Quick Access" / "Remove from Quick Access" —
 * pins or unpins a file/folder to/from the Quick Access pane in
 * File Explorer.
 *
 * The Rust backend provides `inQuickAccess` (path list) so the
 * frontend can check whether the selected item is already there.
 */

function isInQA(props: InvokeProps): boolean {
  if (props.files.length === 0) return false
  return props.inQuickAccess.includes(props.files[0].path)
}

/**
 * "Add to Quick Access" — shown when NOT already in Quick Access.
 * @param label Custom label, defaults to system i18n text.
 */
export function addToQuickAccess(label = t("add.to.quick.access")): MenuItem {
  return {
    key: "add-to-quick-access",
    label,
    icon: "⭐",
    match: (props) => props.files.length >= 1 && props.files[0].isDir && !isInQA(props),
    action: (props: InvokeProps) => ({
      cmd: ADD_TO_QUICK_ACCESS,
      args: [props.files[0].path],
      window: "Hidden",
    }),
  }
}

/**
 * "Remove from Quick Access" — shown when already in Quick Access.
 * @param label Custom label, defaults to system i18n text.
 */
export function removeFromQuickAccess(label = t("remove.from.quick.access")): MenuItem {
  return {
    key: "remove-from-quick-access",
    label,
    icon: "⭐",
    match: (props) => props.files.length >= 1 && props.files[0].isDir && isInQA(props),
    action: (props: InvokeProps) => ({
      cmd: REMOVE_FROM_QUICK_ACCESS,
      args: [props.files[0].path],
      window: "Hidden",
    }),
  }
}
