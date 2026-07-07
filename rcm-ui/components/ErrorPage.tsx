/**
 * ErrorPage — displays an error message from the hash fragment.
 *
 * Route: #error/url-encoded-message
 */

import { getCurrentWindow } from "@tauri-apps/api/window"
import React from "react"

import { BodyReset } from "./BodyReset"

export const ErrorPage: React.FC = () => {
  const raw = window.location.hash.replace("#error/", "")
  const message = decodeURIComponent(raw)

  return (
    <>
      <BodyReset />
      <div style={styles.overlay}>
        <div style={styles.card}>
          <div style={styles.icon}>⚠️</div>
          <h2 style={styles.title}>RCM Error</h2>
          <p style={styles.message}>{message}</p>
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
    fontFamily: "'Segoe UI', system-ui, sans-serif",
    background: "#1e1e1e",
    color: "#d4d4d4",
  },
  card: {
    textAlign: "center" as const,
    padding: "40px 56px",
    borderRadius: 12,
    background: "#1e1e1e",
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
    fontSize: 14,
    color: "#aaa",
    margin: "0 0 24px 0",
    lineHeight: 1.5,
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
