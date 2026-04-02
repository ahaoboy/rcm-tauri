import { Menu, Group, IconItem, Item } from 'rcm';

// 1. Top icon tools (Cut, Copy, Rename, Share, Delete)
const topIconItems = [
  new IconItem('✂️', {
    match: (files) => files.length > 0,
    action: async (files) => console.log('Cut:', files.map(f => f.path)),
  }),
  new IconItem('📋', {
    match: (files) => files.length > 0,
    action: async (files) => console.log('Copy:', files.map(f => f.path)),
  }),
  new IconItem('📝', {
    match: (files) => files.length === 1,
    action: async (files) => console.log('Rename:', files[0].path),
  }),
  new IconItem('🔗', {
    match: (files) => files.length > 0,
    action: async (files) => console.log('Share:', files.map(f => f.path)),
  }),
  new IconItem('🗑️', {
    match: (files) => files.length > 0,
    admin: true, // Example usage of admin property
    action: async (files) => console.log('Delete:', files.map(f => f.path)),
  }),
];

// 2. Open actions
const openGroup = new Group([
  new Item('打开', {
    match: (files) => files.length > 0,
    action: async (files) => console.log('Open:', files),
  }),
  new Item('打开方式...', {
    match: (files) => files.length === 1 && !files[0].isDir,
    action: async (files) => console.log('Open with:', files[0].path),
  }),
  new Item('新建', {
    match: (files) => files.length === 0,
    items: [
      new Item('文件夹', {
        icon: '📁',
        action: async (_, dir) => console.log('New folder in:', dir),
      }),
      new Item('快捷方式', {
        icon: '🔗',
        action: async (_, dir) => console.log('New shortcut in:', dir),
      }),
      new Item('文本文档', {
        icon: '📄',
        action: async (_, dir) => console.log('New text document in:', dir),
      }),
    ],
  }),
]);

// 3. View / Sort
const viewGroup = new Group([
  new Item('查看', {
    match: (files) => files.length === 0,
    items: [
      new Item('大图标', { action: async () => console.log('Large icons') }),
      new Item('中等图标', { action: async () => console.log('Medium icons') }),
      new Item('小图标', { action: async () => console.log('Small icons') }),
    ],
  }),
  new Item('排序方式', {
    match: (files) => files.length === 0,
    items: [
      new Item('名称', { action: async () => console.log('Sort by name') }),
      new Item('大小', { action: async () => console.log('Sort by size') }),
      new Item('修改日期', { action: async () => console.log('Sort by date') }),
    ],
  }),
  new Item('刷新', {
    match: (files) => files.length === 0,
    action: async (_, dir) => console.log('Refresh:', dir),
  }),
]);

// 4. Advanced / Shell actions
const advancedGroup = new Group([
  new Item('压缩为 ZIP 文件', {
    icon: '🗜️',
    match: (files) => files.length > 0,
    action: async (files) => console.log('Compress:', files),
  }),
  new Item('复制文件地址', {
    icon: '📋',
    match: (files) => files.length > 0,
    action: async (files) => console.log('Copy path:', files.map(f => f.path)),
  }),
  new Item('在终端中打开', {
    icon: '💻',
    action: async (files, dir) => {
      const targetDir = files.length === 1 && files[0].isDir ? files[0].path : dir;
      console.log('Open terminal in:', targetDir);
    },
  }),
]);

// 5. Properties
const propertiesGroup = new Group([
  new Item('属性', {
    icon: '⚙️',
    action: async (files, dir) => {
      const target = files.length > 0 ? files.map(f => f.path) : dir;
      console.log('Properties:', target);
    },
  }),
]);

// Export the default Menu instance
export default new Menu(
  topIconItems,
  [openGroup, viewGroup, advancedGroup, propertiesGroup]
);
