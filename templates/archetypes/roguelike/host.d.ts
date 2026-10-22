// The host this project's playtest helpers (tests/*.ts) touch under bun:
// `node:` modules, `process`, `Buffer`, `TextEncoder`, `import.meta.url`.
// Declared here because the project type-checks with `types: []` (no
// @types/node, no bun-types): `og check` reads them through tsconfig.json's
// include. Game code (src/) uses none of them.

declare module "node:fs" {
  export function existsSync(path: string): boolean;
  export function readFileSync(path: string): Uint8Array;
  export function readFileSync(path: string, encoding: string): string;
  export function writeFileSync(path: string, data: string | Uint8Array): void;
  export function mkdirSync(path: string, options?: { recursive?: boolean }): void;
  export function copyFileSync(from: string, to: string): void;
}

declare module "node:path" {
  export function dirname(path: string): string;
  export function join(...parts: string[]): string;
  export function resolve(...parts: string[]): string;
}

declare module "node:url" {
  export function fileURLToPath(url: string | URL): string;
}

declare module "node:zlib" {
  export function inflateSync(data: Uint8Array): Uint8Array;
  export function deflateSync(data: Uint8Array, options?: { level?: number }): Uint8Array;
}

declare const process: {
  readonly env: Record<string, string | undefined>;
  cwd(): string;
  readonly stdout: { write(text: string): boolean };
};
declare const Buffer: { concat(chunks: readonly Uint8Array[]): Uint8Array };
declare class TextEncoder {
  encode(text: string): Uint8Array;
}
declare class URL {
  constructor(url: string, base?: string | URL);
  readonly href: string;
}
interface ImportMeta {
  readonly url: string;
}
