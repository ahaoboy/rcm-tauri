import { t } from "../i18n"
import type { MenuItem, InvokeProps } from "../types"

/**
 * Win11 "Send to" submenu.
 */
export function sendTo(): MenuItem {
  return {
    key: "send-to",
    label: t("send.to"),
    icon: "📨",
    items: [
      {
        key: "send-to-desktop",
        label: t("send.to.desktop"),
        icon: "🖥️",
        action: (props: InvokeProps) => ({
          cmd: "cmd",
          args: ["/c", "mklink", "%USERPROFILE%\\Desktop\\"],
          cwd: props.cwd,
          window: "Hidden",
        }),
      },
      {
        key: "send-to-documents",
        label: t("send.to.documents"),
        icon: "📄",
        action: (props: InvokeProps) => ({
          cmd: "cmd",
          args: ["/c", "copy", "/y"],
          cwd: props.cwd,
          window: "Hidden",
        }),
      },
      {
        key: "send-to-compressed",
        label: t("send.to.compressed"),
        icon: "🗜️",
        match: (props: InvokeProps) => props.files.length > 0,
        action: (props: InvokeProps) => ({
          cmd: "powershell",
          args: ["Compress-Archive", "-Path", props.files.map((f) => f.path).join(",")],
          cwd: props.cwd,
          window: "Hidden",
        }),
      },
    ],
  }
}
