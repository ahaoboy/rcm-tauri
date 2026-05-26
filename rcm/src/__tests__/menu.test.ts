/**
 * Tests for rcm — Right-Click Menu toolkit.
 * Run with: pnpm test
 */
import { describe, it, expect } from 'vitest';

import { Menu, setLocale, getLocale, t } from '../index';
import type { MenuItem, InvokeProps } from '../index';
import * as win11Module from '../configs/win11/index';
import * as customModule from '../configs/custom/index';
import defaultMenu from '../default';

// ── Shared test props ──────────────────────────────────────────────
const makeProps = (overrides: Partial<InvokeProps> = {}): InvokeProps => ({
  files: [],
  cwd: 'C:\\Users\\test',
  env: {},
  admin: false,
  type: 'file',
  ...overrides,
});

// ═══════════════════════════════════════════════════════════════════
// Win11 default right-click menu data structure
// ═══════════════════════════════════════════════════════════════════

describe('Win11 default menu structure', () => {
  it('has correct top-level shape', () => {
    const result = defaultMenu.invoke(makeProps());
    expect(Array.isArray(result.iconItems)).toBe(true);
    expect(Array.isArray(result.groups)).toBe(true);
  });

  it('icon ribbon contains 6 standard Win11 items', () => {
    const result = defaultMenu.invoke(makeProps());
    const keys = result.iconItems.map(i => i.key);
    // Win11 icon bar: Cut, Copy, Paste, Rename, Share, Delete
    expect(keys).toEqual(['cut', 'copy', 'paste', 'rename', 'share', 'delete']);
  });

  it('main groups contain expected top-level keys', () => {
    const result = defaultMenu.invoke(makeProps({
      files: [{ name: 'readme.txt', path: 'C:\\readme.txt', isDir: false }],
    }));
    const allKeys = result.groups.flatMap(g => g.items?.map(i => i.key) ?? []);
    expect(allKeys).toEqual(expect.arrayContaining([
      'vscode', 'open', 'open-with', 'terminal', 'edit',
      'pin-to-start', 'send-to', 'copy-as-path', 'create-shortcut', 'properties',
    ]));
  });
});

// ═══════════════════════════════════════════════════════════════════
// Menu class
// ═══════════════════════════════════════════════════════════════════

describe('Menu', () => {
  it('constructs with empty defaults', () => {
    const m = new Menu();
    expect(m.type).toBe('Menu');
    expect(m.groups).toEqual([]);
    expect(m.iconItems).toEqual([]);
  });

  it('invoke filters items by match', () => {
    const menu = new Menu([
      {
        items: [
          { key: 'always', label: 'Always' },
          {
            key: 'only-txt',
            label: 'Only TXT',
            match: (p) => p.files.some(f => f.name.endsWith('.txt')),
          },
        ],
      },
    ]);

    const noMatch = menu.invoke(makeProps({ files: [] }));
    expect(noMatch.groups[0].items?.map(i => i.key)).toEqual(['always']);

    const match = menu.invoke(
      makeProps({ files: [{ name: 'readme.txt', path: '/readme.txt', isDir: false }] }),
    );
    expect(match.groups[0].items?.map(i => i.key)).toEqual(['always', 'only-txt']);
  });

  it('invoke runs action callbacks and attaches command', () => {
    const menu = new Menu([
      {
        items: [
          {
            key: 'run',
            label: 'Run',
            action: (p) => ({ exe: 'notepad', args: [p.files[0]?.path ?? ''], window: 'Show' }),
          },
        ],
      },
    ]);

    const result = menu.invoke(
      makeProps({ files: [{ name: 'a.txt', path: 'C:\\a.txt', isDir: false }] }),
    );
    const item = result.groups[0].items![0];
    expect(item.command).toBeTruthy();
    expect(item.command).toEqual({
      exe: 'notepad',
      args: ['C:\\a.txt'],
      window: 'Show',
    });
  });

  it('prunes empty structural parent groups', () => {
    const menu = new Menu([
      {
        items: [
          {
            // All children filtered out — parent has no label/command → pruned
            items: [
              { key: 'child', label: 'Child', match: () => false },
            ],
          },
        ],
      },
    ]);

    const result = menu.invoke(makeProps());
    const items = result.groups[0]?.items ?? [];
    expect(items.length).toBe(0);
  });

  it('recurses into nested submenus', () => {
    const menu = new Menu([
      {
        items: [
          {
            key: 'parent',
            label: 'Parent',
            items: [
              { key: 'child1', label: 'Child 1' },
              { key: 'child2', label: 'Child 2', match: () => false },
            ],
          },
        ],
      },
    ]);

    const result = menu.invoke(makeProps());
    const parent = result.groups[0].items![0];
    expect(parent.items?.length).toBe(1);
    expect(parent.items![0].key).toBe('child1');
  });

  it('iconItems are resolved independently from groups', () => {
    const menu = new Menu(
      [{ items: [{ key: 'main', label: 'Main' }] }],
      [{ key: 'icon1', label: 'Icon' }, { key: 'icon2', label: 'Icon', match: () => false }],
    );

    const result = menu.invoke(makeProps());
    expect(result.iconItems.length).toBe(1);
    expect(result.iconItems[0].key).toBe('icon1');
    expect(result.groups.length).toBe(1);
  });
});

// ═══════════════════════════════════════════════════════════════════
// i18n
// ═══════════════════════════════════════════════════════════════════

describe('i18n', () => {
  it('defaults to English', () => {
    expect(getLocale()).toBe('en');
    expect(t('open')).toBe('Open');
    expect(t('copy')).toBe('Copy');
    expect(t('paste')).toBe('Paste');
    expect(t('delete')).toBe('Delete');
  });

  it('switches to Chinese', () => {
    setLocale('zh');
    expect(getLocale()).toBe('zh');
    expect(t('open')).toBe('打开');
    expect(t('copy')).toBe('复制');
    expect(t('paste')).toBe('粘贴');
    expect(t('delete')).toBe('删除');
    setLocale('en'); // reset
  });

  it('falls back to English for missing keys', () => {
    setLocale('zh');
    expect(t('open.with.vscode')).toBe('通过 VS Code 打开');
    setLocale('en');
  });

  it('returns key itself if no translation exists', () => {
    expect(t('nonexistent.key.xyz')).toBe('nonexistent.key.xyz');
  });
});

// ═══════════════════════════════════════════════════════════════════
// Win11 config items — individual behaviors
// ═══════════════════════════════════════════════════════════════════

describe('Win11 config items', () => {
  describe('open', () => {
    it('has correct label and key', () => {
      const item = win11Module.open();
      expect(item.key).toBe('open');
      expect(item.label).toBe('Open');
    });

    it('produces a command when files are present', () => {
      const item = win11Module.open();
      const result = item.action!(makeProps({
        files: [{ name: 'test.txt', path: 'C:\\test.txt', isDir: false }],
      }));
      expect(result).toBeTruthy();
      expect((result as any).exe).toBe('cmd');
    });
  });

  describe('edit', () => {
    it('matches .txt files', () => {
      const item = win11Module.edit();
      expect(item.match!(makeProps({
        files: [{ name: 'readme.txt', path: '/r.txt', isDir: false }],
      }))).toBe(true);
    });

    it('does not match .exe files', () => {
      const item = win11Module.edit();
      expect(item.match!(makeProps({
        files: [{ name: 'app.exe', path: '/a.exe', isDir: false }],
      }))).toBe(false);
    });

    it('does not match empty files', () => {
      const item = win11Module.edit();
      expect(item.match!(makeProps({ files: [] }))).toBe(false);
    });
  });

  describe('runAsAdmin', () => {
    it('matches .exe files', () => {
      const item = win11Module.runAsAdmin();
      expect(item.match!(makeProps({
        files: [{ name: 'setup.exe', path: '/s.exe', isDir: false }],
      }))).toBe(true);
    });

    it('matches .bat/.cmd/.ps1 files', () => {
      const item = win11Module.runAsAdmin();
      for (const name of ['run.bat', 'run.cmd', 'run.ps1']) {
        expect(item.match!(makeProps({
          files: [{ name, path: `/${name}`, isDir: false }],
        }))).toBe(true);
      }
    });

    it('does not match .txt files', () => {
      const item = win11Module.runAsAdmin();
      expect(item.match!(makeProps({
        files: [{ name: 'doc.txt', path: '/d.txt', isDir: false }],
      }))).toBe(false);
    });
  });

  describe('sendTo', () => {
    it('has sub-items', () => {
      const item = win11Module.sendTo();
      expect(item.items).toBeTruthy();
      expect(item.items!.length).toBeGreaterThanOrEqual(3);
    });
  });

  describe('clipboard items', () => {
    it('cut/copy/paste/rename/delete have correct keys', () => {
      expect(win11Module.cut().key).toBe('cut');
      expect(win11Module.copy().key).toBe('copy');
      expect(win11Module.paste().key).toBe('paste');
      expect(win11Module.rename().key).toBe('rename');
      expect(win11Module.deleteItem().key).toBe('delete');
    });
  });
});

// ═══════════════════════════════════════════════════════════════════
// Custom config items
// ═══════════════════════════════════════════════════════════════════

describe('Custom config items', () => {
  describe('vscode', () => {
    it('opens current dir when no files selected', () => {
      const item = customModule.vscode();
      const result = item.action!(makeProps({ files: [] }));
      expect(result).toEqual({
        exe: 'code',
        args: ['.'],
        cwd: 'C:\\Users\\test',
        window: 'Hidden',
      });
    });

    it('opens selected files', () => {
      const item = customModule.vscode();
      const result = item.action!(makeProps({
        files: [
          { name: 'a', path: 'C:\\src\\a', isDir: false },
          { name: 'b', path: 'C:\\src\\b', isDir: false },
        ],
      }));
      expect(result).toEqual({
        exe: 'code',
        args: ['C:\\src\\a', 'C:\\src\\b'],
        cwd: 'C:\\Users\\test',
        window: 'Hidden',
      });
    });
  });

  describe('ssh', () => {
    it('builds basic ssh command', () => {
      const item = customModule.ssh('ssh.connect', { host: 'example.com' });
      const result = item.action!(makeProps());
      expect(result).toEqual({
        exe: 'ssh',
        args: ['-p', '22', 'example.com'],
        cwd: 'C:\\Users\\test',
        window: 'Show',
      });
    });

    it('uses sshpass when password provided', () => {
      const item = customModule.ssh('ssh.connect', { host: 'example.com', pwd: 'secret' });
      const result = item.action!(makeProps());
      expect((result as any).exe).toBe('sshpass');
      expect((result as any).args).toEqual(['-p', 'secret', 'ssh', '-p', '22', 'example.com']);
    });

    it('respects custom port and identity', () => {
      const item = customModule.ssh('ssh.connect', {
        host: 'example.com',
        port: 2222,
        identity: '~/.ssh/id_rsa',
        tty: true,
      });
      const result = item.action!(makeProps());
      expect((result as any).args).toEqual([
        '-t', '-p', '2222', '-i', '~/.ssh/id_rsa', 'example.com',
      ]);
    });
  });

  describe('mpv', () => {
    it('matches video/audio files', () => {
      const item = customModule.mpv();
      for (const ext of ['mkv', 'mp4', 'mp3', 'flac']) {
        expect(item.match!(makeProps({
          files: [{ name: `media.${ext}`, path: `/m.${ext}`, isDir: false }],
        }))).toBe(true);
      }
    });

    it('does not match non-media files', () => {
      const item = customModule.mpv();
      expect(item.match!(makeProps({
        files: [{ name: 'doc.txt', path: '/d.txt', isDir: false }],
      }))).toBe(false);
    });
  });
});

// ═══════════════════════════════════════════════════════════════════
// Snapshot: complete Win11 menu data structure
// ═══════════════════════════════════════════════════════════════════

describe('Win11 menu snapshot', () => {
  it('produces the expected Win11 data structure', () => {
    const result = defaultMenu.invoke(
      makeProps({
        files: [{ name: 'readme.txt', path: 'C:\\Users\\test\\readme.txt', isDir: false }],
      }),
    );

    // Strip runtime command data for structural snapshot
    const strip = (items: MenuItem[]): Partial<MenuItem>[] =>
      items.map(({ command: _c, action: _a, match: _m, items, ...rest }) => ({
        ...rest,
        ...(items ? { items: strip(items) } : {}),
      }));

    const snapshot = {
      iconItems: strip(result.iconItems),
      groups: result.groups.map(g => ({ items: strip(g.items ?? []) })),
    };

    // Verify the top-level structure
    expect(snapshot.iconItems.length).toBe(6);
    expect(snapshot.groups.length).toBe(4);

    // Verify icon ribbon keys in order
    const iconKeys = snapshot.iconItems.map(i => i.key);
    expect(iconKeys).toEqual(['cut', 'copy', 'paste', 'rename', 'share', 'delete']);

    // All items should have labels (i18n resolved)
    for (const group of snapshot.groups) {
      for (const item of group.items) {
        expect(item.label).toBeTruthy();
      }
    }
  });
});
