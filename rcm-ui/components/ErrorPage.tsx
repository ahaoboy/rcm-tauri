/**
 * ErrorPage — the single window for every error/diagnostic.
 *
 * Route: `#error/<url-encoded-json>`
 *
 * The hash carries a [`UiError`] (mirroring `rcm_core::UiError`) whose `kind`
 * selects the layout. Anything that fails to parse falls back to a plain
 * message, so old links and hand-typed errors still display.
 */

import { getCurrentWindow } from "@tauri-apps/api/window"
import React from "react"

import { BodyReset } from "./BodyReset"

/**
 * JSON payload emitted by the Rust side.
 * Mirrors `rcm_core::UiError` (`#[serde(tag = "kind")]`).
 */
export type UiError =
  | { kind: "already-running" }
  | { kind: "missing-programs"; programs: string[] }
  | { kind: "message"; message: string }

/** Decode without throwing on malformed input. */
function safeDecode(value: string): string {
  try {
    return decodeURIComponent(value)
  } catch {
    return value
  }
}

/** Parse the `#error/…` hash into a [`UiError`]. */
function parsePayload(hash: string): UiError {
  const raw = hash.replace(/^#error\/?/, "")
  try {
    const data = JSON.parse(safeDecode(raw))
    if (data && typeof data.kind === "string") return data as UiError
  } catch {
    // Not JSON — treat the whole thing as a message.
  }
  return { kind: "message", message: safeDecode(raw) }
}

/** Per-kind presentation: title + body, rendered inside the shared card. */
function render(payload: UiError) {
  switch (payload.kind) {
    case "already-running":
      return {
        title: "RCM Already Running",
        body: (
          <p style={styles.message}>
            Another instance of RCM is already running.
            {"\n\n"}Please close it before starting a new one.
          </p>
        ),
      }

    case "missing-programs":
      return {
        title: "Missing Program",
        body: (
          <>
            <p style={styles.lead}>
              This action needs a program that is not installed, or not on your PATH:
            </p>
            <ul style={styles.list}>
              {payload.programs.map((name) => (
                <li key={name} style={styles.listItem}>
                  <code style={styles.code}>{name}</code>
                </li>
              ))}
            </ul>
            <p style={styles.hint}>Install the program(s) above, then try again.</p>
          </>
        ),
      }

    case "message":
      return { title: "RCM Error", body: <p style={styles.message}>{payload.message}</p> }
  }
}

export const ErrorPage: React.FC = () => {
  const payload = parsePayload(window.location.hash)
  const { title, body } = render(payload)

  return (
    <>
      <BodyReset />
      <div style={styles.overlay}>
        <div style={styles.card}>
          <div style={styles.icon}>⚠️</div>
          <h2 style={styles.title}>{title}</h2>
          {body}
          <button style={styles.btn} onClick={() => getCurrentWindow().close()}>
            Close
          </button>
        </div>
      </div>
    </>
  )
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    height: "100vh",
    overflow: "auto",
    padding: 16,
    fontFamily: "'Segoe UI', system-ui, sans-serif",
    background: "#1e1e1e",
    color: "#d4d4d4",
  },
  card: {
    textAlign: "center" as const,
    padding: "32px 40px",
    borderRadius: 12,
    background: "#1e1e1e",
    maxWidth: 720,
  },
  icon: {
    fontSize: 48,
    marginBottom: 12,
  },
  title: {
    fontSize: 20,
    fontWeight: 600,
    margin: "0 0 8px 0",
    color: "#f44747",
  },
  message: {
    // Long diagnostics (multi-line reports) must keep their line breaks and
    // remain readable: left-aligned, monospace, scrollable when tall.
    fontSize: 13,
    color: "#aaa",
    margin: "0 0 24px 0",
    lineHeight: 1.5,
    textAlign: "left",
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
    fontFamily: "Consolas, 'Cascadia Mono', monospace",
    maxHeight: "60vh",
    overflow: "auto",
  },
  lead: {
    fontSize: 13,
    color: "#aaa",
    margin: "0 0 12px 0",
    lineHeight: 1.5,
  },
  list: {
    listStyle: "none",
    padding: 0,
    margin: "0 0 16px 0",
    display: "flex",
    flexDirection: "column" as const,
    gap: 6,
  },
  listItem: {
    display: "flex",
    justifyContent: "center",
  },
  code: {
    fontFamily: "Consolas, 'Cascadia Mono', monospace",
    fontSize: 13,
    color: "#ce9178",
    background: "#252526",
    border: "1px solid #3c3c3c",
    borderRadius: 4,
    padding: "2px 10px",
  },
  hint: {
    fontSize: 12,
    color: "#777",
    margin: "0 0 20px 0",
  },
  btn: {
    padding: "8px 24px",
    border: "1px solid #555",
    borderRadius: 6,
    background: "#3c3c3c",
    color: "#d4d4d4",
    cursor: "pointer",
    fontSize: 13,
    fontFamily: "inherit",
  },
}
