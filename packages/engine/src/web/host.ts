// The web host binding: the wasm-bindgen module of crates/og-host-web
// (origin-engine.js + origin-engine_bg.wasm, `--target web`). Column views
// are typed arrays over the module's `memory.buffer` at the byte offsets the
// host answers, so a system writes engine memory directly. Runs in the engine
// Worker (worker.ts) and anywhere else wasm runs (the cross-host test runs it
// under bun).

import { ARRAYS, bindHost, boundHost, type ColumnArray, type HostBinding } from "../runtime.ts";
import type { HostAbi } from "../generated/abi.ts";

/** The wasm-bindgen glue module: its default export initialises it. */
export interface WebGlue extends Record<string, unknown> {
  default(input?: { module_or_path: unknown }): Promise<{ memory: WebAssembly.Memory }>;
  og_web_init(): void;
}

/** Initialise the glue with the wasm (a URL, a Response or the bytes) and bind the runtime to it. */
export async function loadWebHost(glue: WebGlue, wasm?: unknown): Promise<HostBinding> {
  const bound = boundHost();
  if (bound) return bound;
  const exports = await glue.default(wasm === undefined ? undefined : { module_or_path: wasm });
  glue.og_web_init();
  const memory = exports.memory;
  // Arrays by (offset, length, kind) over the current buffer: a system's
  // views cost no allocation while its columns stay put.
  let cache = new Map<string, ColumnArray>();
  let cachedBuffer: ArrayBufferLike = memory.buffer;
  const binding: HostBinding = {
    abi: glue as unknown as HostAbi<Uint32Array, unknown>,
    name: "web",
    columns(raw, kinds, count) {
      const offsets = raw as Uint32Array;
      if (!offsets || offsets.length === 0) return null;
      if (cachedBuffer !== memory.buffer || cache.size > 4096) {
        cache = new Map();
        cachedBuffer = memory.buffer;
      }
      return Array.from(offsets, (offset, i) => {
        const kind = kinds[i]!;
        const key = `${offset}:${count * kind.stride}:${kind.array}`;
        let a = cache.get(key);
        if (!a) cache.set(key, (a = new ARRAYS[kind.array](memory.buffer, offset, count * kind.stride) as ColumnArray));
        return a;
      });
    },
    // A grown memory detaches every array over the old buffer.
    fresh: (array) => array.buffer === memory.buffer,
  };
  bindHost(binding);
  return binding;
}
