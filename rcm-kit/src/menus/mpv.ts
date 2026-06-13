import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * "Open with mpv" menu item — plays media files with mpv player.
 */
export function mpv(labelKey = "open.with.mpv"): MenuItem {
  return {
    key: "mpv",
    label: t(labelKey),
    icon: "🎬",
    match: (props: InvokeProps) => {
      if (!props.files.length) return false
      const name = props.files[0].name.toLowerCase()
      return /\.(mkv|mp4|avi|mov|wmv|flv|webm|mp3|flac|wav|ogg|m4a)$/.test(name)
    },
    action: (props: InvokeProps) => ({
      exe: "mpv",
      args: props.files.map((f) => f.path),
      cwd: props.cwd,
      window: "Show",
    }),
  }
}
