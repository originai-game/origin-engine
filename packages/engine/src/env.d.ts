// The few host globals the loaders touch, declared here so the runtime
// type-checks without @types/node or bun-types (game code never sees them).

declare module "node:module" {
  export function createRequire(url: string): (id: string) => unknown;
}

declare module "node:url" {
  export function pathToFileURL(path: string): URL;
  export function fileURLToPath(url: string | URL): string;
}

declare module "node:path" {
  export function resolve(...parts: string[]): string;
  export function dirname(path: string): string;
  export function join(...parts: string[]): string;
}

declare module "node:fs" {
  export function existsSync(path: string): boolean;
  export function readFileSync(path: string): Uint8Array;
  export function readFileSync(path: string, encoding: string): string;
  export function writeFileSync(path: string, data: string | Uint8Array): void;
  export function rmSync(path: string, options?: { recursive?: boolean; force?: boolean }): void;
}

declare module "bun:test" {
  type Body = () => unknown | Promise<unknown>;
  export function test(name: string, body: Body, timeout?: number): void;
  export function describe(name: string, body: () => void): void;
  export function beforeAll(body: Body): void;
  interface Matchers {
    toBe(value: unknown): void;
    toEqual(value: unknown): void;
    toBeTrue(): void;
    toBeFalse(): void;
    toBeGreaterThan(value: number): void;
    toBeLessThan(value: number): void;
    toMatch(pattern: RegExp | string): void;
    toContain(value: unknown): void;
    toThrow(pattern?: RegExp | string): void;
    not: Matchers;
  }
  export function expect(value: unknown): Matchers;
}

declare const process: {
  readonly env: Record<string, string | undefined>;
  readonly platform: string;
  readonly arch: string;
  readonly argv: string[];
  readonly version: string;
  exit(code?: number): never;
  readonly stdout: { write(text: string): boolean };
  readonly stderr: { write(text: string): boolean };
};

declare const Bun: { readonly version: string } | undefined;

interface ImportMeta {
  readonly dir?: string;
}
