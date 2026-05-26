import type { MenuItem, InvokeProps } from './types';

/**
 * Menu — holds two sections:
 * - `iconItems`: the icon ribbon at the top of Win11 context menu
 * - `groups`:  the main vertical list of menu items
 */
export class Menu {
  readonly type = 'Menu';

  constructor(
    public groups: MenuItem[] = [],
    public iconItems: MenuItem[] = [],
  ) {}

  /** Evaluate the menu against a set of props: filter by match, run actions, recurse. */
  invoke(props: InvokeProps): { iconItems: MenuItem[]; groups: MenuItem[] } {
    return {
      iconItems: this.#resolve(this.iconItems, props),
      groups: this.#resolve(this.groups, props),
    };
  }

  #resolve(items: MenuItem[], props: InvokeProps): MenuItem[] {
    const result: MenuItem[] = [];

    for (const { match, action, items: children, ...rest } of items) {
      if (match && !match(props)) continue;

      const resolved: MenuItem = { ...rest };

      const cmd = action?.(props);
      if (cmd) resolved.action = () => cmd;

      if (children) {
        const sub = this.#resolve(children, props);
        if (sub.length) resolved.items = sub;
        else if (!resolved.label && !resolved.action) continue;
      }

      result.push(resolved);
    }

    return result;
  }
}
