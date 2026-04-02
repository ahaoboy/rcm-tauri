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
 * Callback function types
 */
export type MatchCallback = (files: FileInfo[], dir: string, env: Env) => boolean;
export type ActionCallback = (files: FileInfo[], dir: string, env: Env) => void | Promise<void>;

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
}

/**
 * Abstract base class for menu action items
 */
export abstract class MenuActionable {
  public key?: string;
  public icon?: string;
  public items?: Item[]; // Supports nested sub-menus
  public match?: MatchCallback;
  public action?: ActionCallback;
  public disable?: boolean;
  public admin?: boolean;

  constructor(options: ActionableOptions = {}) {
    this.key = options.key;
    this.icon = options.icon;
    this.items = options.items;
    this.match = options.match;
    this.action = options.action;
    this.disable = options.disable;
    this.admin = options.admin;
  }

  /**
   * Determine if this menu item should be shown in the current context
   */
  public isMatch(files: FileInfo[], dir: string, env: Env): boolean {
    if (this.match) {
      return this.match(files, dir, env);
    }
    return true; // Default to visible
  }

  /**
   * Execute the action after clicking the menu item
   */
  public async execute(files: FileInfo[], dir: string, env: Env): Promise<void> {
    if (this.action) {
      await this.action(files, dir, env);
    }
  }

  /**
   * Custom JSON serialization.
   * Enables JSON.stringify() to produce a clean representation of the menu model structure.
   */
  public toJSON(): Record<string, any> {
    return {
      type: Object.getPrototypeOf(this).constructor.name,
      key: this.key,
      icon: this.icon,
      disable: this.disable,
      admin: this.admin,
      items: this.items ? this.items.map(i => i.toJSON()) : undefined,
    };
  }
}

/**
 * Item: A concrete clickable menu item
 */
export class Item extends MenuActionable {
  public label: string;

  constructor(label: string, options: ActionableOptions = {}) {
    super(options);
    this.label = label;
  }

  public toJSON(): Record<string, any> {
    return {
      ...super.toJSON(),
      label: this.label,
    };
  }
}

/**
 * IconItem: A special menu item displayed as an icon at the top of the Menu
 */
export class IconItem extends MenuActionable {
  constructor(icon: string, options: Omit<ActionableOptions, 'icon'> = {}) {
    super({ ...options, icon });
  }
}

/**
 * Group: A group of Items (visually separated in the UI)
 */
export class Group {
  public items: Item[];

  constructor(items: Item[] = []) {
    this.items = items;
  }

  public toJSON(): Record<string, any> {
    return {
      type: 'Group',
      items: this.items.map(i => i.toJSON()),
    };
  }
}

/**
 * Menu: The core model for the context menu
 */
export class Menu {
  public iconItems: IconItem[];
  public groups: Group[];

  constructor(iconItems: IconItem[] = [], groups: Group[] = []) {
    this.iconItems = iconItems;
    this.groups = groups;
  }

  /**
   * Enables direct parsing into JSON via JSON.stringify()
   */
  public toJSON(): Record<string, any> {
    return {
      type: 'Menu',
      iconItems: this.iconItems.map(i => i.toJSON()),
      groups: this.groups.map(g => g.toJSON()),
    };
  }
}