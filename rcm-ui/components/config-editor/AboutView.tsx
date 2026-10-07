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

/** Commit-page URL for a `git describe` result, or `null` when there is none. */
function commitLink(commit: string | undefined) {
  if (!commit) return null
  // `describe` appends `-dirty`/`-modified` to a clean hash, and neither marker
  // is part of the object name — the link has to point at the commit alone.
  const hash = commit.replace(/-(dirty|modified|broken)$/, "")
  if (!/^[0-9a-f]{7,40}$/.test(hash)) return null
  return { label: commit, url: `${REPO_URL}/commit/${hash}` }
}

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

  const commit = commitLink(paths?.commit)

  // `display: flex` + `m: auto` on the inner stack centres it on both axes while
  // still allowing the outer box to scroll if the content ever exceeds the viewport
  // (unlike `justifyContent: center`, which would clip the top and become unreachable).
  return (
    <Box sx={{ flex: 1, overflow: "auto", p: 3, display: "flex" }}>
      <Stack sx={{ m: "auto", width: "100%", maxWidth: 560, alignItems: "center" }}>
        <Box component="img" src={LOGO_URL} alt="" sx={{ width: 64, height: 64, mb: 1 }} />
        <Typography variant="body2" color="text.secondary" sx={{ mb: 3, textAlign: "center" }}>
          {DESCRIPTION}
        </Typography>

        {/*
          A real table so the labels and values line up in columns regardless of
          how long a path is, and so each row can be told apart at a glance —
          which a stack of independent grids cannot do.
        */}
        <Box sx={{ width: "100%", overflowX: "auto" }}>
          <Box
            component="table"
            sx={{ width: "100%", borderCollapse: "collapse", tableLayout: "fixed" }}
          >
            <tbody>
              <Fact label="Version">
                {paths?.version ?? ""}
                {commit && (
                  <Tooltip title={`Git commit this build came from — ${commit.url}`}>
                    {/*
                      A link, not a button: the opener plugin intercepts `_blank`
                      http(s) links, and a commit is exactly the kind of thing to
                      cite in a bug report.
                    */}
                    <Box
                      component={Link}
                      href={commit.url}
                      target="_blank"
                      rel="noreferrer"
                      sx={styles.commit}
                    >
                      {commit.label}
                    </Box>
                  </Tooltip>
                )}
              </Fact>
              <Fact label="Folder" value={paths?.exe_dir} />
              <Fact label="Config" value={paths?.config_path} />
              <Fact label="Log" value={paths?.log_path} />
            </tbody>
          </Box>
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
 * One table row.
 *
 * `value` renders as a monospace path, ellipsized with the full text on hover
 * because a path is usually wider than the column. Children render as-is
 * instead, for the version row's mix of plain text and commit hash.
 */
const Fact: React.FC<{ label: string; value?: string; children?: React.ReactNode }> = ({
  label,
  value,
  children,
}) => (
  <Box component="tr" sx={styles.row}>
    <Box component="th" scope="row" sx={styles.label}>
      {label}
    </Box>
    <Box component="td" sx={styles.value}>
      {value !== undefined ? (
        <Tooltip title={value} placement="top">
          <Box component="span" sx={styles.path}>
            {value}
          </Box>
        </Tooltip>
      ) : (
        children
      )}
    </Box>
  </Box>
)

const styles = {
  row: {
    "&:not(:first-of-type) td, &:not(:first-of-type) th": { borderTop: "1px solid #2d2d2d" },
  },
  label: {
    width: "6.5rem",
    py: 1,
    pr: 2,
    textAlign: "left",
    verticalAlign: "top",
    fontWeight: 500,
    fontSize: 12,
    color: "text.secondary",
  },
  value: {
    py: 1,
    textAlign: "left",
    fontSize: 12,
    fontFamily: "monospace",
    color: "text.disabled",
    // `tableLayout: fixed` plus this is what lets a long path ellipsize rather
    // than stretch the table past the card.
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  },
  path: { display: "block", overflow: "hidden", textOverflow: "ellipsis" },
  commit: {
    ml: 1,
    px: 0.75,
    py: 0.25,
    borderRadius: 1,
    fontSize: 11,
    color: "text.secondary",
    background: "#252526",
    border: "1px solid #3c3c3c",
    textDecoration: "none",
    "&:hover": { color: "text.primary", borderColor: "#5a5a5a" },
  },
}
