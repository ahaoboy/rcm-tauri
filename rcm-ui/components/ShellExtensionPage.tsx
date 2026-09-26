/**
 * ShellExtensionPage — shown when the `rcm_com` pipe could not be reached.
 *
 * The pipe is created by the shell extension DLL inside Explorer, so a failure
 * at startup is expected on a fresh install: nothing has loaded the DLL yet.
 * The user fixes it from the tray (Register → Apply, which restarts Explorer)
 * and then clicks **Retry** here to reconnect.
 *
 * Route: #shell-ext/url-encoded-report
 */

import { getCurrentWindow } from "@tauri-apps/api/window"
import React, { useCallback, useState } from "react"

import { retryShellExtension } from "../api/menuEvents"
import { BodyReset } from "./BodyReset"
import { styles } from "./ShellExtensionPage.styles"

/** How long the success state stays on screen before the window closes. */
const AUTO_CLOSE_MS = 1200

export const ShellExtensionPage: React.FC = () => {
  const [report, setReport] = useState(() =>
    decodeURIComponent(window.location.hash.replace("#shell-ext/", "")),
  )
  const [busy, setBusy] = useState(false)
  const [connected, setConnected] = useState(false)
  const [checkedAt, setCheckedAt] = useState<string | null>(null)

  const retry = useCallback(async () => {
    setBusy(true)
    try {
      const result = await retryShellExtension()
      setCheckedAt(new Date().toLocaleTimeString())
      if (result.ok) {
        setConnected(true)
        setTimeout(() => {
          getCurrentWindow().close()
        }, AUTO_CLOSE_MS)
      } else {
        // Still not loaded — refresh the report so the user sees current
        // DLL presence and registry state (the fix may have been partial).
        setReport(result.report)
      }
    } finally {
      setBusy(false)
    }
  }, [])

  if (connected) {
    return (
      <>
        <BodyReset />
        <div style={styles.overlay}>
          <div style={styles.card}>
            <h2 style={styles.okTitle}>Connected</h2>
            <p style={styles.okMessage}>
              The shell extension is loaded. The right-click menu is ready.
            </p>
          </div>
        </div>
      </>
    )
  }

  return (
    <>
      <BodyReset />
      <div style={styles.overlay}>
        <div style={styles.card}>
          <h2 style={styles.title}>Shell Extension Not Running</h2>

          <pre style={styles.report}>{report}</pre>

          <div style={styles.actions}>
            <span style={styles.status}>{checkedAt ? `Checked ${checkedAt}` : ""}</span>
            <button onClick={retry} disabled={busy} style={styles.retryBtn}>
              {busy ? "Checking…" : "🔄 Retry"}
            </button>
            <button onClick={() => getCurrentWindow().close()} style={styles.btn}>
              Close
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
