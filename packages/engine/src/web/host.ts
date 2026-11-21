// The web host binding: the wasm-bindgen module of crates/og-host-web
// (origin-engine.js + origin-engine_bg.wasm, `--target web`). Column views
// are typed arrays over the module's `memory.buffer` at the byte offsets the
// host answers, so a system writes engine memory directly. Runs in the engine
// Worker (worker.ts) and anywhere else wasm runs (the cross-host test runs it
// under bun).

import { ARRAYS, bindHost, boundHost, type ColumnArray, type HostBinding } from "../runtime.ts";
import type { HostAbi } from "../generated/abi.ts";
import bindings from "../../types/bindings.json" with { type: "json" };
import { fetchModuleHost, instantiateSplit, type ModuleHost } from "./split.ts";

export { fetchModuleHost, loadEngineModules, loadedEngineModules, type ModuleHost } from "./split.ts";

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
 * variant instead. A split wasm (a publish build's: split.ts) loads its
 * modules through `modules`, by default fetched beside a wasm given as a URL.
 */
export async function loadWebHost(glue: WebGlue, wasm?: unknown, threads?: number, modules?: ModuleHost): Promise<HostBinding> {
  const bound = boundHost();
  if (bound) return bound;
  const host = modules ?? (typeof wasm === "string" || wasm instanceof URL ? fetchModuleHost(wasm) : null);
  const exports = await instantiateSplit(host, () => glue.default(wasm === undefined ? undefined : { module_or_path: wasm }));
  if (threads !== undefined) {
    if (!glue.initThreadPool || !glue.og_lanes_ready) throw new Error("the module is not the lanes variant (no initThreadPool)");
    const init = glue.initThreadPool;
    await startPool(() => init(threads), LANES_POOL_DEADLINE_MS);
    glue.og_lanes_ready(threads);
  }
  glue.og_web_init();
  const memory = exports.memory;
  const abi = withPublishRefusals(glue) as unknown as HostAbi<Uint32Array, unknown>;
  // The shared buffer's place in the module's memory: fixed, while the
  // buffer over the memory is replaced when it grows.
  const sharedOffset = abi.og_host_shared() as number;
  // Arrays by offset over the current buffer (a column's offset is one
  // field's, so its kind; the length is checked): a system's views cost no
  // allocation while its columns stay put.
  let cache = new Map<number, ColumnArray>();
  let cachedBuffer: ArrayBufferLike = memory.buffer;
  const binding: HostBinding = {
    abi,
    name: "web",
    shared: () => ({ buffer: memory.buffer, offset: sharedOffset }),
    columns(raw, kinds, count) {
      const offsets = raw as Uint32Array;
      if (!offsets || offsets.length === 0) return null;
      if (cachedBuffer !== memory.buffer || cache.size > 4096) {
        cache = new Map();
        cachedBuffer = memory.buffer;
      }
      return Array.from(offsets, (offset, i) => {
        const kind = kinds[i]!;
        const length = count * kind.stride;
        let a = cache.get(offset);
        if (!a || a.length !== length || a.constructor !== ARRAYS[kind.array]) {
          cache.set(offset, (a = new ARRAYS[kind.array](memory.buffer, offset, length) as ColumnArray));
        }
        return a;
      });
    },
    // A grown memory detaches every array over the old buffer.
    fresh: (array) => array.buffer === memory.buffer,
  };
  bindHost(binding);
  return binding;
}

/** The wasm exports the Worker uses beyond the binding symbols. */
export interface WebExtras {
  og_web_put_file(path: string, bytes: Uint8Array): void;
  og_web_fail_file(path: string, reason: string): void;
  /** A transcoder job's answer, by the key the engine gave it (og_web_set_transcoder): `""`, or why it was refused. */
  og_web_put_texture(key: string, width: number, height: number, rgba: Uint8Array, srgb: boolean): string;
  og_web_put_texture_encoded(key: string, format: string, width: number, height: number, srgb: boolean, levels: Uint8Array): string;
  /** The boot's cache fill: the game's bundle's Basis textures asked for through the texture hook; JSON `{ total, pending, formats }`. */
  og_web_prepare_textures(engine: number): string;
  og_web_set_fetcher(fetch: (path: string, integrity?: string) => void): void;
  /** Bytes received of a file the fetcher is fetching, for og.stream's and og.load's progress; absent in an older engine module. */
  og_web_file_progress?(path: string, received: number, total: number): void;
  /** The loading screen (og.loading): `report` for its stages and `done`, `idle` while the renderer has nothing waiting; absent in an older engine module. */
  og_web_set_loading?(report: (engine: number, stage: string, fraction: number, label: string) => void, idle: (engine: number) => boolean): void;
  /** Streamed video's decoders (video.ts); absent in an older engine module. */
  og_web_install_video?(
    stream: (key: number, width: number, height: number, codec: string, config: Uint8Array, table: Uint32Array, chunkFrames: number) => boolean,
    frame: (stream: number, player: number, frame: number, looped: boolean) => VideoFrame | null,
    chunk: (stream: number, n: number, data: Uint8Array | null) => void,
    decoder: string,
    hardware: boolean,
    maxWidth: number,
    maxHeight: number,
    codecs: string[],
  ): void;
  /** The engine's Basis transcoder: the only way a Basis texture reaches it (boot.ts installTranscoder). */
  og_web_set_transcoder(transcode: (key: string, ktx2: Uint8Array, target: string) => void): void;
  og_web_fail_texture(key: string, reason: string): void;
  og_web_set_inflater?(inflate: (key: number, bytes: Uint8Array) => void): void;
  og_web_put_inflated?(key: number, bytes: Uint8Array): void;
  og_web_fail_inflated?(key: number, reason: string): void;
  og_web_set_image_decoder?(decode: (key: number, bytes: Uint8Array) => void): void;
  /** RGBA8 rows as the file stores them, and the EXIF orientation it names for them (1: upright). */
  og_web_put_image?(key: number, width: number, height: number, rgba: Uint8Array, orientation: number): void;
  /** The EXIF orientation (1 to 8) a PNG, JPEG or WebP names: the native host's reader (og-runtime's exif_orientation). */
  og_web_image_orientation?(bytes: Uint8Array): number;
  og_web_fail_image?(key: number, reason: string): void;
  /** A document's http(s) image (og-runtime's world/remote.rs), answered with og_web_put_image or og_web_fail_image. */
  og_web_set_remote_fetcher?(fetch: (key: number, url: string, integrity: string) => void): void;
  /** A remote <iframe>'s document (og-runtime's world/remote.rs), answered with og_web_put_text or og_web_fail_text. */
  og_web_set_remote_text_fetcher?(fetch: (key: number, url: string, integrity: string) => void): void;
  og_web_put_text?(key: number, bytes: Uint8Array): void;
  og_web_fail_text?(key: number, reason: string): void;
  og_web_bundle_files(manifest: string): string;
  /** Each cooked file's integrity, `{ "<file>": "sha384-…" }` (or `{ error }`), for a package from an asset base. */
  og_web_bundle_integrity(manifest: string): string;
  /** `highDynamicRange`: the page's `(dynamic-range: high)`, which the Worker cannot ask (undefined: unknown). */
  og_web_attach_canvas(engine: number, canvas: OffscreenCanvas, lowPower: boolean, highDynamicRange?: boolean, cpuPassTimingOnly?: boolean): Promise<string>;
  /** What the browser granted the pointer lock: raw movement now, and whether it refused unadjustedMovement (og.input.mouse). */
  og_web_mouse_report?(engine: number, active: boolean, unsupported: boolean): void;
  og_web_set_viewport(engine: number, viewport: string): string;
  og_web_resize?(engine: number, width: number, height: number): void;
  og_web_lanes?(): string;
  /** What the engine holds (the probe's `memory`, ProbeMemory), as JSON. */
  og_web_memory?(engine: number): string;
  /** A development module's live edit: the files put since applied in place, without a hot reload (`{ ms, steps, warnings }` or `{ error }`). */
  og_web_dev_refresh?(engine: number): string;
  /** The device the attached renderer runs on: its WebGPU limits and features, whether it is a fallback adapter (JSON, null before attaching). */
  og_web_gpu?(engine: number): string;
  /** Make one pipeline no frame used yet (og-render's Renderer::warm); false when none is left. */
  og_web_warm?(engine: number): boolean;
  /** The work a played tick first reaches, without a tick (og_runtime::warm_tick): the UI's documents built on a copy, the height fields' chunk shapes. */
  og_web_warm_tick?(engine: number): void;
  /** Brackets the probe's `frame` with "scripted" and "display" (og-runtime's FrameKind): a harness's frame draws at the configured quality, never waits and is no sample of automatic quality. */
  og_web_frame_kind(engine: number, kind: "display" | "scripted" | "recorded"): void;
  /** Test hook: destroy the GPU device; the renderer is made again on a new one and the game goes on. */
  og_web_lose_device?(engine: number): boolean;
  /** Fork s60: `span(on)` around the renderer's frame pipelines (og-render set_frame_pipelines_hook), which pipelines.ts makes asynchronously. */
  og_web_frame_pipelines?(span: ((on: boolean) => void) | null): void;
  /** `make(on)` around each frame, the tickets handed out, a ticket's readiness and whether any compiles (pipelines.ts makeSpan, ticketCount, ready, anyCompiling; og-render gpu/ready.rs). */
  og_web_make_pipelines?(make: ((on: boolean) => void) | null, count: (() => number) | null, ready: ((ticket: number) => boolean) | null, compiling: (() => boolean) | null): void;
  /** Whether the last frame left work whose pipelines still compile for a later one (og-render gpu/ready.rs). */
  og_web_pipelines_held?(): boolean;
  /** Fork s63: the texture bytes a frame uploads at most (0: no limit). */
  og_web_upload_budget?(bytes: number): void;
  /** Fork s63: the textures still drawn from their stand-ins after the last frame. */
  og_web_textures_deferred?(): number;
  og_web_fonts_pending?(): number;
  /** Fork s70: the page's loading screen covers the canvas: frames prepare without drawing (og-render set_covered). */
  og_web_covered?(on: boolean): void;
  /** `og render`'s every-frame capture (development builds): the display frame drawn last as RGBA8 rows at the canvas's size; empty until its readback lands. */
  og_web_render_frame?(engine: number): Uint8Array;
  /** The first Rust panic recorded (JSON: kind, message, location, tick, lane), or `null`. */
  og_web_panic?(): string;
  /** Test hook: the next item at a lanes site panics (in a lane when the call splits). */
  og_web_lanes_fault?(site: string): boolean;
}
