/**
 * ConfigEditor — file editor for RCM config files with syntax highlighting.
 *
 * Hash-based routing:
 *   #config/rcm.js         → rcm.js
 *   #config/style.css      → style.css
 *   #config/rcm.config.json → rcm.config.json
 */

import { indentWithTab } from "@codemirror/commands"
import { css } from "@codemirror/lang-css"
import { javascript } from "@codemirror/lang-javascript"
import { json } from "@codemirror/lang-json"
import { EditorState } from "@codemirror/state"
import type { Extension } from "@codemirror/state"
import { oneDark } from "@codemirror/theme-one-dark"
import { EditorView, keymap } from "@codemirror/view"
import { basicSetup } from "codemirror"
import React, { useEffect, useState, useRef, useCallback } from "react"

import {
  readConfigFile,
  saveConfigFile,
  openInEditor,
  notifyStyleUpdated,
  pullJs,
  pullCss,
  pullConfig,
  showError,
} from "../api/menuEvents"
import { BodyReset } from "./BodyReset"

const FILES = [
  { key: "rcm.config.json", label: "rcm.config.json", lang: "json" },
  { key: "rcm.js", label: "rcm.js", lang: "javascript" },
  { key: "style.css", label: "style.css", lang: "css" },
] as const
type FileKey = (typeof FILES)[number]["key"]

const LANG: Record<string, () => Extension> = {
  javascript,
  json,
  css,
}

const FILE_BY_KEY: Record<FileKey, (typeof FILES)[number]> = Object.fromEntries(
  FILES.map((f) => [f.key, f]),
) as any

function fileFromHash(): FileKey {
  const raw = window.location.hash.replace("#config/", "")
  return FILE_BY_KEY[raw as FileKey] ? (raw as FileKey) : "rcm.config.json"
}

const SCROLLBAR_THEME = EditorView.theme({
  "&": { height: "100%" },
  ".cm-scroller": { overflow: "auto" },
  ".cm-scroller::-webkit-scrollbar": { width: "8px", height: "8px" },
  ".cm-scroller::-webkit-scrollbar-track": { background: "#1e1e1e" },
  ".cm-scroller::-webkit-scrollbar-thumb": { background: "#424242", borderRadius: "4px" },
  ".cm-scroller::-webkit-scrollbar-thumb:hover": { background: "#555" },
})

function createEditorState(
  doc: string,
  lang: string,
  isDark: boolean,
  onDirty: () => void,
  onSave: () => void,
) {
  return EditorState.create({
    doc,
    extensions: [
      basicSetup,
      ...(isDark ? [oneDark] : []),
      LANG[lang]?.(),
      keymap.of([
        {
          key: "Ctrl-s",
          run: () => {
            onSave()
            return true
          },
        },
        {
          key: "Mod-s",
          run: () => {
            onSave()
            return true
          },
        },
        indentWithTab,
      ]),
      EditorView.updateListener.of((u) => {
        if (u.docChanged) onDirty()
      }),
      SCROLLBAR_THEME,
    ],
  })
}

// ── EditorActivity — owns a single CodeMirror editor, lazy-created on first activation ──
const EditorActivity: React.FC<{
  fileKey: FileKey
  active: boolean
  reloadKey: number
  isDark: boolean
  onDirty: () => void
  onError: (msg: string) => void
  onLoaded: () => void
  registerView: (key: FileKey, view: EditorView | null) => void
  triggerSave: () => void
}> = ({
  fileKey,
  active,
  reloadKey,
  isDark,
  onDirty,
  onError,
  onLoaded,
  registerView,
  triggerSave,
}) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewRef = useRef<EditorView | null>(null)
  const createdRef = useRef(false)

  // Create editor on first activation
  useEffect(() => {
    if (!active || createdRef.current) return
    const el = containerRef.current
    if (!el) return

    readConfigFile(fileKey)
      .then((data) => {
        if (viewRef.current) return
        viewRef.current = new EditorView({
          state: createEditorState(data, FILE_BY_KEY[fileKey].lang, isDark, onDirty, triggerSave),
          parent: el,
        })
        registerView(fileKey, viewRef.current)
        createdRef.current = true
        onLoaded()
      })
      .catch((e) => onError(String(e)))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [active])

  // Reload: destroy & re-trigger creation
  useEffect(() => {
    if (reloadKey === 0 || !createdRef.current) return
    viewRef.current?.destroy()
    viewRef.current = null
    registerView(fileKey, null)
    createdRef.current = false
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reloadKey])

  // Cleanup on unmount
  useEffect(
    () => () => {
      viewRef.current?.destroy()
      registerView(fileKey, null)
      // eslint-disable-next-line react-hooks/exhaustive-deps
    },
    [],
  )

  return <div ref={containerRef} style={{ height: "100%", display: active ? "" : "none" }} />
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConfigEditor — parent that manages tabs & toolbar
// ═══════════════════════════════════════════════════════════════════════════════
export const ConfigEditor: React.FC = () => {
  const [active, setActive] = useState<FileKey>(fileFromHash)
  const [saved, setSaved] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [reloadKey, setReloadKey] = useState(0)
  const darkRef = useRef(window.matchMedia("(prefers-color-scheme: dark)").matches)
  const [dark, setDark] = useState(darkRef.current)
  const viewMapRef = useRef<Map<string, EditorView>>(new Map())
  const [loaded, setLoaded] = useState(false)

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
    const onHashChange = () => setActive(fileFromHash())
    window.addEventListener("hashchange", onHashChange)
    return () => window.removeEventListener("hashchange", onHashChange)
  }, [])

  const registerView = useCallback((key: FileKey, view: EditorView | null) => {
    if (view) viewMapRef.current.set(key, view)
    else viewMapRef.current.delete(key)
  }, [])

  const triggerSave = useCallback(async () => {
    const view = viewMapRef.current.get(active)
    if (!view) return
    try {
      await saveConfigFile(active, view.state.doc.toString())
      setSaved(true)
      setError(null)
      if (active === "style.css") notifyStyleUpdated(view.state.doc.toString()).catch(console.error)
    } catch (e) {
      setError(String(e))
    }
  }, [active])

  const handlePull = useCallback(async () => {
    setError(null)
    const pullFn = active === "rcm.js" ? pullJs : active === "style.css" ? pullCss : pullConfig
    try {
      const path = await pullFn()
      setReloadKey((k) => k + 1)
      setError(`Pulled → ${path}`)
    } catch (e) {
      showError(String(e))
    }
  }, [active])

  return (
    <>
      <BodyReset />
      <div style={styles.container}>
        <div style={styles.tabs}>
          {FILES.map((f) => (
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

        <div style={styles.toolbar}>
          <button onClick={triggerSave} style={styles.btn}>
            💾 Save
          </button>
          <button onClick={handlePull} style={styles.btn}>
            ⬇️ Pull
          </button>
          <button onClick={() => setReloadKey((k) => k + 1)} style={styles.btn}>
            🔄 Reload
          </button>
          <button
            onClick={() => {
              openInEditor(active).catch((e) => setError(String(e)))
            }}
            style={styles.btn}
          >
            📂 Open
          </button>
          {!saved && <span style={styles.unsaved}>● Unsaved</span>}
          {error && <span style={styles.err}>{error}</span>}
        </div>

        {!loaded && <div style={styles.loading}>Loading…</div>}
        <div style={styles.editor}>
          {FILES.map((f) => (
            <EditorActivity
              key={f.key}
              fileKey={f.key}
              active={active === f.key}
              reloadKey={active === f.key ? reloadKey : 0}
              isDark={dark}
              onDirty={() => setSaved(false)}
              onError={setError}
              onLoaded={() => setLoaded(true)}
              registerView={registerView}
              triggerSave={triggerSave}
            />
          ))}
        </div>
      </div>
    </>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    flexDirection: "column",
    height: "100vh",
    background: "#1e1e1e",
    color: "#d4d4d4",
  },
  tabs: {
    display: "flex",
    borderBottom: "1px solid #333",
    background: "#252526",
    flexShrink: 0,
  },
  tab: {
    padding: "8px 16px",
    border: "none",
    background: "transparent",
    color: "#888",
    cursor: "pointer",
    fontSize: 13,
    fontFamily: "inherit",
    borderBottomWidth: 2,
    borderBottomStyle: "solid",
    borderBottomColor: "transparent",
  },
  tabActive: { color: "#fff", borderBottomColor: "#ff85a2" },
  toolbar: {
    display: "flex",
    alignItems: "center",
    gap: 8,
    padding: "6px 10px",
    background: "#2d2d2d",
    borderBottom: "1px solid #333",
    flexShrink: 0,
  },
  btn: {
    padding: "4px 12px",
    border: "1px solid #555",
    borderRadius: 4,
    background: "#3c3c3c",
    color: "#d4d4d4",
    cursor: "pointer",
    fontSize: 12,
    fontFamily: "inherit",
  },
  unsaved: { color: "#f0c040", fontSize: 12, marginLeft: 8 },
  err: { color: "#f44747", fontSize: 12, marginLeft: 8 },
  editor: { flex: 1, overflow: "hidden" },
  loading: {
    flex: 1,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    color: "#888",
  },
}
