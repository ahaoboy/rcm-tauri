import type { MenuItem, InvokeProps } from '../types';
import { t } from '../i18n';

/**
 * Win11 "Print" — sends the file to the default printer.
 */
export function print(): MenuItem {
  return {
    key: 'print',
    label: t('print'),
    icon: '🖨️',
    match: (props: InvokeProps) => {
      if (!props.files.length) return false;
      const name = props.files[0].name.toLowerCase();
      return /\.(txt|pdf|docx?|xlsx?|pptx?|jpg|jpeg|png|gif|bmp)$/.test(name);
    },
    action: (props: InvokeProps) => ({
      exe: 'print',
      args: [props.files[0].path],
      window: 'Show',
    }),
  };
}
