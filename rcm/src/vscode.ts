import { MenuItem, InvokeProps } from './index';

/**
 * Creates a reusable VS Code menu item hook
 * @param label The text displayed on the context menu item
 * @returns A MenuItem statically mapped to execute the VSCode binaries locally
 */
export function vscode(label: string = 'code'): MenuItem {
  return {
    label,
    icon: '💻', // Uses standard native emoji glyph rendering
    action: (props: InvokeProps) => {
      if (props.files.length > 0) {
        // Targets selectively highlighted explicit paths natively
        const targetPaths = props.files.map(f => f.path);
        return { exe: 'code', args: targetPaths, cwd: props.cwd, window: 'Hidden' };
      } else {
        // Defaults to opening the ambient background active context boundary directory
        return { exe: 'code', args: ['.'], cwd: props.cwd, window: 'Hidden' };
      }
    }
  };
}
