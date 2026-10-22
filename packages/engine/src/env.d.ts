// The few host globals the loaders touch, declared here so the runtime
// type-checks without @types/node or bun-types (game code never sees them).

declare module "node:module" {
  export function createRequire(url: string): ((id: string) => unknown) & { resolve(id: string): string };
}

declare module "node:url" {
  export function pathToFileURL(path: string): URL;
  export function fileURLToPath(url: string | URL): string;
}

declare module "node:path" {
  export function resolve(...parts: string[]): string;
  export function dirname(path: string): string;
  export function basename(path: string, suffix?: string): string;
  export function join(...parts: string[]): string;
}

declare module "node:fs" {
  export function existsSync(path: string): boolean;
  export function readFileSync(path: string): Uint8Array;
  export function readFileSync(path: string, encoding: string): string;
  export function writeFileSync(path: string, data: string | Uint8Array): void;
  export function rmSync(path: string, options?: { recursive?: boolean; force?: boolean }): void;
  export function mkdirSync(path: string, options?: { recursive?: boolean }): void;
  export function copyFileSync(from: string, to: string): void;
  export function statSync(path: string): { mtimeMs: number; size: number; isDirectory(): boolean };
  export function watch(path: string, listener: (event: string, file: string | null) => void): { close(): void };
}

declare module "node:os" {
  export function networkInterfaces(): Record<string, Array<{ address: string; family: string | number; internal: boolean }> | undefined>;
  export function tmpdir(): string;
}

declare module "node:crypto" {
  export function createHash(algorithm: string): { update(data: Uint8Array | string): { digest(encoding: "hex"): string } };
}

declare module "node:zlib" {
  export function inflateSync(data: Uint8Array): Uint8Array;
  export function deflateSync(data: Uint8Array, options?: { level?: number }): Uint8Array;
  export function brotliCompressSync(data: Uint8Array | string, options?: { params?: Record<number, number> }): Uint8Array;
  export const constants: { BROTLI_PARAM_QUALITY: number; BROTLI_MAX_QUALITY: number; BROTLI_PARAM_SIZE_HINT: number };
}

declare module "bun:test" {
  type Body = () => unknown | Promise<unknown>;
  export function test(name: string, body: Body, timeout?: number): void;
  export namespace test {
    /** The test runs unless `condition` holds (then it is reported skipped). */
    function skipIf(condition: boolean): (name: string, body: Body, timeout?: number) => void;
  }
  export function describe(name: string, body: () => void): void;
  export function beforeAll(body: Body): void;
  export function afterAll(body: Body): void;
  interface Matchers {
    toBe(value: unknown): void;
    toEqual(value: unknown): void;
    toBeTrue(): void;
    toBeFalse(): void;
    toBeGreaterThan(value: number): void;
    toBeGreaterThanOrEqual(value: number): void;
    toBeLessThanOrEqual(n: number): void;
    toMatchObject(o: Record<string, unknown>): void;
    toBeLessThan(value: number): void;
    toBeCloseTo(value: number, digits?: number): void;
    toBeNull(): void;
    toBeDefined(): void;
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
  readonly pid: number;
  exit(code?: number): never;
  readonly stdout: { write(text: string): boolean };
  readonly stderr: { write(text: string): boolean };
};

declare const Bun: { readonly version: string } | undefined;

interface ImportMeta {
  readonly dir?: string;
  /** bun: whether this module is the process's entry. */
  readonly main?: boolean;
}

declare function atob(data: string): string;
declare function btoa(data: string): string;
