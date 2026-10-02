/**
 * ErrorPage — the single window for every error/diagnostic.
 *
 * Route: `#error/<url-encoded-json>` (see `router.ts`). The payload's `kind`
 * selects the layout; anything that fails to parse degrades to a plain message.
 */

import { getCurrentWindow } from "@tauri-apps/api/window"
import React from "react"

import { getRoute } from "../router"
import type { UiError } from "../types/error"
import { BodyReset } from "./BodyReset"

export type { UiError }

/** Presentation for one error kind, rendered inside the shared card. */
function render(payload: UiError) {
  switch (payload.kind) {
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
  const route = getRoute()
  const payload: UiError = route.name === "error" ? route.error : { kind: "message", message: "" }
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
    // minHeight (not height) + a card with `margin: auto` centres the card on
    // both axes *and* lets it scroll into view when taller than the window.
    // `align-items/justify-content: center` would clip the overflow instead.
    minHeight: "100vh",
    overflow: "auto",
    overflowX: "hidden",
    padding: "clamp(12px, 4vw, 24px)",
    fontFamily: "'Segoe UI', system-ui, sans-serif",
    background: "#1e1e1e",
    color: "#d4d4d4",
  },
  card: {
    margin: "auto",
    width: "100%",
    maxWidth: 720,
    textAlign: "center" as const,
    padding: "clamp(20px, 5vw, 32px) clamp(16px, 6vw, 40px)",
    borderRadius: 12,
    background: "#1e1e1e",
  },
  icon: {
    fontSize: "clamp(32px, 8vw, 48px)",
    marginBottom: 12,
  },
  title: {
    fontSize: 20,
    fontWeight: 600,
    margin: "0 0 8px 0",
    color: "#f44747",
    overflowWrap: "break-word",
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
    overflowWrap: "anywhere",
    fontFamily: "Consolas, 'Cascadia Mono', monospace",
    maxHeight: "50vh",
    overflow: "auto",
  },
  lead: {
    fontSize: 13,
    color: "#aaa",
    margin: "0 0 12px 0",
    lineHeight: 1.5,
    overflowWrap: "break-word",
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
    minWidth: 0,
  },
  code: {
    fontFamily: "Consolas, 'Cascadia Mono', monospace",
    fontSize: 13,
    color: "#ce9178",
    background: "#252526",
    border: "1px solid #3c3c3c",
    borderRadius: 4,
    padding: "2px 10px",
    // Let a long program name wrap rather than stretch the card.
    maxWidth: "100%",
    overflowWrap: "anywhere",
  },
  hint: {
    fontSize: 12,
    color: "#777",
    margin: "0 0 20px 0",
    overflowWrap: "break-word",
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
    maxWidth: "100%",
  },
}
