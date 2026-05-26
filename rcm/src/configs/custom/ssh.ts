import type { MenuItem, InvokeProps } from '../../types';
import { t } from '../../i18n';

/** Options for an SSH connection menu item. */
export interface SshOptions {
  host: string;
  port?: number;
  pwd?: string;
  identity?: string;
  tty?: boolean;
}

/**
 * "SSH Connect" menu item — opens an SSH session to a remote host.
 */
export function ssh(labelKey = 'ssh.connect', opts: SshOptions): MenuItem {
  return {
    key: `ssh-${opts.host}`,
    label: t(labelKey),
    icon: '🖥️',
    action: (props: InvokeProps) => {
      const args: string[] = [];

      if (opts.tty) args.push('-t');
      args.push('-p', String(opts.port ?? 22));
      if (opts.identity) args.push('-i', opts.identity);
      args.push(opts.host);

      if (opts.pwd) {
        return {
          exe: 'sshpass',
          args: ['-p', opts.pwd, 'ssh', ...args],
          cwd: props.cwd,
          window: 'Show',
        };
      }

      return { exe: 'ssh', args, cwd: props.cwd, window: 'Show' };
    },
  };
}
