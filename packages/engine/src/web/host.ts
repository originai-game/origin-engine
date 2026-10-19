// The web host binding: the wasm-bindgen module of crates/og-host-web
// (origin-engine.js + origin-engine_bg.wasm, `--target web`). Column views
// are typed arrays over the module's `memory.buffer` at the byte offsets the
// host answers, so a system writes engine memory directly. Runs in the engine
// Worker (worker.ts) and anywhere else wasm runs (the cross-host test runs it
// under bun).

import { ARRAYS, bindHost, boundHost, type ColumnArray, type HostBinding } from "../runtime.ts";
import type { HostAbi } from "../generated/abi.ts";
import bindings from "../../types/bindings.json" with { type: "json" };

interface CallDecl {
  readonly symbol: string;
  readonly module: string;
  readonly returns: string;
  readonly publish: boolean;
}

/**
 * A publish build exports only the player-facing members (the commands
 * module and the probe's other members are compiled out). The runtime binds
 * every symbol, so each missing non-publish symbol answers its refusal: the
 * probe's `null` (or `false`), a command's `{ ok: false }`.
 */
export function withPublishRefusals(glue: Record<string, unknown>): Record<string, unknown> {
  const missing = (bindings.calls as CallDecl[]).filter((c) => !c.publish && typeof glue[c.symbol] !== "function");
  if (missing.length === 0) return glue;
  const out: Record<string, unknown> = { ...glue };
  const reason = JSON.stringify({ ok: false, reason: "not in a publish build: the command channel and the probe's development members are compiled out" });
  for (const c of missing) {
    const answer: unknown =
      c.returns === "bool" ? false
      : c.returns === "u32" || c.returns === "i32" ? 0
      : c.returns.startsWith("List<") ? "[]"
      : c.module === "probe" ? "null"
      : reason;
    out[c.symbol] = () => answer;
  }
  return out;
}

/** The wasm-bindgen glue module: its default export initialises it. */
export interface WebGlue extends Record<string, unknown> {
  default(input?: { module_or_path: unknown }): Promise<{ memory: WebAssembly.Memory }>;
  og_web_init(): void;
  /** The lanes variant only (wasm-bindgen-rayon): starts `threads` Workers sharing the module's memory. */
  initThreadPool?(threads: number): Promise<unknown>;
  /** The lanes variant only: lanes run on the pool from now on (crates/og-lanes). */
  og_lanes_ready?(threads: number): void;
}

/**
 * How long the lanes pool may take to start. wasm-bindgen-rayon waits for
 * each pool Worker's ready message with no timeout and no error handler, so
 * a Worker that never loads (a `worker-src` that refuses `blob:`, a failed
 * module import inside it) would otherwise hang the boot.
 */
export const LANES_POOL_DEADLINE_MS = 10_000;

/**
 * Runs `start` (the glue's `initThreadPool`) and settles when it does, when a
 * Worker it creates fires `error`, or at the deadline, whichever comes first.
 * While it runs, `globalThis.Worker` is wrapped to record the pool's Workers;
 * on a failure they are terminated. The pool can still be creating Workers
 * after a failure (wasm-bindgen-rayon fetches its script per Worker), so the
 * wrapper stays until `start` settles and terminates each late one; a pool
 * that never settles leaves it for the life of the engine Worker, which
 * starts no other Workers.
 */
async function startPool(start: () => Promise<unknown>, deadlineMs: number): Promise<void> {
  const scope = globalThis as { Worker?: typeof Worker };
  const Original = scope.Worker;
  if (typeof Original !== "function") {
    await start();
    return;
  }
  const workers: Worker[] = [];
  let abandoned = false;
  let fail: (e: Error) => void = () => {};
  const failure = new Promise<never>((_, reject) => (fail = reject));
  failure.catch(() => {});
  scope.Worker = new Proxy(Original, {
    construct(target, args, newTarget) {
      const w = Reflect.construct(target, args, newTarget) as Worker;
      if (abandoned) {
        w.terminate();
        return w;
      }
      workers.push(w);
      w.addEventListener("error", (e) => {
        const message = (e as ErrorEvent).message;
        fail(new Error(`a lanes pool Worker failed to load${message ? ` (${message})` : ""}`));
      });
      return w;
    },
  });
  const pool = Promise.resolve().then(start);
  const restore = () => {
    scope.Worker = Original;
  };
  let timer: ReturnType<typeof setTimeout> | undefined;
  const deadline = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new Error(`the lanes pool did not start within ${deadlineMs / 1000} s`)), deadlineMs);
  });
  try {
    await Promise.race([pool, failure, deadline]);
    restore();
  } catch (e) {
    abandoned = true;
    for (const w of workers) w.terminate();
    pool.then(restore, restore);
    throw e;
  } finally {
    clearTimeout(timer);
  }
}

/**
 * Initialise the glue with the wasm (a URL, a Response or the bytes) and bind
 * the runtime to it. With `threads`, the glue is the lanes variant and its
 * Worker pool starts first, within {@link LANES_POOL_DEADLINE_MS}; a failure
 * there (an error, a pool Worker's `error` event or the deadline) terminates
 * the pool's Workers and throws before the engine is told or anything is
 * bound, so the caller can discard this glue and load the single-threaded
 * variant instead.
 */
export async function loadWebHost(glue: WebGlue, wasm?: unknown, threads?: number): Promise<HostBinding> {
  const bound = boundHost();
  if (bound) return bound;
  const exports = await glue.default(wasm === undefined ? undefined : { module_or_path: wasm });
  if (threads !== undefined) {
    if (!glue.initThreadPool || !glue.og_lanes_ready) throw new Error("the module is not the lanes variant (no initThreadPool)");
    const init = glue.initThreadPool;
    await startPool(() => init(threads), LANES_POOL_DEADLINE_MS);
    glue.og_lanes_ready(threads);
  }
  glue.og_web_init();
  const memory = exports.memory;
  // Arrays by (offset, length, kind) over the current buffer: a system's
  // views cost no allocation while its columns stay put.
  let cache = new Map<string, ColumnArray>();
  let cachedBuffer: ArrayBufferLike = memory.buffer;
  const binding: HostBinding = {
    abi: withPublishRefusals(glue) as unknown as HostAbi<Uint32Array, unknown>,
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
