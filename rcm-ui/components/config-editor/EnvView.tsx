/**
 * EnvView — read-only inspector for the process environment variables.
 *
 * `*PATH`-style variables holding `;`-separated entries are split into a numbered
 * list for readability; every other value is shown on one line. Values (or
 * individual path entries) can be copied with a click.
 */

import Box from "@mui/material/Box"
import Button from "@mui/material/Button"
import Chip from "@mui/material/Chip"
import IconButton from "@mui/material/IconButton"
import List from "@mui/material/List"
import ListItem from "@mui/material/ListItem"
import ListItemText from "@mui/material/ListItemText"
import Stack from "@mui/material/Stack"
import TextField from "@mui/material/TextField"
import Tooltip from "@mui/material/Tooltip"
import Typography from "@mui/material/Typography"
import React, { useCallback, useEffect, useMemo, useState } from "react"

import { getEnvVars } from "../../api/menuEvents"
import type { EnvVar } from "../../api/menuEvents"

/** A path-like value: a `*PATH` variable holding `;`-separated entries. */
function isPathLike(key: string, value: string): boolean {
  return /path$/i.test(key) && value.includes(";")
}

export const EnvView: React.FC = () => {
  const [vars, setVars] = useState<EnvVar[]>([])
  const [filter, setFilter] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState<string | null>(null)

  const load = useCallback(() => {
    getEnvVars()
      .then((data) => {
        setVars(data)
        setError(null)
      })
      .catch((e) => setError(String(e)))
  }, [])

  useEffect(load, [load])

  const shown = useMemo(() => {
    const q = filter.trim().toLowerCase()
    if (!q) return vars
    return vars.filter((v) => v.key.toLowerCase().includes(q) || v.value.toLowerCase().includes(q))
  }, [vars, filter])

  const copy = useCallback((text: string, tag: string) => {
    navigator.clipboard.writeText(text).then(
      () => {
        setCopied(tag)
        setTimeout(() => setCopied((c) => (c === tag ? null : c)), 1200)
      },
      () => {},
    )
  }, [])

  return (
    <Box sx={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0 }}>
      <Stack
        direction="row"
        spacing={1}
        sx={{ alignItems: "center", p: 1, borderBottom: 1, borderColor: "divider" }}
      >
        <TextField
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder="Filter by name or value…"
          sx={{ flex: 1 }}
        />
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ fontVariantNumeric: "tabular-nums" }}
        >
          {shown.length} / {vars.length}
        </Typography>
        <Button size="small" onClick={load}>
          Refresh
        </Button>
        {error && (
          <Typography variant="caption" color="error">
            {error}
          </Typography>
        )}
      </Stack>

      <List dense disablePadding sx={{ flex: 1, overflow: "auto" }}>
        {shown.map((v) => {
          const pathLike = isPathLike(v.key, v.value)
          const entries = pathLike ? v.value.split(";").filter(Boolean) : []
          return (
            <ListItem
              key={v.key}
              alignItems="flex-start"
              sx={{ display: "block", borderBottom: 1, borderColor: "divider", px: 1.5, py: 0.75 }}
            >
              <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                <Typography
                  variant="caption"
                  sx={{ fontFamily: "monospace", fontWeight: 600, color: "primary.main" }}
                >
                  {v.key}
                </Typography>
                {pathLike && (
                  <Chip size="small" variant="outlined" label={`${entries.length} paths`} />
                )}
                <Tooltip title="Copy value">
                  <IconButton size="small" sx={{ ml: "auto" }} onClick={() => copy(v.value, v.key)}>
                    <Typography variant="caption">{copied === v.key ? "✓" : "⧉"}</Typography>
                  </IconButton>
                </Tooltip>
              </Box>

              {pathLike ? (
                <Box component="ol" sx={{ listStyle: "none", m: 0, mt: 0.5, p: 0 }}>
                  {entries.map((p, i) => {
                    const tag = `${v.key}#${i}`
                    return (
                      <Box
                        key={tag}
                        component="li"
                        sx={{ display: "flex", gap: 1, alignItems: "baseline", py: 0.25 }}
                      >
                        <Typography variant="caption" color="text.disabled" sx={{ minWidth: 18 }}>
                          {i + 1}
                        </Typography>
                        <Tooltip title="Click to copy">
                          <Typography
                            variant="caption"
                            onClick={() => copy(p, tag)}
                            sx={{
                              fontFamily: "monospace",
                              cursor: "pointer",
                              wordBreak: "break-all",
                              "&:hover": { color: "primary.main" },
                            }}
                          >
                            {copied === tag ? "✓ copied" : p}
                          </Typography>
                        </Tooltip>
                      </Box>
                    )
                  })}
                </Box>
              ) : (
                <Tooltip title="Click to copy">
                  <Typography
                    variant="caption"
                    onClick={() => copy(v.value, v.key)}
                    sx={{
                      display: "block",
                      mt: 0.5,
                      fontFamily: "monospace",
                      cursor: "pointer",
                      whiteSpace: "pre-wrap",
                      wordBreak: "break-all",
                      "&:hover": { color: "primary.main" },
                    }}
                  >
                    {v.value || <em>(empty)</em>}
                  </Typography>
                </Tooltip>
              )}
            </ListItem>
          )
        })}
        {shown.length === 0 && (
          <ListItem>
            <ListItemText
              primary="No matching variables"
              slotProps={{ primary: { variant: "body2", color: "text.disabled" } }}
            />
          </ListItem>
        )}
      </List>
    </Box>
  )
}
