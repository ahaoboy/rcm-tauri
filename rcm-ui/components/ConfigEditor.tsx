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
 *   config-editor/SettingsView.tsx — the tray's options as a form
 *
 * Styling is MUI (`ThemeRoot` + `theme.ts`). Only this window uses MUI: the menu
 * popups are styled by the user-editable `style.css`, so they must stay plain CSS.
 */

import type { EditorView } from "@codemirror/view"
import Box from "@mui/material/Box"
import Button from "@mui/material/Button"
import Stack from "@mui/material/Stack"
import Tab from "@mui/material/Tab"
import Tabs from "@mui/material/Tabs"
import Typography from "@mui/material/Typography"
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
import { useTheme } from "../hooks/useTheme"
import { setRoute } from "../router"
import {
  ENV_TAB,
  FILES,
  SETTINGS_TAB,
  TABS,
  tabFromHash,
  type FileKey,
  type TabKey,
} from "./config-editor/constants"
import { EnvView } from "./config-editor/EnvView"
import { FileEditor } from "./config-editor/FileEditor"
import { SettingsView } from "./config-editor/SettingsView"
import { ThemeRoot } from "./ThemeRoot"

// ═══════════════════════════════════════════════════════════════════════════════
// ConfigEditor — parent that manages tabs & toolbar
// ═══════════════════════════════════════════════════════════════════════════════
export const ConfigEditor: React.FC = () => {
  const [active, setActive] = useState<TabKey>(tabFromHash)
  const [saved, setSaved] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [reloadKey, setReloadKey] = useState(0)
  // The RCM theme, not just the OS one: this window hosts the setting that
  // chooses it, so it has to react to the change itself. `useTheme` also puts
  // the `rcm-*` class on <html>, which is what re-colours the page.
  const dark = useTheme() === "dark"
  const viewMapRef = useRef<Map<string, EditorView>>(new Map())
  const originalRef = useRef<Map<string, string>>(new Map())
  const [loaded, setLoaded] = useState(false)
  const [urls, setUrls] = useState<Record<string, string | null>>({})

  const isEnv = active === ENV_TAB
  const isSettings = active === SETTINGS_TAB
  // Tabs that are not an editable file: no toolbar, no editor instance.
  const isPlainView = isEnv || isSettings
  // Narrow the active tab to a file key for the file-oriented handlers.
  const activeFile = (isPlainView ? "rcm.config.json" : active) as FileKey

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
    <ThemeRoot>
      <Box sx={{ display: "flex", flexDirection: "column", height: "100vh" }}>
        <Tabs
          value={active}
          onChange={(_, v: TabKey) => setRoute(`config/${v}`)}
          variant="scrollable"
          scrollButtons="auto"
          sx={{ borderBottom: 1, borderColor: "divider", minHeight: 40 }}
        >
          {TABS.map((f) => (
            <Tab key={f.key} value={f.key} label={f.label} />
          ))}
        </Tabs>

        {!isPlainView && (
          <Stack
            direction="row"
            spacing={1}
            sx={{ alignItems: "center", px: 1.5, py: 1, borderBottom: 1, borderColor: "divider" }}
          >
            <Button
              variant="contained"
              size="small"
              startIcon={<span>💾</span>}
              onClick={triggerSave}
            >
              Save
            </Button>
            <Button
              size="small"
              startIcon={<span>⬇️</span>}
              onClick={handlePull}
              disabled={!canPull}
            >
              Pull
            </Button>
            <Button
              size="small"
              startIcon={<span>🔄</span>}
              onClick={() => setReloadKey((k) => k + 1)}
            >
              Reload
            </Button>
            <Button
              size="small"
              startIcon={<span>📂</span>}
              onClick={() => {
                openInEditor(activeFile).catch((e) => setError(String(e)))
              }}
            >
              Open
            </Button>
            {!saved && (
              <Typography variant="caption" color="warning.main">
                ● Unsaved
              </Typography>
            )}
            {error && (
              <Typography variant="caption" color="error" sx={{ overflow: "hidden" }}>
                {error}
              </Typography>
            )}
          </Stack>
        )}

        {isEnv ? (
          <EnvView />
        ) : isSettings ? (
          <SettingsView />
        ) : (
          <>
            {!loaded && (
              <Box sx={{ p: 2 }}>
                <Typography color="text.secondary">Loading…</Typography>
              </Box>
            )}
            <Box sx={{ flex: 1, overflow: "hidden" }}>
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
            </Box>
          </>
        )}
      </Box>
    </ThemeRoot>
  )
}
