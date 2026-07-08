/**
 * ConfigEditor — simple file editor for RCM config files.
 *
 * Hash-based routing:
 *   #config/rcm.js         → rcm.js
 *   #config/style.css      → style.css
 *   #config/rcm.config.json → rcm.config.json
 */

import React, { useEffect, useState, useCallback } from "react"

import { readConfigFile, saveConfigFile, openInEditor } from "../api/menuEvents"
import { BodyReset } from "./BodyReset"

const FILES = [
  { key: "rcm.js", label: "rcm.js", lang: "javascript" },
  { key: "style.css", label: "style.css", lang: "css" },
  { key: "rcm.config.json", label: "rcm.config.json", lang: "json" },
] as const

type FileKey = (typeof FILES)[number]["key"]

function fileFromHash(): FileKey {
  const raw = window.location.hash.replace("#config/", "")
  return FILES.find((f) => f.key === raw) ? (raw as FileKey) : "rcm.js"
}

export const ConfigEditor: React.FC = () => {
  const [active, setActive] = useState<FileKey>(fileFromHash)
  const [content, setContent] = useState("")
  const [saved, setSaved] = useState(true)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Load file content when tab changes
  const loadFile = useCallback(async (key: FileKey) => {
    setLoading(true)
    setError(null)
    try {
      const data = await readConfigFile(key)
      setContent(data)
      setSaved(true)
    } catch (e) {
      setError(String(e))
      setContent("")
    } finally {
      setLoading(false)
    }
  }, [])

  // Listen for hash changes
  useEffect(() => {
    const onHashChange = () => {
      const key = fileFromHash()
      setActive(key)
      loadFile(key)
    }
    window.addEventListener("hashchange", onHashChange)
    // Initial load
    onHashChange()
    return () => window.removeEventListener("hashchange", onHashChange)
  }, [loadFile])

  const handleSave = async () => {
    try {
      await saveConfigFile(active, content)
      setSaved(true)
      setError(null)
    } catch (e) {
      setError(String(e))
    }
  }

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
        {/* Tabs */}
        <div style={styles.tabs}>
          {FILES.map((f) => (
            <button
              key={f.key}
              onClick={() => handleTabClick(f.key)}
              style={{
                ...styles.tab,
                ...(active === f.key ? styles.tabActive : {}),
              }}
            >
              {f.label}
            </button>
          ))}
        </div>

        {/* Toolbar */}
        <div style={styles.toolbar}>
          <button onClick={handleSave} disabled={loading} style={styles.btn}>
            💾 Save
          </button>
          <button onClick={() => loadFile(active)} disabled={loading} style={styles.btn}>
            🔄 Reload
          </button>
          <button onClick={handleOpenExternal} style={styles.btn}>
            📂 Open
          </button>
          {!saved && <span style={styles.unsaved}>● Unsaved</span>}
          {error && <span style={styles.err}>{error}</span>}
        </div>

        {/* Editor */}
        {loading ? (
          <div style={styles.loading}>Loading…</div>
        ) : (
          <textarea
            value={content}
            onChange={(e) => {
              setContent(e.target.value)
              setSaved(false)
            }}
            style={styles.editor}
            spellCheck={false}
          />
        )}
      </div>
    </>
  )
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    flexDirection: "column",
    height: "100vh",
    fontFamily: "Consolas, monospace",
    fontSize: 13,
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
  tabActive: {
    color: "#fff",
    borderBottomColor: "#ff85a2",
  },
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
  unsaved: {
    color: "#f0c040",
    fontSize: 12,
    marginLeft: 8,
  },
  err: {
    color: "#f44747",
    fontSize: 12,
    marginLeft: 8,
  },
  editor: {
    flex: 1,
    padding: 12,
    border: "none",
    background: "#1e1e1e",
    color: "#d4d4d4",
    fontFamily: "Consolas, 'Courier New', monospace",
    fontSize: 13,
    lineHeight: 1.5,
    resize: "none",
    outline: "none",
    tabSize: 2,
  },
  loading: {
    flex: 1,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    color: "#888",
  },
}
