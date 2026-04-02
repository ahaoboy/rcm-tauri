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
export type Env = Record<string, string>;

/**
 * Unified properties provided to menu items during evaluation
 */
export interface InvokeProps {
  files: FileInfo[];
  cwd: string;
  env: Env;
  admin: boolean;
  type: string; // The namespace classification (e.g., 'File', 'Desktop', 'Directory')
}

/**
 * Callback function types mapped against unified properties
 */
export type MatchCallback = (props: InvokeProps) => boolean;
export type ActionCallback = (props: InvokeProps) => void;

/**
 * Basic configuration options for actionable items
 */
export interface ActionableOptions {
  key?: string; // Optional unique identifier for debugging and testing
  icon?: string;
  items?: Item[]; // Sub-menu items
  match?: MatchCallback; // Indicates if this item should be displayed
  action?: ActionCallback; // The callback triggered upon clicking
  disable?: boolean;
  admin?: boolean; // Indicates if the command should be run with administrator privileges
  window?: 'Hidden' | 'Show' | 'Visible' | 'Minimized' | 'Maximized'; // Controls execution window visibility
}

// ------------------------------------
// Core Return Snapshots
// ------------------------------------

export interface ItemSnapshot {
  type: string;
  key?: string;
  icon?: string;
  label?: string;
  disable?: boolean;
  admin?: boolean;
  window?: string;
  items?: ItemSnapshot[];
}

export interface GroupSnapshot {
  type: 'Group';
  items: ItemSnapshot[];
}

export interface MenuSnapshot {
  type: 'Menu';
  iconItems: ItemSnapshot[];
  groups: GroupSnapshot[];
}

// ------------------------------------
// Internal Models
// ------------------------------------

export abstract class MenuActionable {
  public key?: string;
  public icon?: string;
  public items?: Item[];
  public match?: MatchCallback;
  public action?: ActionCallback;
  public disable?: boolean;
  public admin?: boolean;
  public window?: 'Hidden' | 'Show' | 'Visible' | 'Minimized' | 'Maximized';

  constructor(options: ActionableOptions = {}) {
    this.key = options.key;
    this.icon = options.icon;
    this.items = options.items;
    this.match = options.match;
    this.action = options.action;
    this.disable = options.disable;
    this.admin = options.admin;
    this.window = options.window;
  }

  public isMatch(props: InvokeProps): boolean {
    if (this.match) {
      return this.match(props); // Delegate correctly to JS custom business logic boundaries
    }
    return true;
  }

  public execute(props: InvokeProps): void {
    if (this.action) {
      this.action(props);
    }
  }

  /**
   * Translates the node recursively evaluating contexts creating a sanitized data schema natively
   */
  public invoke(props: InvokeProps): ItemSnapshot {
    return {
      type: Object.getPrototypeOf(this).constructor.name,
      key: this.key,
      icon: this.icon,
      disable: this.disable,
      admin: this.admin,
      window: this.window,
      label: (this as any).label,
      items: this.items
        ? this.items.filter(i => i.isMatch(props)).map(i => i.invoke(props))
        : undefined,
    };
  }
}

export class Item extends MenuActionable {
  public label: string;

  constructor(label: string, options: ActionableOptions = {}) {
    super(options);
    this.label = label;
  }
}

export class IconItem extends MenuActionable {
  constructor(icon: string, options: Omit<ActionableOptions, 'icon'> = {}) {
    super({ ...options, icon });
  }
}

export class Group {
  public items: Item[];

  constructor(items: Item[] = []) {
    this.items = items;
  }

  public invoke(props: InvokeProps): GroupSnapshot {
    return {
      type: 'Group',
      items: this.items.filter(i => i.isMatch(props)).map(i => i.invoke(props)),
    };
  }
}

export class Menu {
  public iconItems: IconItem[];
  public groups: Group[];

  constructor(iconItems: IconItem[] = [], groups: Group[] = []) {
    this.iconItems = iconItems;
    this.groups = groups;
  }

  public invoke(props: InvokeProps): MenuSnapshot {
    return {
      type: 'Menu',
      iconItems: this.iconItems.filter(i => i.isMatch(props)).map(i => i.invoke(props)) as any,
      groups: this.groups.map(g => g.invoke(props)).filter(g => g.items.length > 0),
    };
  }
}
