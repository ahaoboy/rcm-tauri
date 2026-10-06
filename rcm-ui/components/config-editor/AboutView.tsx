/**
 * AboutView — application identity, version and file locations.
 *
 * The content lives here; Rust supplies only the runtime facts the webview
 * cannot read (version and the on-disk paths) plus the one action that needs a
 * native handler — revealing the app folder.
 */

import Box from "@mui/material/Box"
import Button from "@mui/material/Button"
import Link from "@mui/material/Link"
import Stack from "@mui/material/Stack"
import Tooltip from "@mui/material/Tooltip"
import Typography from "@mui/material/Typography"
import React, { useEffect, useState } from "react"

import { getRuntimePaths, openConfigFolder, type RuntimePaths } from "../../api/menuEvents"

/** Presentation content — owned by the UI, not by the backend. */
const DESCRIPTION =
  "A customizable Windows right-click context menu, backed by a shell extension and scriptable with JavaScript."
const REPO_URL = "https://github.com/ahaoboy/rcm-tauri"

/** The application icon, served from `public/` so it is reached at `/`. */
const LOGO_URL = "/icon.png"

export const AboutView: React.FC = () => {
  const [paths, setPaths] = useState<RuntimePaths | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    getRuntimePaths()
      .then(setPaths)
      .catch((e) => setError(String(e)))
  }, [])

  const openFolder = () => {
    openConfigFolder().catch((e) => setError(String(e)))
  }

  // `display: flex` + `m: auto` on the inner stack centres it on both axes while
  // still allowing the outer box to scroll if the content ever exceeds the viewport
  // (unlike `justifyContent: center`, which would clip the top and become unreachable).
  return (
    <Box sx={{ flex: 1, overflow: "auto", p: 3, display: "flex" }}>
      <Stack
        sx={{ m: "auto", alignItems: "center", textAlign: "center", width: "100%", maxWidth: 460 }}
      >
        <Box component="img" src={LOGO_URL} alt="" sx={{ width: 64, height: 64, mb: 1 }} />
        <Typography variant="body2" color="text.secondary" sx={{ maxWidth: 400, mb: 3 }}>
          {DESCRIPTION}
        </Typography>

        <Box sx={{ width: "100%", textAlign: "left" }}>
          <Fact label="Version" value={paths?.version} />
          <Fact label="Folder" value={paths?.exe_dir} />
          <Fact label="Config" value={paths?.config_path} />
          <Fact label="Log" value={paths?.log_path} />
        </Box>

        {error && (
          <Typography variant="caption" color="error" sx={{ mt: 1 }}>
            {error}
          </Typography>
        )}

        <Stack direction="row" spacing={1} sx={{ mt: 3 }}>
          {/*
            An ordinary link: the opener plugin intercepts `_blank` http(s) links
            and hands them to the default browser, so no backend command is needed.
          */}
          <Button component={Link} href={REPO_URL} target="_blank" rel="noreferrer" size="small">
            GitHub
          </Button>
          <Button size="small" onClick={openFolder}>
            Open app folder
          </Button>
        </Stack>
      </Stack>
    </Box>
  )
}

/**
 * One `label / value` row.
 *
 * Renders nothing until the value has loaded, so the list does not jump as each
 * fact arrives. The full value is shown on hover because a long path has to be
 * ellipsized here.
 */
const Fact: React.FC<{ label: string; value?: string }> = ({ label, value }) => {
  if (!value) return null
  return (
    <Box
      sx={{ display: "grid", gridTemplateColumns: "auto minmax(0, 1fr)", gap: "4px 12px", py: 0.5 }}
    >
      <Typography variant="caption" color="text.secondary" sx={{ whiteSpace: "nowrap" }}>
        {label}
      </Typography>
      <Tooltip title={value} placement="top">
        <Typography
          variant="caption"
          color="text.disabled"
          sx={{
            fontFamily: "monospace",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {value}
        </Typography>
      </Tooltip>
    </Box>
  )
}
