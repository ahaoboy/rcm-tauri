/**
 * ShellExtensionPage — shown when the `rcm_com` pipe could not be reached.
 *
 * The pipe is created by the shell extension DLL inside Explorer, so a failure
 * at startup is expected on a fresh install: nothing has loaded the DLL yet.
 * The user fixes it from the tray (Register → Apply, which restarts Explorer)
 * and then clicks **Retry** here — or just right-clicks a folder, which the page
 * notices on its own and confirms.
 *
 * Route: #shell-ext/url-encoded-report
 */

import { getCurrentWindow } from "@tauri-apps/api/window"
import React, { useCallback, useEffect, useRef, useState } from "react"

import { retryShellExtension, shellExtensionConnected } from "../api/menuEvents"
import { getRoute } from "../router"
import { BodyReset } from "./BodyReset"
import { styles } from "./ShellExtensionPage.styles"

/** How long the success state stays on screen before the window closes. */
const AUTO_CLOSE_MS = 1200

/** How often to ask the backend whether the extension has been heard from. */
const POLL_MS = 1000

export const ShellExtensionPage: React.FC = () => {
  const route = getRoute()
  const [report, setReport] = useState(route.name === "shell-ext" ? route.report : "")
  const [busy, setBusy] = useState(false)
  const [connected, setConnected] = useState(false)
  const [checkedAt, setCheckedAt] = useState<string | null>(null)

  // Success can arrive from the poll and from a Retry click, either of which may
  // beat the other; closing twice is an error, so only the first one counts.
  const settled = useRef(false)
  const succeed = useCallback(() => {
    if (settled.current) return
    settled.current = true
    setConnected(true)
    setTimeout(() => {
      getCurrentWindow().close()
    }, AUTO_CLOSE_MS)
  }, [])

  // The extension can appear without the user pressing anything, so watch for it.
  // The first check runs immediately, in case it came up before this page opened.
  useEffect(() => {
    let cancelled = false
    const check = async () => {
      if (cancelled || settled.current) return
      try {
        const heard = await shellExtensionConnected()
        // Re-checked after the await: the window may have gone away while the
        // call was in flight.
        if (cancelled || settled.current || !heard) return
        setCheckedAt(new Date().toLocaleTimeString())
        succeed()
      } catch {
        // The backend is going away (window closing) — nothing to report.
      }
    }
    void check()
    const timer = setInterval(check, POLL_MS)
    return () => {
      cancelled = true
      clearInterval(timer)
    }
  }, [succeed])

  const retry = useCallback(async () => {
    setBusy(true)
    try {
      const result = await retryShellExtension()
      setCheckedAt(new Date().toLocaleTimeString())
      if (result.ok) {
        succeed()
      } else {
        // Still not loaded — refresh the report so the user sees current
        // DLL presence and registry state (the fix may have been partial).
        setReport(result.report)
      }
    } finally {
      setBusy(false)
    }
  }, [succeed])

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
