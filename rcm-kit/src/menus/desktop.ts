import { ADD_TO_DESKTOP, REMOVE_FROM_DESKTOP } from "../consts"
import { t } from "../i18n"
import { hasShortcut } from "../tool"
import type { MenuItem, InvokeProps } from "../types"

/**
 * "Add to Desktop" / "Remove from Desktop" — creates or removes a desktop
 * shortcut for the selected file/folder (Explorer's *Send to > Desktop
 * (create shortcut)* behaviour).
 *
 * The Rust backend provides `desktop` (`Entry[]` with path/args/
 * target) so the frontend can decide which action applies. Naming collisions
 * on add are handled by upath on the Rust side (`a.lnk` → `a(1).lnk`).
 */

/** True when a single file/folder is selected. */
function isSingleSelection(props: InvokeProps): boolean {
  return props.files.length === 1
}

/**
 * "Add to Desktop" — shown when the selected item has no desktop shortcut yet.
 * @param label Custom label, defaults to system i18n text.
 */
export function addToDesktop(label = t("add.to.desktop")): MenuItem {
  return {
    key: "add-to-desktop",
    label,
    icon: "🖥️",
    match: (props) => isSingleSelection(props) && !hasShortcut(props.desktop, props.files[0].path),
    action: (props) => ({
      cmd: ADD_TO_DESKTOP,
      args: [props.files[0].path],
      window: "Hidden",
    }),
  }
}

/**
 * "Remove from Desktop" — shown when the selected item already has a
 * matching desktop shortcut.
 * @param label Custom label, defaults to system i18n text.
 */
export function removeFromDesktop(label = t("remove.from.desktop")): MenuItem {
  return {
    key: "remove-from-desktop",
    label,
    icon: "🖥️",
    match: (props) => isSingleSelection(props) && hasShortcut(props.desktop, props.files[0].path),
    action: (props) => ({
      cmd: REMOVE_FROM_DESKTOP,
      args: [props.files[0].path],
      window: "Hidden",
    }),
  }
}
