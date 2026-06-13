import type { MenuItem, InvokeProps } from "../types"
import { t } from "../i18n"
import { COPY_PATH, COPY_NAME, COPY_BASE64 } from "../consts"
import { copy } from "./clipboard"

const filesArg = (props: InvokeProps) => props.files.map((f) => f.path)

/** Copy as path — full path(s) with Linux-style '/' separators */
export function copyAsPath(): MenuItem {
  return {
    key: "copy-as-path",
    label: "path",
    icon: "📋",
    action: (props: InvokeProps) => ({
      exe: COPY_PATH,
      args: filesArg(props),
      cwd: props.cwd,
    }),
  }
}

/** Copy as name — file name(s) only */
export function copyAsName(): MenuItem {
  return {
    key: "copy-as-name",
    label: "name",
    icon: "🏷️",
    action: (props: InvokeProps) => ({
      exe: COPY_NAME,
      args: filesArg(props),
      cwd: props.cwd,
    }),
  }
}

/** Copy as base64 — file content(s) encoded as base64 (single file only) */
export function copyAsBase64(): MenuItem {
  return {
    key: "copy-as-base64",
    label: "base64",
    icon: "🔐",
    match: (props: InvokeProps) => props.files.length === 1,
    action: (props: InvokeProps) => ({
      exe: COPY_BASE64,
      args: filesArg(props),
      cwd: props.cwd,
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
export function copyAs(): MenuItem {
  return {
    key: "copy-as",
    label: t("copy.as"),
    icon: "📎",
    items: [copyAsPath(), copyAsName(), copyAsBase64(), copyFile()],
  }
}
