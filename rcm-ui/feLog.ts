/**
 * Frontend logging bridge — sends logs to Rust for centralised collection.
 *
 * Always calls console.log (for devtools visibility).
 * Also emits `log-event` to Rust so logs appear in the unified log file
 * when the "Log" tray toggle is enabled.
 */

import { emitLog } from "./api/menuEvents"

type LogLevel = "INFO" | "WARN" | "ERROR"

function send(tag: string, level: LogLevel, msg: string) {
  const formatted = `[${tag}] ${msg}`
  switch (level) {
    case "ERROR":
      console.error(formatted)
      break
    case "WARN":
      console.warn(formatted)
      break
    default:
      console.log(formatted)
  }
  // Send to Rust for file logging (fire-and-forget)
  emitLog(tag, `[${level}] ${msg}`)
}

export const feLog = {
  info(tag: string, msg: string) {
    send(tag, "INFO", msg)
  },
  warn(tag: string, msg: string) {
    send(tag, "WARN", msg)
  },
  error(tag: string, msg: string) {
    send(tag, "ERROR", msg)
  },
  /** Log an event being sent from frontend to Rust. */
  eventSend(eventName: string, detail: string) {
    send("EVENT:SEND", "INFO", `${eventName} | ${detail}`)
  },
  /** Log an event received from Rust. */
  eventRecv(eventName: string, detail: string) {
    send("EVENT:RECV", "INFO", `${eventName} | ${detail}`)
  },
}
