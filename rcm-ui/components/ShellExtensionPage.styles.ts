import type React from "react"

/** Styles for the shell-extension diagnostic window. */
export const styles: Record<string, React.CSSProperties> = {
  // Full-viewport scroll container.
  //
  // `BodyReset` pins `body { overflow: hidden }` for the menu windows, so the
  // document itself can never scroll. This overlay takes that role instead: the
  // scrollbar sits at the window edge (page-level look) rather than inside a
  // nested box, and the whole report is read as one continuous flow.
  overlay: {
    display: "flex",
    height: "100vh",
    overflow: "auto",
    padding: 16,
    boxSizing: "border-box",
    fontFamily: "'Segoe UI', system-ui, sans-serif",
    background: "#1e1e1e",
    color: "#d4d4d4",
  },
  // `margin: auto` centres the card in both axes when the content is short, and
  // degrades to normal top-aligned scrolling when it is taller than the window
  // (flexbox `align-items: center` would clip the top instead).
  card: {
    display: "flex",
    flexDirection: "column",
    width: "100%",
    maxWidth: 640,
    margin: "auto",
    padding: "20px 24px",
    boxSizing: "border-box",
    borderRadius: 12,
    background: "#1e1e1e",
    textAlign: "center",
  },
  title: {
    fontSize: 17,
    fontWeight: 600,
    margin: "0 0 10px 0",
    color: "#f44747",
    flexShrink: 0,
  },
  okTitle: {
    fontSize: 17,
    fontWeight: 600,
    margin: "0 0 10px 0",
    color: "#4ec9b0",
    flexShrink: 0,
  },
  okMessage: { fontSize: 13, color: "#bbb", lineHeight: 1.5, margin: 0 },
  // Natural height — no inner scrollbar; the overlay scrolls instead.
  report: {
    margin: 0,
    padding: 12,
    borderRadius: 8,
    background: "#141414",
    border: "1px solid #333",
    color: "#aaa",
    fontSize: 12,
    lineHeight: 1.5,
    textAlign: "left",
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
    fontFamily: "Consolas, 'Cascadia Mono', monospace",
  },
  // Sticky so Retry stays reachable while the report is scrolled.
  actions: {
    position: "sticky",
    bottom: 0,
    display: "flex",
    alignItems: "center",
    gap: 10,
    marginTop: 14,
    paddingTop: 10,
    paddingBottom: 4,
    background: "#1e1e1e",
    flexShrink: 0,
  },
  // Always rendered (empty until the first retry) so the buttons keep their
  // position when the timestamp appears.
  status: { flex: 1, textAlign: "left", fontSize: 12, color: "#888" },
  retryBtn: {
    minWidth: 116,
    padding: "8px 20px",
    border: "1px solid #ff85a2",
    borderRadius: 6,
    background: "#3a2028",
    color: "#ff85a2",
    cursor: "pointer",
    fontSize: 13,
    fontFamily: "inherit",
    fontWeight: 600,
  },
  btn: {
    minWidth: 88,
    padding: "8px 20px",
    border: "1px solid #555",
    borderRadius: 6,
    background: "#3c3c3c",
    color: "#d4d4d4",
    cursor: "pointer",
    fontSize: 13,
    fontFamily: "inherit",
  },
}
