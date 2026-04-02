export interface RunOptions {
  cwd?: string;
  admin?: boolean;
  window?: 'Hidden' | 'Show' | 'Visible' | 'Minimized' | 'Maximized';
}

export declare function run(exe: string, args?: string[], options?: RunOptions): void;
export declare function which(exe: string): string | null;
export declare function where(exe: string): string | null;
export declare function findUniquePath(dir: string, name: string): string;