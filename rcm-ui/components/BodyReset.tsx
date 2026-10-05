/**
 * BodyReset — injects a minimal CSS reset for standalone pages.
 * Use at the top of page components to kill browser defaults.
 */

import React from "react"

const CSS = `
html {
  /*
   * These pages are dark-only, so tell the browser that up front. This is what
   * makes native widgets — most visibly the scrollbar — render dark instead of
   * the default white. Without it a scroll container that has no explicit
   * \`::-webkit-scrollbar\` styling (such as the env inspector's list) gets a
   * light scrollbar on a dark surface.
   */
  color-scheme: dark;
}
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
/*
 * One scrollbar look for every scroll container in these pages. The values
 * match the CodeMirror editor's chrome so the editor and the surrounding panes
 * agree.
 *
 * This lives here rather than beside a component's styles because
 * \`::-webkit-scrollbar\` is a pseudo-element: it cannot be expressed in a React
 * inline \`style\` object, so it needs a real stylesheet.
 */
*::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}
*::-webkit-scrollbar-track {
  background: #1e1e1e;
}
*::-webkit-scrollbar-thumb {
  background: #424242;
  border-radius: 4px;
}
*::-webkit-scrollbar-thumb:hover {
  background: #555;
}
`

export const BodyReset: React.FC = () => <style>{CSS}</style>
