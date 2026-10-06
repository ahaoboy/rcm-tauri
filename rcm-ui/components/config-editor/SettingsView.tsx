/**
 * SettingsView — the tray's options as a form.
 *
 * The same settings the tray menu exposes, but as labelled controls. Every change
 * is applied immediately (there is no save step): each control writes straight
 * through to Rust, which returns the resulting state so the UI always shows what
 * is actually in effect — several of these can be refused by the OS.
 *
 * Explanations live in tooltips rather than as body text: there are few enough
 * settings that a paragraph under each would bury the controls, and the ⓘ marker
 * is enough to advertise that one is there.
 */

import Alert from "@mui/material/Alert"
import Box from "@mui/material/Box"
import Button from "@mui/material/Button"
import Divider from "@mui/material/Divider"
import Snackbar from "@mui/material/Snackbar"
import Stack from "@mui/material/Stack"
import Switch from "@mui/material/Switch"
import ToggleButton from "@mui/material/ToggleButton"
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup"
import Tooltip from "@mui/material/Tooltip"
import Typography from "@mui/material/Typography"
import React, { useCallback, useEffect, useState } from "react"

import {
  applyChanges,
  getSettings,
  resetSettings,
  type Settings,
  type Style,
  type ThemeName,
  updateSettings,
} from "../../api/menuEvents"
import { useConfigStore } from "../../stores"

const STYLE_OPTIONS: { value: Style; label: string }[] = [
  { value: "win11", label: "Windows 11" },
  { value: "classic", label: "Classic" },
]

const THEME_OPTIONS: { value: ThemeName; label: string }[] = [
  { value: "system", label: "System" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
]

export const SettingsView: React.FC = () => {
  const [settings, setSettings] = useState<Settings | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  // ThemeProvider reads the store, so writing the theme here re-themes the whole
  // window — not just this tab.
  const setTheme = useConfigStore((s) => s.setTheme)

  useEffect(() => {
    getSettings()
      .then(setSettings)
      .catch((e) => setError(String(e)))
  }, [])

  const reload = useCallback(() => {
    getSettings()
      .then(setSettings)
      .catch((e) => setError(String(e)))
  }, [])

  /**
   * Apply a change and adopt whatever state comes back.
   *
   * The form is disabled while a write is in flight, so two rapid clicks cannot
   * interleave.
   */
  const apply = useCallback(
    (patch: Partial<Settings>) => {
      setBusy(true)
      setError(null)
      // Optimistic: re-theme at once rather than waiting for the round trip.
      if (patch.theme) setTheme(patch.theme)
      updateSettings(patch)
        .then((next) => {
          setSettings(next)
          setNotice("Applied")
        })
        .catch((e) => {
          setError(String(e))
          // The write failed, so the optimistic change was wrong — re-read.
          reload()
        })
        .finally(() => setBusy(false))
    },
    [setTheme, reload],
  )

  const runAction = useCallback((action: () => Promise<Settings | void>, message: string) => {
    setBusy(true)
    setError(null)
    action()
      .then((next) => {
        if (next) setSettings(next)
        setNotice(message)
      })
      .catch((e) => setError(String(e)))
      .finally(() => setBusy(false))
  }, [])

  if (!settings) {
    return (
      <Box sx={{ p: 2 }}>
        {error ? <Alert severity="error">{error}</Alert> : <Typography>Loading…</Typography>}
      </Box>
    )
  }

  return (
    <Box sx={{ p: 2, flex: 1, overflow: "auto" }}>
      <Stack divider={<Divider />}>
        <Row label="Menu style" tip="Windows 11 is compact; Classic is the Windows 10 menu.">
          <ToggleButtonGroup
            exclusive
            size="small"
            disabled={busy}
            value={settings.style}
            onChange={(_, v: Style | null) => v && apply({ style: v })}
          >
            {STYLE_OPTIONS.map((o) => (
              <ToggleButton key={o.value} value={o.value}>
                {o.label}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
        </Row>

        <Row label="Theme" tip="System follows the Windows light/dark setting.">
          <ToggleButtonGroup
            exclusive
            size="small"
            disabled={busy}
            value={settings.theme}
            onChange={(_, v: ThemeName | null) => v && apply({ theme: v })}
          >
            {THEME_OPTIONS.map((o) => (
              <ToggleButton key={o.value} value={o.value}>
                {o.label}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
        </Row>

        <Row label="Icon ribbon" tip="Show the quick-action icons above the menu.">
          <Toggle checked={settings.icons} disabled={busy} onChange={(v) => apply({ icons: v })} />
        </Row>

        <Row
          label="Hide the native menu"
          tip="On: RCM's menu replaces the Windows one. Off: the Windows menu is used and RCM's is not shown."
        >
          <Toggle
            checked={settings.blocking}
            disabled={busy}
            onChange={(v) => apply({ blocking: v })}
          />
        </Row>

        <Row
          label="Shell extension"
          detail={settings.extension_dll}
          tip="The DLL Explorer loads to capture right-clicks. Registration changes need Apply."
        >
          <Toggle
            checked={settings.registered}
            disabled={busy}
            onChange={(v) => apply({ registered: v })}
          />
        </Row>

        <Row label="Apply" tip="Restart Explorer so registration changes take effect.">
          <Button
            variant="contained"
            size="small"
            disabled={busy}
            onClick={() => runAction(applyChanges, "Explorer restarted")}
          >
            Apply
          </Button>
        </Row>

        <Row label="Launch at startup" tip="Start RCM when you sign in to Windows.">
          <Toggle
            checked={settings.autostart}
            disabled={busy}
            onChange={(v) => apply({ autostart: v })}
          />
        </Row>

        <Row
          label="Dev mode"
          tip="Keep the menu open after running a command, to run several in a row."
        >
          <Toggle checked={settings.dev} disabled={busy} onChange={(v) => apply({ dev: v })} />
        </Row>

        <Row
          label="Reset"
          tip="Restore rcm.config.json, rcm.js and style.css to the bundled defaults."
        >
          <Button
            variant="outlined"
            color="error"
            size="small"
            disabled={busy}
            onClick={() => runAction(resetSettings, "Reset to defaults")}
          >
            Reset
          </Button>
        </Row>
      </Stack>

      <Snackbar
        open={Boolean(notice) && !error}
        autoHideDuration={2000}
        message={notice}
        onClose={() => setNotice(null)}
      />
      <Snackbar
        open={Boolean(error)}
        autoHideDuration={6000}
        onClose={() => setError(null)}
        anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
      >
        <Alert severity="error" variant="filled" onClose={() => setError(null)}>
          {error}
        </Alert>
      </Snackbar>
    </Box>
  )
}

/**
 * One labelled setting, with the control on the right.
 *
 * `tip` becomes a tooltip behind an ⓘ marker rather than inline text, keeping the
 * row a single line. `detail` is for information worth seeing without hovering
 * (the DLL path).
 */
const Row: React.FC<{
  label: string
  tip?: string
  detail?: string
  children: React.ReactNode
}> = ({ label, tip, detail, children }) => (
  <Box
    sx={{
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      gap: 2,
      py: 1,
    }}
  >
    <Box sx={{ minWidth: 0 }}>
      <Typography variant="body2">
        {label}
        {tip && (
          <Tooltip title={tip}>
            <Box component="span" sx={{ color: "text.disabled", cursor: "help", pl: 0.5 }}>
              ⓘ
            </Box>
          </Tooltip>
        )}
      </Typography>
      {detail && (
        <Typography
          variant="caption"
          color="text.disabled"
          sx={{ display: "block", fontFamily: "monospace", wordBreak: "break-all" }}
        >
          {detail}
        </Typography>
      )}
    </Box>
    <Box sx={{ flexShrink: 0 }}>{children}</Box>
  </Box>
)

/** The on/off control for a single boolean setting. */
const Toggle: React.FC<{
  checked: boolean
  disabled?: boolean
  onChange: (v: boolean) => void
}> = ({ checked, disabled, onChange }) => (
  <Switch checked={checked} disabled={disabled} size="small" onChange={(_, v) => onChange(v)} />
)
