import { Menu, Group, Item, InvokeProps } from './index';
import { run, findUniquePath } from 'rcm-sys';

// =======================
// Practical Examples
// =======================

const unzipItem = new Item('Extract Here (unzip)', {
  icon: '📦',
  match: (props: InvokeProps) => {
    // Only yield this element when precisely picking one `.zip` encoded archive natively
    return props.files.length === 1 && props.files[0].name.toLowerCase().endsWith('.zip');
  },
  action: (props: InvokeProps) => {
    const file = props.files[0];
    const baseName = file.name.slice(0, -4);
    
    // Interrogates filesystem generating trailing fallback indices successfully eliminating namespace collisions 
    const extractDir = findUniquePath(props.cwd, baseName);
    
    // Evaluate backend hooks spawning execution context cleanly
    run('tar', ['-xf', file.path, '-C', extractDir], { 
      cwd: props.cwd, 
      window: 'Hidden' 
    });
  }
});

const vscodeItem = new Item('Open with VS Code', {
  icon: '💻',
  action: (props: InvokeProps) => {
    if (props.files.length > 0) {
      const targetPaths = props.files.map(f => f.path);
      run('code', targetPaths, { cwd: props.cwd, window: 'Hidden' });
    } else {
      run('code', ['.'], { cwd: props.cwd, window: 'Hidden' });
    }
  }
});

export default new Menu(
  [], // iconItems
  [new Group([vscodeItem, unzipItem])] // groups
);
