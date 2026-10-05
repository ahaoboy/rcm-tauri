/**
 * FileEditor — owns a single CodeMirror editor instance.
 *
 * One instance is rendered per config file; each editor is created lazily on
 * first activation and kept alive afterwards (hidden via `display: none`) so
 * switching tabs preserves scroll position and undo history.
 */

import { indentWithTab } from "@codemirror/commands"
import { EditorState } from "@codemirror/state"
import { oneDark } from "@codemirror/theme-one-dark"
import { EditorView, keymap } from "@codemirror/view"
import { basicSetup } from "codemirror"
import React, { useEffect, useRef } from "react"

import { readConfigFile } from "../../api/menuEvents"
import { FILE_BY_KEY, LANG, type FileKey } from "./constants"

/**
 * Layout for the editor instance.
 *
 * The scrollbar chrome is not set here: `BodyReset` already styles every scroll
 * container in these pages (`*::-webkit-scrollbar`), and CodeMirror's scroller
 * is one of them. Setting it in both places duplicated the exact same values.
 */
const EDITOR_LAYOUT = EditorView.theme({
  "&": { height: "100%" },
  ".cm-scroller": { overflow: "auto" },
})

function createEditorState(
  doc: string,
  lang: string,
  isDark: boolean,
  onContentChange: (content: string) => void,
  onSave: () => void,
) {
  return EditorState.create({
    doc,
    extensions: [
      basicSetup,
      ...(isDark ? [oneDark] : []),
      LANG[lang]?.(),
      keymap.of([
        {
          key: "Ctrl-s",
          run: () => {
            onSave()
            return true
          },
        },
        {
          key: "Mod-s",
          run: () => {
            onSave()
            return true
          },
        },
        indentWithTab,
      ]),
      EditorView.updateListener.of((u) => {
        if (u.docChanged) onContentChange(u.state.doc.toString())
      }),
      EDITOR_LAYOUT,
    ],
  })
}

export const FileEditor: React.FC<{
  fileKey: FileKey
  active: boolean
  reloadKey: number
  isDark: boolean
  onContentChange: (content: string) => void
  onError: (msg: string) => void
  onLoaded: (originalContent: string) => void
  registerView: (key: FileKey, view: EditorView | null) => void
  triggerSave: () => void
}> = ({
  fileKey,
  active,
  reloadKey,
  isDark,
  onContentChange,
  onError,
  onLoaded,
  registerView,
  triggerSave,
}) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewRef = useRef<EditorView | null>(null)
  const createdRef = useRef(false)
  const lastReloadRef = useRef(0)

  // Create editor on first activation
  useEffect(() => {
    if (!active || createdRef.current) return
    const el = containerRef.current
    if (!el) return

    readConfigFile(fileKey)
      .then((data) => {
        if (viewRef.current) return
        viewRef.current = new EditorView({
          state: createEditorState(
            data,
            FILE_BY_KEY[fileKey].lang,
            isDark,
            onContentChange,
            triggerSave,
          ),
          parent: el,
        })
        registerView(fileKey, viewRef.current)
        createdRef.current = true
        onLoaded(data)
      })
      .catch((e) => onError(String(e)))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [active])

  // Reload: re-read file from disk, update editor content in-place.
  // Only fires when reloadKey actually increments (not on every parent re-render).
  useEffect(() => {
    if (reloadKey === 0 || reloadKey <= lastReloadRef.current || !createdRef.current) return
    lastReloadRef.current = reloadKey

    readConfigFile(fileKey)
      .then((data) => {
        const view = viewRef.current
        if (!view) {
          createdRef.current = false
          return
        }
        view.dispatch({
          changes: { from: 0, to: view.state.doc.length, insert: data },
        })
        onLoaded(data)
      })
      .catch((e) => onError(String(e)))
  }, [reloadKey, fileKey, onError, onLoaded])

  // Refresh layout when becoming visible (CodeMirror needs remeasure after display:none)
  useEffect(() => {
    if (active && viewRef.current) {
      // Delay to let the browser apply display:"" before measuring
      requestAnimationFrame(() => viewRef.current?.requestMeasure())
    }
  }, [active])

  // Cleanup on unmount
  useEffect(
    () => () => {
      viewRef.current?.destroy()
      registerView(fileKey, null)
      // eslint-disable-next-line react-hooks/exhaustive-deps
    },
    [],
  )

  return <div ref={containerRef} style={{ height: "100%", display: active ? "" : "none" }} />
}
