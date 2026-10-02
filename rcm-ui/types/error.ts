/**
 * Error payloads rendered by the error window.
 *
 * Mirrors `rcm_core::UiError` (`#[serde(tag = "kind", rename_all = "kebab-case")]`).
 * The JSON is carried in the URL hash — see `router.ts`.
 */
export type UiError =
  | { kind: "missing-programs"; programs: string[] }
  | { kind: "message"; message: string }
