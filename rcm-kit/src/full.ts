import {
  newMenu,
  open,
  print,
  copy,
  copyAs,
  createShortcut,
  cut,
  disk,
  edit,
  groupBy,
  Menu,
  openFileLocation,
  openWith,
  paste,
  pinToStart,
  pinToTaskbar,
  properties,
  rename,
  runAsAdmin,
  selectAll,
  sendTo,
  share,
  sortBy,
  ssh,
  terminal,
  trash,
  unzip,
  vscode,
  zip,
} from "rcm-kit"

export default new Menu(
  // ── Main vertical menu groups ───────────────────────────────────
  [
    {
      // Primary actions
      items: [
        newMenu(),
        vscode(),
        open(),
        openWith(),
        disk(),
        terminal(),
        ssh(),
        unzip(),
        zip(),
        edit(),
        print(),
        runAsAdmin(),
        groupBy(),
        sortBy(),
      ],
    },
    {
      // Pin & send
      items: [pinToStart(), pinToTaskbar(), sendTo()],
    },
    {
      // Clipboard & file ops
      items: [
        cut(),
        copy(),
        copyAs(),
        paste(),
        createShortcut(),
        openFileLocation(),
        trash(),
        rename(),
        selectAll(),
      ],
    },
    {
      // Meta
      items: [properties()],
    },
  ],
  // ── Icon ribbon (top bar) ───────────────────────────────────────
  [cut(), copy(), paste(), rename(), share(), trash()],
)
