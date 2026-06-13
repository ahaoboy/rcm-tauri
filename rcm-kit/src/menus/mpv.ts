import { t } from "../i18n"
import { isMedia } from "../tool"
import type { MenuItem, InvokeProps } from "../types"

/**
 * "Open with mpv" — plays video and audio files.
 */
export function mpv(labelKey = "open.with.mpv"): MenuItem {
  return {
    key: "mpv",
    label: t(labelKey),
    icon: "🎬",
    match: ({ files }: InvokeProps) => files.length > 0 && isMedia(files[0].path),
    action: (props: InvokeProps) => ({
      cmd: "mpv",
      args: props.files.map((f) => f.path),
      cwd: props.cwd,
    }),
  }
}
