import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';
import { COPY_PATH, COPY_NAME, COPY_BASE64 } from '../../system-commands';

/**
 * Win11 "Copy as" submenu — provides different copy formats.
 *
 * Sub-items:
 *   • Copy as path  — full path(s) with Linux-style '/' separators
 *   • Copy as name  — file name(s) only
 *   • Copy as base64 — file content(s) encoded as base64
 */
export function copyAsPath(): MenuItem {
  const filesArg = (props: InvokeProps) => props.files.map(f => f.path);

  return {
    key: 'copy-as',
    label: t('copy.as'),
    icon: '📎',
    items: [
      {
        key: 'copy-as-path',
        label: 'path',
        icon: '📋',
        action: (props: InvokeProps) => ({
          exe: COPY_PATH,
          args: filesArg(props),
        }),
      },
      {
        key: 'copy-as-name',
        label: 'name',
        icon: '🏷️',
        action: (props: InvokeProps) => ({
          exe: COPY_NAME,
          args: filesArg(props),
        }),
      },
      {
        key: 'copy-as-base64',
        label: 'base64',
        icon: '🔐',
        action: (props: InvokeProps) => ({
          exe: COPY_BASE64,
          args: filesArg(props),
        }),
      },
    ],
  };
}
