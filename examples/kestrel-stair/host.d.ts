// The host globals this project's bun scripts touch (tools/, tests/frames.ts),
// beside packages/engine/src/env.d.ts's node: modules: the example
// type-checks with `bun run typecheck` and no node_modules (FINDINGS #14 in
// examples/hollowbell). Game code (src/) uses none of them.

declare const console: { log(...values: unknown[]): void };
declare class TextEncoder {
  encode(text: string): Uint8Array;
}
declare class URL {
  constructor(url: string, base?: string | URL);
  readonly href: string;
}
declare const Buffer: { concat(chunks: readonly Uint8Array[]): Uint8Array };
interface ImportMeta {
  readonly url: string;
}

// tools/fetch-font.ts and tools/make-assets.ts (bun): the network and a directory listing.
declare function fetch(url: string): Promise<{ ok: boolean; status: number; arrayBuffer(): Promise<ArrayBuffer> }>;
declare module "node:fs" {
  export function readdirSync(path: string): string[];
}
