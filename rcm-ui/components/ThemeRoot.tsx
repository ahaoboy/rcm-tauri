/**
 * ThemeRoot — applies the MUI theme to a window.
 *
 * Reads the resolved theme from the config store (the same source the settings
 * tab writes to), so changing the theme re-renders the whole window. That is what
 * makes the setting take effect in the window that hosts it, rather than only in
 * the menu popups.
 */

import CssBaseline from "@mui/material/CssBaseline"
import { ThemeProvider } from "@mui/material/styles"
import React, { useMemo } from "react"

import { useConfigStore } from "../stores"
import { createAppTheme, modeFor } from "../theme"

export const ThemeRoot: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const theme = useConfigStore((s) => s.theme)
  const muiTheme = useMemo(() => createAppTheme(modeFor(theme)), [theme])

  return (
    <ThemeProvider theme={muiTheme}>
      <CssBaseline />
      {children}
    </ThemeProvider>
  )
}
