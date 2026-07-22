import { ADD_TO_AUTORUN, REMOVE_FROM_AUTORUN } from "../consts"
import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Add to Startup" / "Remove from Startup" —
 * adds or removes a .exe file from the Windows startup (autorun) list
 * via the registry Run keys.
 *
 * The Rust backend provides `inAutorun` (entry name list) so the
 * frontend can check whether the selected .exe is already registered.
 *
 * Only matches single .exe file selections (not .lnk, not folders).
 */

/** True when the selected file is an .exe (single selection). */
function isExe(props: InvokeProps): boolean {
  if (props.files.length !== 1) return false
  return props.files[0].path.toLowerCase().endsWith(".exe")
}

/** Extract file stem from a Windows path (name without extension). */
function fileStem(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? path
  const dot = name.lastIndexOf(".")
  return dot > 0 ? name.slice(0, dot) : name
}

/** Extract the bare .exe path from a registry command string.
 *  Handles quoted paths, trailing \\0, and extra arguments. */
function exePath(command: string): string {
  // Strip trailing null bytes (registry REG_SZ sometimes includes them)
  let cmd = command.replace(/\0/g, "")
  // If the command starts with a quote, extract the quoted part
  if (cmd.startsWith('"')) {
    const end = cmd.indexOf('"', 1)
    if (end > 0) cmd = cmd.slice(1, end)
  } else {
    // Otherwise take the first space-delimited token
    cmd = cmd.split(/\s+/)[0]
  }
  return cmd
}

function isInAutorun(props: InvokeProps): boolean {
  const target = props.files[0].path.toLowerCase()
  return props.autorun.some((e) => exePath(e.command).toLowerCase() === target)
}

/**
 * "Add to Startup" — shown when the .exe is NOT already in autorun.
 * @param label Custom label, defaults to system i18n text.
 */
export function addToAutorun(label = t("add.to.startup")): MenuItem {
  return {
    key: "add-to-autorun",
    label,
    icon: "🚀",
    match: (props) => isExe(props) && !isInAutorun(props),
    action: (props) => ({
      cmd: ADD_TO_AUTORUN,
      args: [fileStem(props.files[0].path), props.files[0].path],
      window: "Hidden",
    }),
  }
}

/**
 * "Remove from Startup" — shown when the .exe IS already in autorun.
 * @param label Custom label, defaults to system i18n text.
 */
export function removeFromAutorun(label = t("remove.from.startup")): MenuItem {
  return {
    key: "remove-from-autorun",
    label,
    icon: "🚀",
    match: (props) => isExe(props) && isInAutorun(props),
    action: (props) => ({
      cmd: REMOVE_FROM_AUTORUN,
      args: [props.files[0].path],
      window: "Hidden",
    }),
  }
}
