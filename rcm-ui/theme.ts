/**
 * The MUI theme for the config window.
 *
 * Only the config editor (and its tabs) uses MUI. The menu windows are styled by
 * the user-editable `style.css`, so they must stay plain CSS — a Material theme
 * there would override the file users are meant to customise.
 */

import { alpha, createTheme, type Theme } from "@mui/material/styles"

import type { ThemeName } from "./api/menuEvents"

/**
 * Build the theme for one of RCM's theme settings.
 *
 * `dark` is passed as the *resolved* mode, so `system` never reaches here — the
 * config store already turned it into light or dark.
 */
export function createAppTheme(mode: "light" | "dark"): Theme {
  const dark = mode === "dark"
  // Neutral used for the scrollbar thumb: white on a dark surface, black on a
  // light one, at low alpha so it reads as chrome rather than content.
  const neutral = dark ? "#ffffff" : "#000000"

  return createTheme({
    palette: {
      mode,
      // VS Code's accent, so the editor window does not look like a Material demo
      // dropped into it. MUI derives hover/focus shades from this.
      primary: {
        main: dark ? "#4daafc" : "#0e639c",
        // Explicit rather than derived: this is the label colour on a filled
        // selected toggle, so it has to be legible against that exact fill.
        contrastText: dark ? "#0d1117" : "#ffffff",
      },
      error: { main: dark ? "#f44747" : "#c62828" },
      success: { main: dark ? "#4ec9b0" : "#1a7f6b" },
      // Not pure white in light mode: this is a full-window surface, and #fff
      // glares. The CodeMirror pane sets no background of its own, so it inherits
      // this too — and its gutter is #f5f5f5, which sits almost exactly here.
      background: {
        default: dark ? "#1e1e1e" : "#f6f6f6",
        // Unused today, but kept coherent: elevated surfaces stay a shade lighter
        // than the page rather than pure white.
        paper: dark ? "#252526" : "#fbfbfb",
      },
      divider: dark ? "#333333" : "#d6d6d6",
    },
    shape: { borderRadius: 4 },
    typography: {
      fontFamily: '"Segoe UI", system-ui, -apple-system, sans-serif',
      fontSize: 13,
    },
    components: {
      // `CssBaseline` owns the page reset, so it is also the right place for the
      // scrollbar: `::-webkit-scrollbar` is a pseudo-element and cannot live in
      // an `sx` prop.
      MuiCssBaseline: {
        styleOverrides: {
          // Without this the browser renders native widgets — most visibly the
          // scrollbar — for the light scheme, even on a dark page.
          ":root": { colorScheme: mode },
          "*::-webkit-scrollbar": { width: 8, height: 8 },
          // Transparent, so the container's own background shows through.
          "*::-webkit-scrollbar-track": { background: "transparent" },
          "*::-webkit-scrollbar-thumb": {
            background: alpha(neutral, 0.25),
            borderRadius: 4,
          },
          "*::-webkit-scrollbar-thumb:hover": { background: alpha(neutral, 0.4) },
        },
      },
      // Compact controls: the defaults are sized for a marketing site, not a
      // settings pane inside an editor window.
      MuiButton: {
        defaultProps: { disableElevation: true },
        styleOverrides: { root: { textTransform: "none", minWidth: 0 } },
      },
      MuiTab: {
        styleOverrides: { root: { textTransform: "none", minHeight: 40 } },
      },
      // The selected toggle is the one place a user has to read state at a
      // glance, so it gets a solid fill instead of MUI's default.
      //
      // The default is `alpha(text.primary, 0.16)` — a 16% white wash in dark
      // mode with the label unchanged, which is nearly indistinguishable from
      // unselected. Even `color="primary"` only tints that wash blue; a fill is
      // what actually reads as "active".
      MuiToggleButton: {
        styleOverrides: {
          root: ({ theme }) => {
            const { primary } = theme.palette
            return {
              textTransform: "none",
              "&.Mui-selected": {
                backgroundColor: primary.main,
                color: primary.contrastText,
                "&:hover": { backgroundColor: primary.dark },
              },
            }
          },
        },
      },
      MuiTooltip: {
        defaultProps: { arrow: true },
      },
      MuiTextField: {
        defaultProps: { size: "small", variant: "outlined" },
      },
    },
  })
}

/** Map RCM's theme setting onto a concrete MUI palette mode. */
export function modeFor(theme: ThemeName): "light" | "dark" {
  return theme === "light" ? "light" : "dark"
}
