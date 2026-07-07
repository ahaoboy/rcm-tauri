/**
 * BodyReset — injects a minimal CSS reset for standalone pages.
 * Use at the top of page components to kill browser defaults.
 */

import React from "react"

const CSS = `
body {
  margin: 0;
  padding: 0;
  overflow: hidden;
  font-family: "Segoe UI", system-ui, -apple-system, sans-serif;
  font-size: 14px;
  -webkit-font-smoothing: antialiased;
}
*, *::before, *::after {
  box-sizing: border-box;
}
`

export const BodyReset: React.FC = () => <style>{CSS}</style>
