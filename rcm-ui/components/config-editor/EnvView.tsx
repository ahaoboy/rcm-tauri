/**
 * EnvView — read-only inspector for the process environment variables.
 *
 * `*PATH`-style variables holding `;`-separated entries are split into a
 * numbered list for readability; every other value is shown on one line.
 * Values (or individual path entries) can be copied with a click.
 */

import React, { useCallback, useEffect, useMemo, useState } from "react"

import { getEnvVars } from "../../api/menuEvents"
import type { EnvVar } from "../../api/menuEvents"
import { styles } from "./styles"

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
    <div style={styles.envWrap}>
      <div style={styles.envBar}>
        <input
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder="Filter by name or value…"
          style={styles.envFilter}
        />
        <span style={styles.envCount}>
          {shown.length} / {vars.length}
        </span>
        <button onClick={load} style={styles.btn}>
          🔄 Refresh
        </button>
        {error && <span style={styles.err}>{error}</span>}
      </div>

      <div style={styles.envList}>
        {shown.map((v) => {
          const pathLike = isPathLike(v.key, v.value)
          const entries = pathLike ? v.value.split(";").filter(Boolean) : []
          return (
            <div key={v.key} style={styles.envRow}>
              <div style={styles.envHead}>
                <span style={styles.envKey}>{v.key}</span>
                {pathLike && <span style={styles.envBadge}>{entries.length} paths</span>}
                <button
                  onClick={() => copy(v.value, v.key)}
                  style={styles.envCopy}
                  title="Copy value"
                >
                  {copied === v.key ? "✓" : "⧉"}
                </button>
              </div>

              {pathLike ? (
                <ol style={styles.envPaths}>
                  {entries.map((p, i) => (
                    <li key={`${p}-${i}`} style={styles.envPathItem}>
                      <span style={styles.envPathIdx}>{i + 1}</span>
                      <span
                        style={styles.envPathText}
                        onClick={() => copy(p, `${v.key}#${i}`)}
                        title="Click to copy"
                      >
                        {copied === `${v.key}#${i}` ? "✓ copied" : p}
                      </span>
                    </li>
                  ))}
                </ol>
              ) : (
                <div
                  style={styles.envValue}
                  onClick={() => copy(v.value, v.key)}
                  title="Click to copy"
                >
                  {v.value || <span style={styles.envEmpty}>(empty)</span>}
                </div>
              )}
            </div>
          )
        })}
        {shown.length === 0 && <div style={styles.envEmptyBox}>No matching variables</div>}
      </div>
    </div>
  )
}
