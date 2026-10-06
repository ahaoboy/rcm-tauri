/**
 * BodyReset — a minimal CSS reset for standalone pages.
 *
 * The config window no longer needs this: MUI's `CssBaseline` owns its reset and
 * palette. What remains here is for the windows that have no theme provider —
 * error, about and shell-extension — which would otherwise inherit browser
 * defaults on a dark surface.
 */

import React from "react"

const CSS = `
html {
  /*
   * Those windows are dark-only, so tell the browser up front. This is what makes
   * native widgets — most visibly the scrollbar — render dark rather than white.
   */
  color-scheme: dark;
  background: #1e1e1e;
}
body {
  margin: 0;
  padding: 0;
  overflow: hidden;
  font-family: "Segoe UI", system-ui, -apple-system, sans-serif;
  font-size: 14px;
  color: #d4d4d4;
  -webkit-font-smoothing: antialiased;
}
*, *::before, *::after {
  box-sizing: border-box;
}
`

export const BodyReset: React.FC = () => <style>{CSS}</style>
