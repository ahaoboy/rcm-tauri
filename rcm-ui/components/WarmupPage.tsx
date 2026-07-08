/** WarmupPage — notifies Rust once rendered. Rust closes the window. */
import { emit } from "@tauri-apps/api/event"
import React, { useEffect } from "react"

export const WarmupPage: React.FC = () => {
  useEffect(() => {
    emit("warmup-ready")
  }, [])
  return null
}
