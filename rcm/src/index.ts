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
export * as win11 from './configs/win11';
export * as custom from './configs/custom';
export { default as defaultMenu } from './default';
export { default as liteMenu } from './lite';

// ── System commands ────────────────────────────────────────────────
export * as SysCmd from './system-commands';

// ── Tools / Utilities ──────────────────────────────────────────────
export * from './tool';
