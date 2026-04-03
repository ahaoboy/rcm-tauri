import { Menu, InvokeProps } from './index';
import { findUniquePath } from 'rcm-sys';
import { vscode } from './vscode';

// =======================
// Practical Examples
// =======================

export default new Menu(
  [
    {
      items: [
        vscode('Open with VS Code'),
        {
          label: 'Extract Here (unzip)',
          icon: '📦',
          match: (props: InvokeProps) => {
            return props.files.length === 1 && props.files[0].name.toLowerCase().endsWith('.zip');
          },
          action: (props: InvokeProps) => {
            const file = props.files[0];
            const baseName = file.name.slice(0, -4);
            const extractDir = findUniquePath(props.cwd, baseName);

            return {
              exe: 'tar',
              args: ['-xf', file.path, '-C', extractDir],
              cwd: props.cwd,
              window: 'Hidden'
            };
          }
        }
      ]
    }
  ]
);
