import type { MenuItem, InvokeProps } from "../types"
import { t } from "../i18n"
import { SORT_BY } from "../consts"

const SORT_ITEMS: [string, string, string][] = [
  ["name", "sort.by.name", "📋"],
  ["date-modified", "sort.by.date.modified", "📅"],
  ["type", "sort.by.type", "📁"],
  ["size", "sort.by.size", "📊"],
  ["date-created", "sort.by.date.created", "📆"],
]

/**
 * Win11 "Sort by" submenu — change how files are sorted in the current folder.
 * Only shown when no files are selected (background right-click).
 */
export function sortBy(): MenuItem {
  return {
    key: "sort-by",
    label: t("sort.by"),
    icon: "🔤",
    match: (props: InvokeProps) => props.files.length === 0,
    items: SORT_ITEMS.map(([key, labelKey, icon]) => ({
      key: `sort-by-${key}`,
      label: t(labelKey),
      icon,
      action: (props: InvokeProps) => ({
        exe: SORT_BY,
        args: [key],
        cwd: props.cwd,
        window: "Hidden",
      }),
    })),
  }
}
