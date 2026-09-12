// src/menu.ts
var Menu = class {
  constructor(groups = [], iconItems = []) {
    this.groups = groups;
    this.iconItems = iconItems;
  }
  groups;
  iconItems;
  type = "Menu";
  /** Evaluate the menu against a set of props: filter by match, run actions, recurse. */
  invoke(props) {
    return {
      iconItems: this.#resolve(this.iconItems, props),
      groups: this.#resolve(this.groups, props)
    };
  }
  #resolve(items, props) {
    const result = [];
    for (const { match, action, items: children, ...rest } of items) {
      if (match && !match(props)) continue;
      const resolved = { ...rest };
      const cmd = action?.(props);
      if (cmd) resolved.command = cmd;
      if (children) {
        const sub = this.#resolve(children, props);
        if (sub.length) resolved.items = sub;
        else if (!resolved.label && !resolved.action) continue;
      }
      result.push(resolved);
    }
    return result;
  }
};

// src/i18n/index.ts
var locale = "en";
var messages = {};
function setLocale(loc) {
  locale = loc;
}
function getLocale() {
  return locale;
}
function t(key) {
  return messages[locale]?.[key] ?? messages["en"]?.[key] ?? key;
}
function addMessages(loc, msgs) {
  if (!messages[loc]) messages[loc] = {};
  Object.assign(messages[loc], msgs);
}
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
  zip: "Zip",
  compress: "Compress to…",
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
  eject: "Eject"
});
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
  zip: "压缩",
  compress: "压缩为…",
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
  eject: "弹出"
});

// src/menus/open.ts
function open() {
  return {
    key: "open",
    label: t("open"),
    icon: "📂",
    action: (props) => {
      if (!props.files.length) return;
      const target = props.files[0];
      return {
        cmd: "cmd",
        args: ["/c", "start", "", target.path],
        cwd: props.cwd,
        window: "Hidden"
      };
    }
  };
}

// src/consts.ts
var UNZIP = "@unzip";
var ZIP = "@zip";
var RENAME = "@rename";
var NEW_FILE = "@new-file";
var NEW_FOLDER = "@new-folder";
var TRASH = "@trash";
var DELETE = "@delete";
var PROPERTIES = "@properties";
var COPY = "@copy";
var OPEN_WITH = "@open-with";
var COPY_PATH = "@copy-path";
var COPY_NAME = "@copy-name";
var COPY_BASE64 = "@copy-base64";
var COPY_TARGET = "@copy-target";
var OPEN_FILE_LOCATION = "@open-file-location";
var PASTE_FILES = "@paste-files";
var GROUP_BY = "@group-by";
var SORT_BY = "@sort-by";
var FORMAT = "@format";
var EJECT = "@eject";
var PIN_TO_START = "@pin-to-start";
var UNPIN_FROM_START = "@unpin-from-start";
var ADD_TO_QUICK_ACCESS = "@add-to-quick-access";
var REMOVE_FROM_QUICK_ACCESS = "@remove-from-quick-access";
var ADD_TO_AUTORUN = "@add-to-autorun";
var REMOVE_FROM_AUTORUN = "@remove-from-autorun";
var ADD_TO_DESKTOP = "@add-to-desktop";
var REMOVE_FROM_DESKTOP = "@remove-from-desktop";
var ALL = /* @__PURE__ */ new Set([
  UNZIP,
  ZIP,
  RENAME,
  NEW_FILE,
  NEW_FOLDER,
  TRASH,
  DELETE,
  PROPERTIES,
  COPY,
  OPEN_WITH,
  COPY_PATH,
  COPY_NAME,
  COPY_BASE64,
  COPY_TARGET,
  OPEN_FILE_LOCATION,
  PASTE_FILES,
  GROUP_BY,
  SORT_BY,
  FORMAT,
  EJECT,
  PIN_TO_START,
  UNPIN_FROM_START,
  ADD_TO_QUICK_ACCESS,
  REMOVE_FROM_QUICK_ACCESS,
  ADD_TO_AUTORUN,
  REMOVE_FROM_AUTORUN,
  ADD_TO_DESKTOP,
  REMOVE_FROM_DESKTOP
]);
var TEXT_EXTS = [
  ".txt",
  ".ini",
  ".cfg",
  ".log",
  ".md",
  ".xml",
  ".json",
  ".yml",
  ".yaml",
  ".toml",
  ".bat",
  ".cmd",
  ".ps1",
  ".reg",
  ".csv",
  ".tsv",
  ".tex",
  ".rst",
  ".org",
  // programming languages
  ".rs",
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".py",
  ".pyi",
  ".pyx",
  ".c",
  ".h",
  ".cpp",
  ".hpp",
  ".cc",
  ".hh",
  ".cxx",
  ".hxx",
  ".cs",
  ".vb",
  ".fs",
  ".fsx",
  ".go",
  ".java",
  ".kt",
  ".kts",
  ".scala",
  ".groovy",
  ".rb",
  ".rake",
  ".gemspec",
  ".php",
  ".phtml",
  ".swift",
  ".lua",
  ".r",
  ".R",
  ".pl",
  ".pm",
  ".sh",
  ".bash",
  ".zsh",
  ".fish",
  ".sql",
  ".psql",
  ".dart",
  ".elm",
  ".erl",
  ".hrl",
  ".ex",
  ".exs",
  ".hs",
  ".lhs",
  ".ml",
  ".mli",
  ".nim",
  ".zig",
  ".vue",
  ".svelte",
  ".html",
  ".htm",
  ".css",
  ".scss",
  ".sass",
  ".less",
  ".styl",
  ".graphql",
  ".gql",
  ".proto",
  ".diff",
  ".patch",
  ".lock"
];
var VIDEO_EXTS = [
  ".3g2",
  ".3gp",
  ".asf",
  ".avi",
  ".f4v",
  ".flv",
  ".h264",
  ".h265",
  ".m2ts",
  ".m4v",
  ".mkv",
  ".mov",
  ".mp4",
  ".mp4v",
  ".mpeg",
  ".mpg",
  ".ogm",
  ".ogv",
  ".rm",
  ".rmvb",
  ".ts",
  ".vob",
  ".webm",
  ".wmv",
  ".y4m",
  ".m4s"
];
var AUDIO_EXTS = [
  ".aac",
  ".ac3",
  ".aiff",
  ".ape",
  ".au",
  ".cue",
  ".dsf",
  ".dts",
  ".flac",
  ".m4a",
  ".mid",
  ".midi",
  ".mka",
  ".mp3",
  ".mp4a",
  ".oga",
  ".ogg",
  ".opus",
  ".spx",
  ".tak",
  ".tta",
  ".wav",
  ".weba",
  ".wma",
  ".wv"
];
var IMAGE_EXTS = [
  ".apng",
  ".avif",
  ".bmp",
  ".gif",
  ".j2k",
  ".jp2",
  ".jfif",
  ".jpeg",
  ".jpg",
  ".jxl",
  ".mj2",
  ".png",
  ".svg",
  ".tga",
  ".tif",
  ".tiff",
  ".webp"
];
var SUBTITLE_EXTS = [
  ".aqt",
  ".ass",
  ".gsub",
  ".idx",
  ".jss",
  ".lrc",
  ".mks",
  ".pgs",
  ".pjs",
  ".psb",
  ".rt",
  ".sbv",
  ".slt",
  ".smi",
  ".sub",
  ".sup",
  ".srt",
  ".ssa",
  ".ssf",
  ".ttxt",
  ".usf",
  ".vt",
  ".vtt"
];
var PRINTABLE_EXTS = [
  ".txt",
  ".pdf",
  ".doc",
  ".docx",
  ".xls",
  ".xlsx",
  ".ppt",
  ".pptx",
  ...IMAGE_EXTS
];

// src/menus/open-with.ts
function openWith() {
  return {
    key: "open-with",
    label: t("open.with"),
    icon: "🔽",
    match: ({ files }) => files.length === 1,
    action: (props) => {
      const target = props.files[0];
      const path2 = target ? target.path : props.cwd;
      if (!path2 || props.files.length > 1) return;
      return {
        cmd: OPEN_WITH,
        args: [path2],
        window: "Hidden"
      };
    }
  };
}

// src/tool.ts
var ARCHIVE_EXTS = [
  // TarGz
  ".tar.gz",
  ".tgz",
  // TarXz
  ".tar.xz",
  ".txz",
  // TarBz
  ".tar.bz2",
  ".tbz2",
  ".tbz",
  // TarZstd
  ".tar.zst",
  ".tzst",
  ".tzstd",
  // Tar (plain)
  ".tar",
  // Zip
  ".zip",
  // 7z
  ".7z"
];
function isZip(path2) {
  const lower = path2.toLowerCase();
  return ARCHIVE_EXTS.some((ext) => lower.endsWith(ext));
}
var EXECUTABLE_EXTS = [".exe", ".com", ".scr", ".pif", ".msi", ".msix", ".appx"];
function isExecutable(path2) {
  const lower = path2.toLowerCase();
  return EXECUTABLE_EXTS.some((ext) => lower.endsWith(ext));
}
function basename(path2) {
  return path2.split(/[\\/]/).pop() ?? path2;
}
function hasExt(path2, ...exts) {
  const lower = path2.toLowerCase();
  return exts.some((ext) => lower.endsWith(ext.toLowerCase()));
}
var isText = (path2) => hasExt(path2, ...TEXT_EXTS);
var isVideo = (path2) => hasExt(path2, ...VIDEO_EXTS);
var isAudio = (path2) => hasExt(path2, ...AUDIO_EXTS);
var isImage = (path2) => hasExt(path2, ...IMAGE_EXTS);
var isSubtitle = (path2) => hasExt(path2, ...SUBTITLE_EXTS);
var isMedia = (path2) => isVideo(path2) || isAudio(path2);
var isPrintable = (path2) => hasExt(path2, ...PRINTABLE_EXTS);
function fileStem(path2) {
  const name = basename(path2);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(0, dot) : name;
}
function hasShortcut(list, path2) {
  const norm = path2.toLowerCase();
  const stem = fileStem(path2);
  return list.some(
    (lnk) => lnk.target !== null && lnk.target.toLowerCase() === norm || fileStem(lnk.path) === stem
  );
}

// src/menus/edit.ts
function edit() {
  return {
    key: "edit",
    label: t("edit"),
    icon: "✏️",
    match: ({ files }) => files.length > 0 && isText(files[0].path),
    action: (props) => ({
      cmd: "notepad",
      args: [props.files[0].path]
    })
  };
}

// src/menus/print.ts
function print2() {
  return {
    key: "print",
    label: t("print"),
    icon: "🖨️",
    match: ({ files }) => files.length > 0 && isPrintable(files[0].path),
    action: (props) => ({
      cmd: "print",
      args: [props.files[0].path]
    })
  };
}

// src/menus/run-as-admin.ts
function runAsAdmin() {
  return {
    key: "run-as-admin",
    label: t("run.as.admin"),
    icon: "🛡️",
    admin: true,
    match: ({ files }) => files.length > 0 && isExecutable(files[0].path),
    action: (props) => ({
      cmd: props.files[0].path,
      admin: true
    })
  };
}

// src/menus/share.ts
function share() {
  return {
    key: "share",
    label: t("share"),
    icon: "📤",
    action: () => ({
      cmd: "ms-settings:share",
      args: []
    })
  };
}

// src/menus/pin-to-start.ts
function isExeOrLnk(props) {
  if (props.files.length !== 1) return false;
  const lower = props.files[0].path.toLowerCase();
  return lower.endsWith(".exe") || lower.endsWith(".lnk");
}
function isPinned(props) {
  return hasShortcut(props.startmenu, props.files[0].path);
}
function pinToStart(label = t("pin.to.start")) {
  return {
    key: "pin-to-start",
    label,
    icon: "📌",
    match: (props) => isExeOrLnk(props) && !isPinned(props),
    action: (props) => ({
      cmd: PIN_TO_START,
      args: [props.files[0].path],
      window: "Hidden"
    })
  };
}
function unpinFromStart(label = t("unpin.from.start")) {
  return {
    key: "unpin-from-start",
    label,
    icon: "📌",
    match: (props) => isExeOrLnk(props) && isPinned(props),
    action: (props) => ({
      cmd: UNPIN_FROM_START,
      args: [props.files[0].path],
      window: "Hidden"
    })
  };
}

// src/menus/pin-to-taskbar.ts
function pinToTaskbar() {
  return {
    key: "pin-to-taskbar",
    label: t("pin.to.taskbar"),
    icon: "📌",
    action: () => ({
      cmd: "powershell",
      args: ["-Command", ""],
      window: "Hidden"
    })
  };
}

// src/menus/clipboard.ts
function cut() {
  return { key: "cut", label: t("cut"), icon: "✂️" };
}
function copy(label = t("copy")) {
  return {
    key: "copy",
    label,
    icon: "📋",
    match: (props) => props.files.length > 0,
    action: (props) => ({
      cmd: "@copy",
      args: props.files.map((f) => f.path)
    })
  };
}
function paste() {
  return {
    key: "paste",
    label: t("paste"),
    icon: "📄",
    match: (props) => props.files.length === 0 && props.clipboard?.has_files === true,
    action: (props) => ({
      cmd: "@paste-files",
      args: [],
      cwd: props.cwd
    })
  };
}
function rename() {
  return { key: "rename", label: t("rename"), icon: "✏️" };
}
function trash() {
  return {
    key: "trash",
    label: t("trash"),
    icon: "🗑️",
    match: (props) => props.files.length > 0,
    action: (props) => ({
      cmd: "@trash",
      args: props.files.map((f) => f.path)
    })
  };
}
function selectAll() {
  return { key: "select-all", label: t("select.all"), icon: "🔲" };
}
function refresh() {
  return { key: "refresh", label: t("refresh"), icon: "🔄" };
}

// src/menus/copy-as.ts
var filesArg = (props) => props.files.map((f) => f.path);
function isSingleLnk(props) {
  return props.files.length === 1 && props.files[0].path.toLowerCase().endsWith(".lnk");
}
function copyAsPath(label = t("copy.as.path")) {
  return {
    key: "copy-as-path",
    label,
    icon: "📋",
    action: (props) => ({
      cmd: COPY_PATH,
      args: filesArg(props),
      cwd: props.cwd
    })
  };
}
function copyAsName(label = t("copy.as.name")) {
  return {
    key: "copy-as-name",
    label,
    icon: "🏷️",
    action: (props) => ({
      cmd: COPY_NAME,
      args: filesArg(props),
      cwd: props.cwd
    })
  };
}
function copyAsBase64(label = t("copy.as.base64")) {
  return {
    key: "copy-as-base64",
    label,
    icon: "🔐",
    match: (props) => props.files.length === 1,
    action: (props) => ({
      cmd: COPY_BASE64,
      args: filesArg(props),
      cwd: props.cwd
    })
  };
}
function copyAsTarget(label = t("copy.as.target")) {
  return {
    key: "copy-as-target",
    label,
    icon: "🎯",
    match: isSingleLnk,
    action: (props) => ({
      cmd: COPY_TARGET,
      args: filesArg(props)
    })
  };
}
function copyFile() {
  return copy("file");
}
function copyAs(label = t("copy.as")) {
  return {
    key: "copy-as",
    label,
    icon: "📎",
    items: [copyAsPath(), copyAsName(), copyAsTarget(), copyAsBase64(), copyFile()]
  };
}

// src/menus/create-shortcut.ts
function createShortcut() {
  return {
    key: "create-shortcut",
    label: t("create.shortcut"),
    icon: "🔗",
    match: (props) => props.files.length > 0,
    action: (props) => {
      const file = props.files[0];
      return {
        cmd: "powershell",
        args: [
          "-Command",
          `$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('${file.path}.lnk'); $s.TargetPath = '${file.path}'; $s.Save()`
        ],
        cwd: props.cwd,
        window: "Hidden"
      };
    }
  };
}

// src/menus/properties.ts
function properties() {
  return {
    key: "properties",
    label: t("properties"),
    icon: "ℹ️",
    action: (props) => {
      const target = props.files[0];
      const path2 = target ? target.path : props.cwd;
      if (!path2) return;
      return { cmd: "@properties", args: [path2] };
    }
  };
}

// src/menus/new-menu.ts
var NEW_ITEMS = [
  ["new-folder", "new.folder", "📁", NEW_FOLDER, ""],
  ["new-txt", "new.text.document", "📝", NEW_FILE, ".txt"],
  ["new-md", "new.md.document", "📘", NEW_FILE, ".md"],
  ["new-js", "new.js.file", "📜", NEW_FILE, ".js"],
  ["new-json", "new.json.file", "📋", NEW_FILE, ".json"],
  ["new-html", "new.html.file", "🌐", NEW_FILE, ".html"],
  ["new-css", "new.css.file", "🎨", NEW_FILE, ".css"]
];
function newMenu() {
  return {
    key: "new",
    label: t("new"),
    icon: "➕",
    items: NEW_ITEMS.map(([key, labelKey, icon, cmd, ext]) => ({
      key,
      label: t(labelKey),
      icon,
      action: (props) => ({
        cmd,
        args: ext ? [ext] : [],
        cwd: props.cwd,
        window: "Hidden"
      })
    }))
  };
}

// src/menus/group-by.ts
var GROUP_ITEMS = [
  ["name", "group.by.name", "📋"],
  ["date-modified", "group.by.date.modified", "📅"],
  ["type", "group.by.type", "📁"],
  ["size", "group.by.size", "📊"],
  ["date-created", "group.by.date.created", "📆"],
  ["none", "group.by.none", "🚫"]
];
function groupBy() {
  return {
    key: "group-by",
    label: t("group.by"),
    icon: "📑",
    match: (props) => props.files.length === 0,
    items: GROUP_ITEMS.map(([key, labelKey, icon]) => ({
      key: `group-by-${key}`,
      label: t(labelKey),
      icon,
      action: (props) => ({
        cmd: GROUP_BY,
        args: [key],
        cwd: props.cwd,
        window: "Hidden"
      })
    }))
  };
}

// src/menus/sort-by.ts
var SORT_ITEMS = [
  ["name", "sort.by.name", "📋"],
  ["date-modified", "sort.by.date.modified", "📅"],
  ["type", "sort.by.type", "📁"],
  ["size", "sort.by.size", "📊"],
  ["date-created", "sort.by.date.created", "📆"]
];
function sortBy() {
  return {
    key: "sort-by",
    label: t("sort.by"),
    icon: "🔤",
    match: (props) => props.files.length === 0,
    items: SORT_ITEMS.map(([key, labelKey, icon]) => ({
      key: `sort-by-${key}`,
      label: t(labelKey),
      icon,
      action: (props) => ({
        cmd: SORT_BY,
        args: [key],
        cwd: props.cwd,
        window: "Hidden"
      })
    }))
  };
}

// src/menus/open-in-terminal.ts
function openInTerminal() {
  return {
    key: "open-in-terminal",
    label: t("open.in.terminal"),
    icon: "🖥️",
    match: (props) => props.files.length > 0,
    action: (props) => ({
      cmd: "wt",
      args: ["-d", props.files[0].isDir ? props.files[0].path : props.cwd],
      cwd: props.cwd
    })
  };
}

// src/menus/restore-prev-versions.ts
function restorePreviousVersions() {
  return {
    key: "restore-prev-versions",
    label: t("restore.prev.versions"),
    icon: "⏪",
    action: () => ({
      cmd: "control",
      args: ["/name", "Microsoft.System"]
    })
  };
}

// src/menus/vscode.ts
function vscode(labelKey = "code") {
  return {
    key: "vscode",
    label: t(labelKey),
    icon: "💻",
    match: ({ files }) => !files.every((f) => isZip(f.path) || isExecutable(f.path)) || files.length === 0,
    action: (props) => {
      const targets = props.files.length ? props.files.map((f) => f.path) : ["."];
      return { cmd: "code", args: targets, cwd: props.cwd, window: "Hidden" };
    }
  };
}

// src/menus/ssh.ts
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
function readWtSshProfiles() {
  try {
    const home = os.homedir();
    const localAppData = path.join(home, "AppData", "Local");
    print("[ssh] localAppData: " + localAppData);
    const wtBase = path.join(localAppData, "Packages");
    let settingsPath = "";
    try {
      const dirs = fs.readdirSync(wtBase);
      const wtDir = dirs.find((d) => d.startsWith("Microsoft.WindowsTerminal"));
      if (wtDir) {
        settingsPath = path.join(wtBase, wtDir, "LocalState", "settings.json");
      }
    } catch {
      print("[ssh] failed to scan WT packages");
      return [];
    }
    if (!settingsPath) {
      print("[ssh] no settings.json found");
      return [];
    }
    const raw = fs.readFileSync(settingsPath, "utf8");
    const cfg = JSON.parse(raw);
    const profiles = cfg?.profiles?.list ?? [];
    return profiles.filter((p) => p.name && p.commandline && p.commandline.includes("ssh"));
  } catch (e) {
    print("[ssh] error: " + String(e.message || e));
    return [];
  }
}
function ssh() {
  try {
    const profiles = readWtSshProfiles();
    print("[ssh] found " + profiles.length + " profiles");
    if (profiles.length === 0) {
      return { key: "ssh", label: "SSH", icon: "🖥️" };
    }
    return {
      key: "ssh",
      label: "SSH",
      icon: "🖥️",
      match: ({ files }) => files.length === 0,
      // only show on background click
      items: profiles.map((p) => ({
        key: `ssh-${p.name}`,
        label: p.name,
        icon: "🖥️",
        action: (_props) => ({
          cmd: "wt",
          args: ["-w", "0", "new-tab", "--profile", p.name],
          window: "Hidden"
        })
      }))
    };
  } catch (e) {
    print("[ssh] fatal: " + String(e.message || e));
    return { key: "ssh", label: "SSH", icon: "🖥️" };
  }
}

// src/menus/terminal.ts
function terminal(labelKey = "open.in.wt") {
  return {
    key: "terminal",
    label: t(labelKey),
    icon: ">_",
    match: (props) => props.files.length === 0 || props.files.length === 1 && props.files[0].isDir,
    action: (props) => {
      const targetDir = props.files.length === 1 && props.files[0].isDir ? props.files[0].path : props.cwd;
      return {
        cmd: "wt",
        args: ["-d", targetDir],
        cwd: targetDir,
        window: "Hidden"
      };
    }
  };
}

// src/menus/zip.ts
function zip() {
  return {
    key: "zip",
    label: t("zip"),
    icon: "🗜️",
    // Only hide when a single archive file is selected — pointless to re-archive it.
    match: ({ files }) => !(files.length === 1 && !files[0].isDir && isZip(files[0].path)),
    action: (props) => ({
      cmd: ZIP,
      args: [".zip", ...props.files.map((f) => f.path)],
      cwd: props.cwd,
      window: "Hidden"
    })
  };
}

// src/menus/compress.ts
var ARCHIVE_FORMATS = [".zip", ".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst", ".7z"];
function compress() {
  return {
    key: "compress",
    label: t("compress"),
    icon: "🗜️",
    // Only hide when a single archive file is selected — pointless to re-archive it.
    match: ({ files }) => !(files.length === 1 && !files[0].isDir && isZip(files[0].path)),
    items: ARCHIVE_FORMATS.map((ext) => ({
      key: `compress-${ext}`,
      label: ext,
      action: (props) => ({
        cmd: ZIP,
        args: [ext, ...props.files.map((f) => f.path)],
        cwd: props.cwd,
        window: "Hidden"
      })
    }))
  };
}

// src/menus/unzip.ts
function unzip() {
  return {
    key: "unzip",
    label: "unzip",
    icon: "📦",
    match: (props) => props.files.length > 0 && props.files.every((f) => isZip(f.path)),
    action: (props) => ({
      cmd: UNZIP,
      args: props.files.map((f) => f.path),
      cwd: props.cwd,
      window: "Hidden"
    })
  };
}

// src/menus/open-file-location.ts
function openFileLocation() {
  return {
    key: "open-file-location",
    label: t("open.file.location"),
    icon: "📂",
    match: (props) => {
      const file = props.files[0];
      return props.files.length === 1 && !file.isDir && file.path.endsWith(".lnk");
    },
    action: (props) => ({
      cmd: OPEN_FILE_LOCATION,
      args: [props.files[0].path],
      cwd: props.cwd,
      window: "Hidden"
    })
  };
}

// src/menus/disk.ts
function isDriveRoot(path2) {
  return /^[A-Z]:\\$/i.test(path2);
}
function format() {
  return {
    key: "format",
    label: `${t("format")}…`,
    icon: "💾",
    admin: true,
    action: (props) => {
      const path2 = props.files[0]?.path;
      if (!path2) return;
      return { cmd: FORMAT, args: [path2], window: "Hidden" };
    }
  };
}
function eject() {
  return {
    key: "eject",
    label: t("eject"),
    icon: "⏏️",
    action: (props) => {
      const path2 = props.files[0]?.path;
      if (!path2) return;
      return { cmd: EJECT, args: [path2], window: "Hidden" };
    }
  };
}
function disk() {
  return {
    key: "disk",
    label: t("drive.tools"),
    match: ({ files }) => files.length === 1 && isDriveRoot(files[0].path),
    items: [format(), eject()]
  };
}

// src/menus/quick-access.ts
function isInQA(props) {
  if (props.files.length === 0) return false;
  return props.quickAccess.includes(props.files[0].path);
}
function addToQuickAccess(label = t("add.to.quick.access")) {
  return {
    key: "add-to-quick-access",
    label,
    icon: "⭐",
    match: (props) => props.files.length >= 1 && props.files[0].isDir && !isInQA(props),
    action: (props) => ({
      cmd: ADD_TO_QUICK_ACCESS,
      args: [props.files[0].path],
      window: "Hidden"
    })
  };
}
function removeFromQuickAccess(label = t("remove.from.quick.access")) {
  return {
    key: "remove-from-quick-access",
    label,
    icon: "⭐",
    match: (props) => props.files.length >= 1 && props.files[0].isDir && isInQA(props),
    action: (props) => ({
      cmd: REMOVE_FROM_QUICK_ACCESS,
      args: [props.files[0].path],
      window: "Hidden"
    })
  };
}

// src/menus/autorun.ts
function isExe(props) {
  if (props.files.length !== 1) return false;
  return props.files[0].path.toLowerCase().endsWith(".exe");
}
function fileStem2(path2) {
  const name = path2.split(/[\\/]/).pop() ?? path2;
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(0, dot) : name;
}
function exePath(cmd) {
  if (cmd.startsWith('"')) {
    const end = cmd.indexOf('"', 1);
    if (end > 0) cmd = cmd.slice(1, end);
  } else {
    cmd = cmd.split(/\s+/)[0];
  }
  return cmd;
}
function isInAutorun(props) {
  const target = props.files[0].path.toLowerCase();
  return props.autorun.some((e) => exePath(e.command).toLowerCase() === target);
}
function addToAutorun(label = t("add.to.startup")) {
  return {
    key: "add-to-autorun",
    label,
    icon: "🚀",
    match: (props) => isExe(props) && !isInAutorun(props),
    action: (props) => ({
      cmd: ADD_TO_AUTORUN,
      args: [fileStem2(props.files[0].path), props.files[0].path],
      window: "Hidden"
    })
  };
}
function removeFromAutorun(label = t("remove.from.startup")) {
  return {
    key: "remove-from-autorun",
    label,
    icon: "🚀",
    match: (props) => isExe(props) && isInAutorun(props),
    action: (props) => ({
      cmd: REMOVE_FROM_AUTORUN,
      args: [props.files[0].path],
      window: "Hidden"
    })
  };
}

// src/menus/desktop.ts
function isSingleSelection(props) {
  return props.files.length === 1;
}
function addToDesktop(label = t("add.to.desktop")) {
  return {
    key: "add-to-desktop",
    label,
    icon: "🖥️",
    match: (props) => isSingleSelection(props) && !hasShortcut(props.desktop, props.files[0].path),
    action: (props) => ({
      cmd: ADD_TO_DESKTOP,
      args: [props.files[0].path],
      window: "Hidden"
    })
  };
}
function removeFromDesktop(label = t("remove.from.desktop")) {
  return {
    key: "remove-from-desktop",
    label,
    icon: "🖥️",
    match: (props) => isSingleSelection(props) && hasShortcut(props.desktop, props.files[0].path),
    action: (props) => ({
      cmd: REMOVE_FROM_DESKTOP,
      args: [props.files[0].path],
      window: "Hidden"
    })
  };
}
export {
  ADD_TO_AUTORUN,
  ADD_TO_DESKTOP,
  ADD_TO_QUICK_ACCESS,
  ALL,
  ARCHIVE_EXTS,
  ARCHIVE_FORMATS,
  AUDIO_EXTS,
  COPY,
  COPY_BASE64,
  COPY_NAME,
  COPY_PATH,
  COPY_TARGET,
  DELETE,
  EJECT,
  EXECUTABLE_EXTS,
  FORMAT,
  GROUP_BY,
  IMAGE_EXTS,
  Menu,
  NEW_FILE,
  NEW_FOLDER,
  OPEN_FILE_LOCATION,
  OPEN_WITH,
  PASTE_FILES,
  PIN_TO_START,
  PRINTABLE_EXTS,
  PROPERTIES,
  REMOVE_FROM_AUTORUN,
  REMOVE_FROM_DESKTOP,
  REMOVE_FROM_QUICK_ACCESS,
  RENAME,
  SORT_BY,
  SUBTITLE_EXTS,
  TEXT_EXTS,
  TRASH,
  UNPIN_FROM_START,
  UNZIP,
  VIDEO_EXTS,
  ZIP,
  addMessages,
  addToAutorun,
  addToDesktop,
  addToQuickAccess,
  basename,
  compress,
  copy,
  copyAs,
  copyAsBase64,
  copyAsName,
  copyAsPath,
  copyAsTarget,
  copyFile,
  createShortcut,
  cut,
  disk,
  edit,
  fileStem,
  getLocale,
  groupBy,
  hasExt,
  hasShortcut,
  isAudio,
  isExecutable,
  isImage,
  isMedia,
  isPrintable,
  isSubtitle,
  isText,
  isVideo,
  isZip,
  newMenu,
  open,
  openFileLocation,
  openInTerminal,
  openWith,
  paste,
  pinToStart,
  pinToTaskbar,
  print2 as print,
  properties,
  refresh,
  removeFromAutorun,
  removeFromDesktop,
  removeFromQuickAccess,
  rename,
  restorePreviousVersions,
  runAsAdmin,
  selectAll,
  setLocale,
  share,
  sortBy,
  ssh,
  t,
  terminal,
  trash,
  unpinFromStart,
  unzip,
  vscode,
  zip
};
