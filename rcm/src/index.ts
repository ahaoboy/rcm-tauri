/**
 * rcm — Right-Click Menu toolkit
 *
 * @module rcm
 */

// ── Core ───────────────────────────────────────────────────────────
export { Menu } from './menu';
export type {
  FileInfo,
  Env,
  InvokeProps,
  Command,
  MatchFn,
  ActionFn,
  MenuItem,
} from './types';

// ── i18n ───────────────────────────────────────────────────────────
export { setLocale, getLocale, t, addMessages } from './i18n';

// ── Configs ────────────────────────────────────────────────────────
export * from './menus';
export { default as defaultMenu } from './default';
export { default as liteMenu } from './lite';

// ── System commands ────────────────────────────────────────────────
export * from './consts';

// ── Tools / Utilities ──────────────────────────────────────────────
export * from './tool';
