import { NEW_FILE, NEW_FOLDER } from "../consts"
import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

const NEW_ITEMS: [string, string, string, string, string][] = [
  ["new-folder", "new.folder", "📁", NEW_FOLDER, ""],
  ["new-txt", "new.text.document", "📝", NEW_FILE, ".txt"],
  ["new-md", "new.md.document", "📘", NEW_FILE, ".md"],
  ["new-js", "new.js.file", "📜", NEW_FILE, ".js"],
  ["new-json", "new.json.file", "📋", NEW_FILE, ".json"],
  ["new-html", "new.html.file", "🌐", NEW_FILE, ".html"],
  ["new-css", "new.css.file", "🎨", NEW_FILE, ".css"],
]

/**
 * Win11 "New" submenu — create new files/folders via system commands.
 */
export function newMenu(): MenuItem {
  return {
    key: "new",
    label: t("new"),
    icon: "➕",
    items: NEW_ITEMS.map(([key, labelKey, icon, exe, ext]) => ({
      key,
      label: t(labelKey),
      icon,
      action: (props: InvokeProps) => ({
        exe,
        args: ext ? [ext] : [],
        cwd: props.cwd,
        window: "Hidden",
      }),
    })),
  }
}
