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

import { readConfigFile, saveConfigFile, openInEditor, notifyStyleUpdated } from "../api/menuEvents"
import { BodyReset } from "./BodyReset"

const FILES = [
  { key: "rcm.config.json", label: "rcm.config.json", lang: "json" },
  { key: "rcm.js", label: "rcm.js", lang: "javascript" },
  { key: "style.css", label: "style.css", lang: "css" },
] as const

type FileKey = (typeof FILES)[number]["key"]

const LANG: Record<string, () => Extension> = {
  javascript: () => javascript(),
  css: () => css(),
  json: () => json(),
}

function fileFromHash(): FileKey {
  const raw = window.location.hash.replace("#config/", "")
  return FILES.find((f) => f.key === raw) ? (raw as FileKey) : "rcm.config.json"
}

export const ConfigEditor: React.FC = () => {
  const [active, setActive] = useState<FileKey>(fileFromHash)
  const [saved, setSaved] = useState(true)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [reloadKey, setReloadKey] = useState(0)
  const editorRef = useRef<HTMLDivElement>(null)
  const viewRef = useRef<EditorView | null>(null)
  const dark = useRef(window.matchMedia("(prefers-color-scheme: dark)").matches)

  // Listen for system theme changes
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)")
    const handler = (e: MediaQueryListEvent) => {
      dark.current = e.matches
      setReloadKey((k) => k + 1)
    }
    mq.addEventListener("change", handler)
    return () => mq.removeEventListener("change", handler)
  }, [])

  const doSave = useCallback(async () => {
    if (!viewRef.current) return
    const content = viewRef.current.state.doc.toString()
    try {
      await saveConfigFile(active, content)
      setSaved(true)
      setError(null)
      // Notify all windows if style.css was edited
      if (active === "style.css") {
        notifyStyleUpdated(content).catch(console.error)
      }
    } catch (e) {
      setError(String(e))
    }
  }, [active])

  // Create / recreate editor when file changes
  useEffect(() => {
    const el = editorRef.current
    if (!el) return

    setLoading(true)
    setError(null)

    readConfigFile(active)
      .then((data) => {
        // Destroy previous editor
        viewRef.current?.destroy()

        const updateListener = EditorView.updateListener.of((update) => {
          if (update.docChanged) setSaved(false)
        })

        const saveKeymap = keymap.of([
          {
            key: "Ctrl-s",
            run: () => {
              doSave()
              return true
            },
          },
          {
            key: "Mod-s",
            run: () => {
              doSave()
              return true
            },
          },
        ])

        const view = new EditorView({
          state: EditorState.create({
            doc: data,
            extensions: [
              basicSetup,
              ...(dark.current ? [oneDark] : []),
              LANG[FILES.find((f) => f.key === active)?.lang ?? "javascript"]?.(),
              updateListener,
              saveKeymap,
              keymap.of([indentWithTab]),
              EditorView.theme({
                "&": { height: "100%" },
                ".cm-scroller": { overflow: "auto" },
                ".cm-scroller::-webkit-scrollbar": { width: "8px", height: "8px" },
                ".cm-scroller::-webkit-scrollbar-track": { background: "#1e1e1e" },
                ".cm-scroller::-webkit-scrollbar-thumb": {
                  background: "#424242",
                  borderRadius: "4px",
                },
                ".cm-scroller::-webkit-scrollbar-thumb:hover": { background: "#555" },
              }),
            ],
          }),
          parent: el,
        })

        viewRef.current = view
        setSaved(true)
        setLoading(false)
      })
      .catch((e) => {
        setError(String(e))
        setLoading(false)
      })

    return () => {
      viewRef.current?.destroy()
      viewRef.current = null
    }
  }, [active, reloadKey, doSave])

  // Listen for hash changes
  useEffect(() => {
    const onHashChange = () => setActive(fileFromHash())
    window.addEventListener("hashchange", onHashChange)
    return () => window.removeEventListener("hashchange", onHashChange)
  }, [])

  const handleOpenExternal = async () => {
    try {
      await openInEditor(active)
    } catch (e) {
      setError(String(e))
    }
  }

  const handleTabClick = (key: FileKey) => {
    window.location.hash = `config/${key}`
  }

  return (
    <>
      <BodyReset />
      <div style={styles.container}>
        <div style={styles.tabs}>
          {FILES.map((f) => (
            <button
              key={f.key}
              onClick={() => handleTabClick(f.key)}
              style={{ ...styles.tab, ...(active === f.key ? styles.tabActive : {}) }}
            >
              {f.label}
            </button>
          ))}
        </div>

        <div style={styles.toolbar}>
          <button onClick={doSave} disabled={loading} style={styles.btn}>
            💾 Save
          </button>
          <button onClick={() => setReloadKey((k) => k + 1)} disabled={loading} style={styles.btn}>
            🔄 Reload
          </button>
          <button onClick={handleOpenExternal} style={styles.btn}>
            📂 Open
          </button>
          {!saved && <span style={styles.unsaved}>● Unsaved</span>}
          {error && <span style={styles.err}>{error}</span>}
        </div>

        {loading && !viewRef.current ? <div style={styles.loading}>Loading…</div> : null}
        <div ref={editorRef} style={styles.editor} />
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
    borderBottom: "2px solid transparent",
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
