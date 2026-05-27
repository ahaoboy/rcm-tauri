import type { MenuItem, InvokeProps } from '../../types';
// @ts-ignore — rquickjs builtins
import * as fs from 'fs';
// @ts-ignore
import * as path from 'path';
// @ts-ignore
import * as os from 'os';

// rquickjs globals
declare function print(s: string): void;

interface WtProfile {
  name: string;
  commandline: string;
  guid?: string;
  icon?: string;
}

function readWtSshProfiles(): WtProfile[] {
  try {
    // @ts-ignore — os.homedir() in LLRT
    const home: string = os.homedir();
    const localAppData = path.join(home, 'AppData', 'Local');
    print('[ssh] localAppData: ' + localAppData);

    const wtBase = path.join(localAppData, 'Packages');
    let settingsPath = '';

    try {
      const dirs: string[] = fs.readdirSync(wtBase);
      const wtDir = dirs.find((d: string) => d.startsWith('Microsoft.WindowsTerminal'));
      if (wtDir) {
        settingsPath = path.join(wtBase, wtDir, 'LocalState', 'settings.json');
      }
    } catch {
      print('[ssh] failed to scan WT packages');
      return [];
    }

    if (!settingsPath) { print('[ssh] no settings.json found'); return []; }

    // LLRT readFileSync with encoding returns string directly
    const raw: string = fs.readFileSync(settingsPath, 'utf8');
    const cfg = JSON.parse(raw);
    const profiles: WtProfile[] = cfg?.profiles?.list ?? [];

    return profiles.filter(
      (p) => p.name && p.commandline && p.commandline.includes('ssh'),
    );
  } catch (e: any) {
    print('[ssh] error: ' + String(e.message || e));
    return [];
  }
}

export function ssh(): MenuItem {
  try {
    const profiles = readWtSshProfiles();
    print('[ssh] found ' + profiles.length + ' profiles');

    if (profiles.length === 0) {
      return { key: 'ssh', label: 'SSH', icon: '🖥️' };
    }

    return {
      key: 'ssh',
      label: 'SSH',
      icon: '🖥️',
      items: profiles.map((p) => ({
        key: `ssh-${p.name}`,
        label: p.name,
        icon: '🖥️',
        action: (_props: InvokeProps) => ({
          exe: 'wt',
          args: ['-w', '0', 'new-tab', '--profile', p.name],
          window: 'Show',
        }),
      })),
    };
  } catch (e: any) {
    print('[ssh] fatal: ' + String(e.message || e));
    return { key: 'ssh', label: 'SSH', icon: '🖥️' };
  }
}
