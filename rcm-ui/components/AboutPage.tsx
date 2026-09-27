/**
 * AboutPage — application identity, version and file locations.
 *
 * Route: #about
 *
 * The page's content lives here. Rust supplies only the runtime facts the
 * webview cannot read (version and the on-disk paths), plus the one action that
 * needs a native handler: revealing the config folder.
 */

import { getCurrentWindow } from "@tauri-apps/api/window"
import React, { useEffect, useState } from "react"

import { getRuntimePaths, openConfigFolder, type RuntimePaths } from "../api/menuEvents"
import { BodyReset } from "./BodyReset"

/** Presentation content — owned by the UI, not by the backend. */
const APP_NAME = "RCM"
const TAGLINE = "Right Click Menu"
const DESCRIPTION =
  "A customizable Windows right-click context menu, backed by a shell extension and scriptable with JavaScript."
const REPO_URL = "https://github.com/ahaoboy/rcm-tauri"

/**
 * The application icon.
 *
 * Served from `public/`, so it is copied to the build root and reached at `/`
 * rather than bundled or hashed.
 */
const LOGO_URL = "/icon.png"

/** The GitHub mark, used to signal that the link leaves the app. */
const GitHubMark: React.FC<{ size: number }> = ({ size }) => (
  <svg viewBox="0 0 16 16" width={size} height={size} fill="currentColor" aria-hidden="true">
    <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
  </svg>
)

export const AboutPage: React.FC = () => {
  const [paths, setPaths] = useState<RuntimePaths | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    getRuntimePaths()
      .then(setPaths)
      .catch((e) => setError(String(e)))
  }, [])

  /** Run a backend action, surfacing a failure instead of swallowing it. */
  const run = (action: () => Promise<void>) => {
    action().catch((e) => setError(String(e)))
  }

  return (
    <>
      <BodyReset />
      <div style={styles.page}>
        <img src={LOGO_URL} alt="" style={styles.logo} />
        <h1 style={styles.title}>{APP_NAME}</h1>
        <div style={styles.tagline}>{TAGLINE}</div>
        <p style={styles.description}>{DESCRIPTION}</p>

        <dl style={styles.facts}>
          <Fact label="Version" value={paths?.version} />
          <Fact label="Folder" value={paths?.exe_dir} />
          <Fact label="Config" value={paths?.config_path} />
          <Fact label="Log" value={paths?.log_path} />
        </dl>

        {error && <div style={styles.error}>{error}</div>}

        <div style={styles.actions}>
          {/*
            An ordinary link: the opener plugin intercepts `_blank` http(s)
            links and hands them to the default browser, so no backend command
            is needed. Requires the `opener:default` capability.
          */}
          <a
            style={{ ...styles.btn, ...styles.link }}
            href={REPO_URL}
            target="_blank"
            rel="noreferrer"
          >
            <GitHubMark size={14} />
            Github
          </a>
          <button style={styles.btn} onClick={() => run(openConfigFolder)}>
            Open App folder
          </button>
          <button
            style={{ ...styles.btn, ...styles.primary }}
            onClick={() => getCurrentWindow().close()}
          >
            Close
          </button>
        </div>
      </div>
    </>
  )
}

/** One `label / value` row; renders nothing until the value has loaded. */
const Fact: React.FC<{ label: string; value?: string }> = ({ label, value }) => {
  if (!value) return null
  return (
    <>
      <dt style={styles.dt}>{label}</dt>
      <dd style={styles.dd} title={value}>
        {value}
      </dd>
    </>
  )
}

const styles: Record<string, React.CSSProperties> = {
  page: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    height: "100vh",
    padding: "24px 28px",
    background: "#1e1e1e",
    color: "#d4d4d4",
    textAlign: "center",
  },
  logo: {
    width: 64,
    height: 64,
    marginBottom: 10,
    // The source art has its own padding; keep it crisp at this size.
    imageRendering: "auto",
  },
  title: {
    fontSize: 20,
    fontWeight: 600,
    margin: "0 0 2px 0",
  },
  tagline: {
    fontSize: 12,
    letterSpacing: "0.08em",
    textTransform: "uppercase",
    color: "#888",
    marginBottom: 12,
  },
  description: {
    fontSize: 13,
    lineHeight: 1.5,
    color: "#aaa",
    margin: "0 0 20px 0",
    maxWidth: 400,
  },
  facts: {
    // Long paths must not stretch the window: the grid keeps both columns
    // inside the available width and lets each value ellipsize.
    display: "grid",
    gridTemplateColumns: "auto minmax(0, 1fr)",
    gap: "6px 12px",
    width: "100%",
    margin: "0 0 20px 0",
    textAlign: "left",
    fontSize: 12,
  },
  dt: {
    color: "#888",
    whiteSpace: "nowrap",
  },
  dd: {
    margin: 0,
    color: "#bbb",
    fontFamily: "Consolas, 'Cascadia Mono', monospace",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  },
  error: {
    width: "100%",
    marginBottom: 12,
    padding: "6px 10px",
    borderRadius: 6,
    background: "rgba(244, 71, 71, 0.12)",
    color: "#f44747",
    fontSize: 12,
    textAlign: "left",
    wordBreak: "break-word",
  },
  actions: {
    display: "flex",
    gap: 8,
    marginTop: "auto",
  },
  btn: {
    padding: "8px 16px",
    border: "1px solid #555",
    borderRadius: 6,
    background: "#3c3c3c",
    color: "#d4d4d4",
    cursor: "pointer",
    fontSize: 13,
    fontFamily: "inherit",
  },
  /** A link styled as a button, so it matches the buttons beside it. */
  link: {
    display: "inline-flex",
    alignItems: "center",
    gap: 6,
    textDecoration: "none",
  },
  primary: {
    borderColor: "#0e639c",
    background: "#0e639c",
    color: "#fff",
  },
}
