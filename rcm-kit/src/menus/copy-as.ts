import { COPY_PATH, COPY_NAME, COPY_BASE64, COPY_TARGET } from "../consts"
import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"
import { copy } from "./clipboard"

const filesArg = (props: InvokeProps) => props.files.map((f) => f.path)

/** True when the selected file is a single .lnk shortcut. */
function isSingleLnk(props: InvokeProps): boolean {
  return props.files.length === 1 && props.files[0].path.toLowerCase().endsWith(".lnk")
}

export function copyAsPath(label = t("copy.as.path")): MenuItem {
  return {
    key: "copy-as-path",
    label,
    icon: "📋",
    action: (props: InvokeProps) => ({
      cmd: COPY_PATH,
      args: filesArg(props),
      cwd: props.cwd,
    }),
  }
}

/** Copy as name — file name(s) only */
export function copyAsName(label = t("copy.as.name")): MenuItem {
  return {
    key: "copy-as-name",
    label,
    icon: "🏷️",
    action: (props: InvokeProps) => ({
      cmd: COPY_NAME,
      args: filesArg(props),
      cwd: props.cwd,
    }),
  }
}

/** Copy as base64 — file content(s) encoded as base64 (single file only) */
export function copyAsBase64(label = t("copy.as.base64")): MenuItem {
  return {
    key: "copy-as-base64",
    label,
    icon: "🔐",
    match: (props: InvokeProps) => props.files.length === 1,
    action: (props: InvokeProps) => ({
      cmd: COPY_BASE64,
      args: filesArg(props),
      cwd: props.cwd,
    }),
  }
}

/** "Copy as target" — resolve .lnk and copy its target path (single .lnk only). */
export function copyAsTarget(label = t("copy.as.target")): MenuItem {
  return {
    key: "copy-as-target",
    label,
    icon: "🎯",
    match: isSingleLnk,
    action: (props: InvokeProps) => ({
      cmd: COPY_TARGET,
      args: filesArg(props),
    }),
  }
}

export function copyFile(): MenuItem {
  return copy("file")
}

/**
 * Win11 "Copy as" submenu — provides different copy formats.
 *
 * Sub-s:
 *   • Copy as path  — full path(s) with Linux-style '/' separators
 *   • Copy as name  — file name(s) only
 *   • Copy as base64 — file content(s) encoded as base64
 */
export function copyAs(label = t("copy.as")): MenuItem {
  return {
    key: "copy-as",
    label,
    icon: "📎",
    items: [copyAsPath(), copyAsName(), copyAsTarget(), copyAsBase64(), copyFile()],
  }
}
