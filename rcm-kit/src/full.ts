import {
  addToAutorun,
  addToDesktop,
  addToQuickAccess,
  compress,
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
  removeFromAutorun,
  removeFromDesktop,
  removeFromQuickAccess,
  rename,
  runAsAdmin,
  selectAll,
  share,
  sortBy,
  ssh,
  terminal,
  trash,
  unpinFromStart,
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
        compress(),
        edit(),
        print(),
        runAsAdmin(),
        groupBy(),
        sortBy(),
      ],
    },
    {
      // Pin & send
      items: [
        pinToStart(),
        unpinFromStart(),
        pinToTaskbar(),
        addToQuickAccess(),
        removeFromQuickAccess(),
        addToAutorun(),
        removeFromAutorun(),
        addToDesktop(),
        removeFromDesktop(),
      ],
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
