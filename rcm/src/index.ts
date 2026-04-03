/**
 * File information structure
 */
export type FileInfo = {
  name: string;
  path: string;
  isDir: boolean;
};

/**
 * Environmental context
 */
export type Env = Record<string, any>;

/**
 * Unified properties provided to menu items during evaluation
 */
export interface InvokeProps {
  files: FileInfo[];
  cwd: string;
  env: Env;
  admin: boolean;
  type: string;
}

export interface Command {
  exe: string;
  args?: string[];
  cwd?: string;
  admin?: boolean;
  window?: 'Hidden' | 'Show' | 'Visible' | 'Minimized' | 'Maximized';
}

export type MatchCallback = (props: InvokeProps) => boolean;
export type ActionCallback = (props: InvokeProps) => Command | void;

export interface MenuItem {
  key?: string;
  icon?: string;
  label?: string;
  disable?: boolean;
  admin?: boolean;
  window?: 'Hidden' | 'Show' | 'Visible' | 'Minimized' | 'Maximized';
  items?: MenuItem[];
  match?: MatchCallback;
  action?: ActionCallback;
  command?: Command;
}

export class Menu {
  public type = 'Menu';
  public iconItems: MenuItem[];
  public groups: MenuItem[];

  constructor(groups: MenuItem[] = [], iconItems: MenuItem[] = []) {
    this.iconItems = iconItems;
    this.groups = groups;
  }

  public invoke = (props: InvokeProps) => {
    const processItems = (items: MenuItem[]): MenuItem[] => {
      return items
        .filter(item => !item.match || item.match(props))
        .map(({ match, action, items, ...rest }) => {
          const command = action?.(props);
          return {
            ...rest,
            ...(command && { command }),
            ...(items && { items: processItems(items) })
          };
        })
        .filter(item => {
          // Clean up structural parent group nodes cleanly if all underlying menu elements were bypassed
          if (item.items && item.items.length === 0 && !item.label && !item.command) {
            return false;
          }
          return true;
        });
    };

    return {
      iconItems: processItems(this.iconItems),
      groups: processItems(this.groups),
    };
  }
}

export * from './vscode';
export * from './ssh';
