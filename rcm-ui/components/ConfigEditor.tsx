/**
 * ConfigEditor — tabbed window for editing RCM config files, plus a read-only
 * inspector for the process environment variables.
 *
 * Hash-based routing:
 *   #config/rcm.config.json → rcm.config.json
 *   #config/rcm.js          → rcm.js
 *   #config/style.css       → style.css
 *   #config/env             → environment variable inspector (not a file)
 *
 * The heavy lifting lives in sibling modules:
 *   config-editor/constants.ts   — file list, tab routing
 *   config-editor/FileEditor.tsx — one CodeMirror instance per file
 *   config-editor/EnvView.tsx    — environment variable inspector
 *   config-editor/styles.ts      — shared inline styles
 */

import type { EditorView } from "@codemirror/view"
import React, { useCallback, useEffect, useRef, useState } from "react"

import {
  getConfig,
  notifyStyleUpdated,
  openInEditor,
  pullConfig,
  pullCss,
  pullJs,
  saveConfigFile,
  showError,
} from "../api/menuEvents"
import { BodyReset } from "./BodyReset"
import {
  ENV_TAB,
  FILES,
  TABS,
  tabFromHash,
  type FileKey,
  type TabKey,
} from "./config-editor/constants"
import { EnvView } from "./config-editor/EnvView"
import { FileEditor } from "./config-editor/FileEditor"
import { styles } from "./config-editor/styles"

// ═══════════════════════════════════════════════════════════════════════════════
// ConfigEditor — parent that manages tabs & toolbar
// ═══════════════════════════════════════════════════════════════════════════════
export const ConfigEditor: React.FC = () => {
  const [active, setActive] = useState<TabKey>(tabFromHash)
  const [saved, setSaved] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [reloadKey, setReloadKey] = useState(0)
  const darkRef = useRef(window.matchMedia("(prefers-color-scheme: dark)").matches)
  const [dark, setDark] = useState(darkRef.current)
  const viewMapRef = useRef<Map<string, EditorView>>(new Map())
  const originalRef = useRef<Map<string, string>>(new Map())
  const [loaded, setLoaded] = useState(false)
  const [urls, setUrls] = useState<Record<string, string | null>>({})

  const isEnv = active === ENV_TAB
  // Narrow the active tab to a file key for the file-oriented handlers.
  const activeFile = (isEnv ? "rcm.config.json" : active) as FileKey

  // Track system theme
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)")
    const handler = (e: MediaQueryListEvent) => {
      darkRef.current = e.matches
      setDark(e.matches)
      setReloadKey((k) => k + 1)
    }
    mq.addEventListener("change", handler)
    return () => mq.removeEventListener("change", handler)
  }, [])

  // Hash → tab
  useEffect(() => {
    const onHashChange = () => setActive(tabFromHash())
    window.addEventListener("hashchange", onHashChange)
    return () => window.removeEventListener("hashchange", onHashChange)
  }, [])

  // Fetch pull URLs
  useEffect(() => {
    getConfig()
      .then((cfg) =>
        setUrls({
          "rcm.js": cfg.js_url,
          "style.css": cfg.css_url,
          "rcm.config.json": cfg.config_url,
        }),
      )
      .catch(() => {})
  }, [])

  const canPull = !!urls[activeFile]

  const registerView = useCallback((key: FileKey, view: EditorView | null) => {
    if (view) viewMapRef.current.set(key, view)
    else viewMapRef.current.delete(key)
  }, [])

  const triggerSave = useCallback(async () => {
    const file = activeFile
    const view = viewMapRef.current.get(file)
    if (!view) return
    const content = view.state.doc.toString()
    try {
      await saveConfigFile(file, content)
      originalRef.current.set(file, content)
      setSaved(true)
      setError(null)
      if (file === "style.css") notifyStyleUpdated(content).catch(console.error)
    } catch (e) {
      setError(String(e))
    }
  }, [activeFile])

  const handlePull = useCallback(async () => {
    setError(null)
    const pullFn =
      activeFile === "rcm.js" ? pullJs : activeFile === "style.css" ? pullCss : pullConfig
    try {
      const path = await pullFn()
      setReloadKey((k) => k + 1)
      setError(`Pulled → ${path}`)
    } catch (e) {
      showError(String(e)).catch(() => setError(String(e)))
    }
  }, [activeFile])

  return (
    <>
      <BodyReset />
      <div style={styles.container}>
        <div style={styles.tabs}>
          {TABS.map((f) => (
            <button
              key={f.key}
              onClick={() => {
                window.location.hash = `config/${f.key}`
              }}
              style={{ ...styles.tab, ...(active === f.key ? styles.tabActive : {}) }}
            >
              {f.label}
            </button>
          ))}
        </div>

        {!isEnv && (
          <div style={styles.toolbar}>
            <button onClick={triggerSave} style={styles.btn}>
              💾 Save
            </button>
            <button
              onClick={handlePull}
              disabled={!canPull}
              style={{ ...styles.btn, ...(canPull ? {} : styles.btnDisabled) }}
            >
              ⬇️ Pull
            </button>
            <button onClick={() => setReloadKey((k) => k + 1)} style={styles.btn}>
              🔄 Reload
            </button>
            <button
              onClick={() => {
                openInEditor(activeFile).catch((e) => setError(String(e)))
              }}
              style={styles.btn}
            >
              📂 Open
            </button>
            {!saved && <span style={styles.unsaved}>● Unsaved</span>}
            {error && <span style={styles.err}>{error}</span>}
          </div>
        )}

        {isEnv ? (
          <EnvView />
        ) : (
          <>
            {!loaded && <div style={styles.loading}>Loading…</div>}
            <div style={styles.editor}>
              {FILES.map((f) => (
                <FileEditor
                  key={f.key}
                  fileKey={f.key}
                  active={active === f.key}
                  reloadKey={active === f.key ? reloadKey : 0}
                  isDark={dark}
                  onContentChange={(content) => {
                    setSaved(originalRef.current.get(f.key) === content)
                  }}
                  onError={setError}
                  onLoaded={(original) => {
                    originalRef.current.set(f.key, original)
                    setSaved(true)
                    setLoaded(true)
                  }}
                  registerView={registerView}
                  triggerSave={triggerSave}
                />
              ))}
            </div>
          </>
        )}
      </div>
    </>
  )
}
