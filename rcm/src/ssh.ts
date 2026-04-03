import { MenuItem, InvokeProps, Command } from './index';

export interface SshOptions {
  host: string;
  port?: number | string;
  pwd?: string;
  identity?: string; // Maps to -i: Identity file (private key)
  tty?: boolean; // Maps to -t: Force pseudo-terminal allocation
}

/**
 * Creates a reusable SSH terminal connection menu item hook
 * @param label The textual label applied visually towards the user inside the context menu
 * @param options Connection parameters securely mapping endpoints and validations
 * @returns An executable SSH payload item configuration
 */
export function ssh(label: string, options: SshOptions): MenuItem {
  return {
    label,
    icon: '🖥️',
    action: (props: InvokeProps): Command => {
      const sshArgs: string[] = [];

      if (options.tty) {
        sshArgs.push('-t');
      }

      if (options.port) {
        sshArgs.push('-p', String(options.port));
      } else {
        sshArgs.push('-p', '22');
      }

      if (options.identity) {
        sshArgs.push('-i', options.identity);
      }

      // Target hostname
      sshArgs.push(options.host);

      if (options.pwd) {
        return {
          exe: 'sshpass',
          args: ['-p', options.pwd, 'ssh', ...sshArgs],
          cwd: props.cwd,
          window: 'Show'
        };
      }

      return {
        exe: 'ssh',
        args: sshArgs,
        cwd: props.cwd,
        window: 'Show'
      };
    }
  };
}
