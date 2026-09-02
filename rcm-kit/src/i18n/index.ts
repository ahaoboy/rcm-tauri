/**
 * Simple i18n module — no external dependencies.
 * Only affects `label` in menu items. Built-in en + zh, defaults to en.
 */

let locale = "en"

const messages: Record<string, Record<string, string>> = {}

/** Set the active locale (e.g. 'en', 'zh') */
export function setLocale(loc: string): void {
  locale = loc
}

/** Get the current locale */
export function getLocale(): string {
  return locale
}

/** Translate a key. Falls back to en, then to the key itself. */
export function t(key: string): string {
  return messages[locale]?.[key] ?? messages["en"]?.[key] ?? key
}

/** Register translation messages for a locale */
export function addMessages(loc: string, msgs: Record<string, string>): void {
  if (!messages[loc]) messages[loc] = {}
  Object.assign(messages[loc], msgs)
}

// ── Built-in English ────────────────────────────────────────────────
addMessages("en", {
  open: "Open",
  "open.with": "Open with",
  edit: "Edit",
  print: "Print",
  "run.as.admin": "Run as administrator",
  "pin.to.start": "Pin to Start",
  "unpin.from.start": "Unpin from Start",
  "pin.to.taskbar": "Pin to Taskbar",
  share: "Share",
  "send.to": "Send to",
  "send.to.desktop": "Desktop (create shortcut)",
  "send.to.mail": "Mail recipient",
  "send.to.documents": "Documents",
  "send.to.compressed": "Compressed (zipped) folder",
  cut: "Cut",
  copy: "Copy",
  paste: "Paste",
  "copy.as": "Copy as",
  "copy.as.path": "path",
  "copy.as.name": "name",
  "copy.as.target": "target",
  "copy.as.base64": "base64",
  "create.shortcut": "Create shortcut",
  delete: "Delete",
  trash: "Trash",
  code: "Code",
  rename: "Rename",
  properties: "Properties",
  new: "New",
  "new.folder": "Folder",
  "new.text.document": "Text File",
  "new.md.document": "MD File",
  "new.js.file": "JS File",
  "new.json.file": "JSON File",
  "new.html.file": "HTML File",
  "new.css.file": "CSS File",
  "open.in.terminal": "Terminal",
  refresh: "Refresh",
  "select.all": "Select all",
  "undo.delete": "Undo Delete",
  "undo.rename": "Undo Rename",
  "restore.prev.versions": "Restore previous versions",
  "pin.to.quick.access": "Pin to Quick access",
  "add.to.quick.access": "Add to Quick access",
  "remove.from.quick.access": "Remove from Quick access",
  "add.to.startup": "Add to Startup",
  "remove.from.startup": "Remove from Startup",
  "add.to.desktop": "Add to Desktop (create shortcut)",
  "remove.from.desktop": "Remove from Desktop",
  view: "View",
  "sort.by": "Sort by",
  "display.settings": "Display settings",
  personalize: "Personalize",
  "open.with.vscode": "Open with VS Code",
  "open.in.vscode": "Open in VS Code",
  "open.with.mpv": "Open with mpv",
  "open.in.wt": "Terminal",
  "ssh.connect": "SSH Connect",
  "extract.here": "Extract Here",
  "open.file.location": "Open file location",
  "group.by": "Group by",
  "group.by.name": "Name",
  "group.by.date.modified": "Date modified",
  "group.by.type": "Type",
  "group.by.size": "Size",
  "group.by.date.created": "Date created",
  "group.by.none": "None",
  "sort.by.name": "Name",
  "sort.by.date.modified": "Date modified",
  "sort.by.type": "Type",
  "sort.by.size": "Size",
  "sort.by.date.created": "Date created",
  "drive.tools": "Drive Tools",
  format: "Format",
  eject: "Eject",
})

// ── Built-in Chinese ────────────────────────────────────────────────
addMessages("zh", {
  open: "打开",
  "open.with": "打开方式",
  edit: "编辑",
  print: "打印",
  "run.as.admin": "以管理员身份运行",
  "pin.to.start": '固定到"开始"屏幕',
  "pin.to.taskbar": "固定到任务栏",
  share: "共享",
  "send.to": "发送到",
  "send.to.desktop": "桌面快捷方式",
  "send.to.mail": "邮件收件人",
  "send.to.documents": "文档",
  "send.to.compressed": "压缩(zipped)文件夹",
  cut: "剪切",
  copy: "复制",
  paste: "粘贴",
  "copy.as.path": "路径",
  "copy.as.name": "名称",
  "copy.as.target": "目标",
  "copy.as.base64": "base64",
  "create.shortcut": "创建快捷方式",
  delete: "删除",
  trash: "回收站",
  code: "Code",
  rename: "重命名",
  properties: "属性",
  new: "新建",
  "new.folder": "文件夹",
  "new.text.document": "文本文件",
  "new.md.document": "MD 文件",
  "new.js.file": "JS 文件",
  "new.json.file": "JSON 文件",
  "new.html.file": "HTML 文件",
  "new.css.file": "CSS 文件",
  "open.in.terminal": "在终端中打开",
  refresh: "刷新",
  "select.all": "全选",
  "undo.delete": "撤消删除",
  "undo.rename": "撤消重命名",
  "restore.prev.versions": "还原以前的版本",
  "pin.to.quick.access": "固定到快速访问",
  "add.to.startup": "添加到开机启动",
  "remove.from.startup": "取消开机启动",
  "add.to.desktop": "发送到桌面快捷方式",
  "remove.from.desktop": "从桌面移除快捷方式",
  view: "查看",
  "sort.by": "排序方式",
  "display.settings": "显示设置",
  personalize: "个性化",
  "open.with.vscode": "通过 VS Code 打开",
  "open.in.vscode": "在 VS Code 中打开",
  "open.with.mpv": "通过 mpv 打开",
  "open.in.wt": "在终端中打开",
  "ssh.connect": "SSH 连接",
  "extract.here": "解压到当前目录",
  "open.file.location": "打开文件所在位置",
  "group.by": "分组依据",
  "group.by.name": "名称",
  "group.by.date.modified": "修改日期",
  "group.by.type": "类型",
  "group.by.size": "大小",
  "group.by.date.created": "创建日期",
  "group.by.none": "无",
  "sort.by.name": "名称",
  "sort.by.date.modified": "修改日期",
  "sort.by.type": "类型",
  "sort.by.size": "大小",
  "sort.by.date.created": "创建日期",
  "drive.tools": "磁盘工具",
  format: "格式化",
  eject: "弹出",
})
