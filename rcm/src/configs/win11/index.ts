/**
 * Win11 right-click menu item factories.
 * Each module exports a function returning a MenuItem.
 */
export { open } from './open';
export { openWith } from './open-with';
export { edit } from './edit';
export { print } from './print';
export { runAsAdmin } from './run-as-admin';
export { share } from './share';
export { pinToStart } from './pin-to-start';
export { pinToTaskbar } from './pin-to-taskbar';
export { sendTo } from './send-to';
export { cut, copy, paste, rename, trash, selectAll, refresh } from './clipboard';
export * from './copy-as';
export { createShortcut } from './create-shortcut';
export { properties } from './properties';
export { newMenu } from './new-menu';
export { openInTerminal } from './open-in-terminal';
export { restorePreviousVersions } from './restore-prev-versions';
