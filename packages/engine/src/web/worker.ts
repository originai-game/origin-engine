// The engine Worker (docs/design.md §2.2): the wasm instance, the renderer on
// the OffscreenCanvas the shell transferred, and the game bundle. A game's
// worker entry is two lines (og build writes it):
//
//   import { startWorker } from "@origingame/engine/web/worker";
//   import game from "./game.ts";
//   startWorker(game);
//
// The shell (shell.ts) sends `start`, then `call`s. With `root`, the Worker
// fetches the game's cooked bundle before the engine starts (the manifest,
// then every blob in parallel) and hands it to the wasm; once the renderer is
// attached it transcodes the Basis textures with the vendored transcoder
// (basis.ts) to the block format the renderer takes for each kind (BC7/BC5,
// ASTC 4x4 or ETC2/EAC, else RGBA8), so every load of the bundle completes at
// once, before the game starts; later files are fetched on request.
// Probe frames arrive through the request ring (rpc-ring.ts) when the page is
// cross-origin isolated, else as `rpc` messages; input arrives through the
// input ring, else as `input` messages. Ticks follow the display frame
// (requestAnimationFrame in the Worker) unless the shell drives them.
//
// Messages to the shell: { type: "progress", value: 0..1, stage, loaded,
// total } (bytes of the wasm and the cooked bundle actually received, so the
// value is real and never goes backwards), then { type: "ready", engine,
// tier, transport, rpc, textureFormat, files, textures } or { type: "error",
// message } before ready; { type: "frame", first } after the first rendered
// frame; { type: "crash", message, error } when the engine fails after
// ready (a trap, a Rust panic on the engine's thread or in a lane, a failing
// system, an uncaught error; `error` is the structured HostError of shell.ts:
// kind, message, tick, lane, location); { type: "beat", tick } about once a
// second while frames play, which the shell's frame deadline watches;
// { type: "log", lines } with
// the engine's log and the Worker's console (the crash card shows the last
// 200); { type: "text", target } when the UI's focused text control changes
// and { type: "copy", text } when a copy took text (text-input.ts); { type: "a11y", tree } when the UI's
// accessibility tree changed (a11y.ts); { type: "result", id, value | error } for `call` (`advance`,
// `command`, `hash`, `frame`, `probe`, and `timings`: the played frames'
// display intervals and engine work); { type: "rpc",
// frame } for an answer that did not cross the ring.
//
// Requests on the ring are served between frames by the request loop
// whether or not the clock is playing (a page whose display frames stop, a
// hidden tab or frame, still answers), and the loop always waits for the
// next request instead of spinning: a waiting request used to starve the
// Worker's animation frames while the clock played, so isolated pages never
// answered.

import { createEngine, systemClock, type Engine, type GameToken } from "../runtime.ts";
import type { PlatformAnswer, PlatformHost } from "../platform-local.ts";
import { loadWebHost, type WebGlue } from "./host.ts";
import { drainRing, stampAt, type InjectedInput } from "./input.ts";
import { WorkerAudio, type AudioLatencyMessage, type AudioMessage, type WebAudioGlue } from "./audio.ts";
import { readFrames, waitForFrames, writeFrame, type RpcRings } from "./rpc-ring.ts";
import { loadBasisTranscoder, transcodeKtx2, TEXTURE_FORMATS, type BasisTranscoder, type TextureFormat } from "./basis.ts";
import type { HostError } from "./shell.ts";

interface Start {
  type: "start";
  glue: string;
  wasm: string;
  /**
   * The lanes variant (the atomics build, its own glue and module) and the
   * Worker pool's threads, when the shell chose lanes (laneChoice in
   * shell.ts); without it, or when it fails to start, `glue`/`wasm` run
   * single-threaded.
   */
  lanes?: { glue: string; wasm: string; threads: number; bytes?: number };
  /** Why the shell did not choose lanes, reported back in ready. */
  lanesWhy?: string;
  canvas?: OffscreenCanvas;
  webgl?: boolean;
  seed?: number;
  ring?: SharedArrayBuffer;
  rpc?: RpcRings;
  autoplay?: boolean;
  /** The URL base whose `cooked/` bundle the engine opens (`.` next to the page). */
  root?: string;
  /** The Basis transcoder's script and module. */
  basis?: { js: string; wasm: string };
  /** The project's physics profile (project.ogt `profile`). */
  profile?: "2d" | "3d";
  /** The locale og.i18n is asked for (the page's `navigator.language`). */
  locale?: string;
  /** The play surface in CSS pixels, its DPR and safe area: what the UI lays out in. */
  viewport?: Viewport;
  /** Bytes the boot will fetch (the wasm, the cooked bundle), for real progress; omitted: counted as they arrive. */
  expect?: { wasm?: number; cooked?: number };
  /** Without autoplay: take the first tick before the first frame, so the frame behind a play gate shows the game's first screen (its UI is built by ticks). */
  prime?: boolean;
}

type Viewport = { width: number; height: number; dpr: number; safeArea?: Record<string, number>; coarsePointer?: boolean; reducedMotion?: boolean; highContrast?: boolean };

interface Call {
  type: "call";
  id: number;
  method: string;
  args: unknown[];
}

type Message =
  | Start
  | Call
  | { type: "input"; event: InjectedInput }
  | { type: "play"; on: boolean }
  | { type: "rpc"; frame: string }
  | { type: "viewport"; viewport: Viewport }
  | { type: "resize"; width: number; height: number; viewport: Viewport }
  | { type: "platform-answer"; request: number; answer: PlatformAnswer }
  | { type: "platform-host"; host: string; capabilities: string[] }
  | AudioMessage
  | AudioLatencyMessage;

/** og.platform's host in the Worker: the calls go to the page, whose answers come back as `platform-answer`. */
function toPage(host: string, capabilities: readonly string[]): PlatformHost {
  return { host, capabilities, handle: (calls) => self.postMessage({ type: "platform", calls }) };
}

interface WorkerScope {
  onmessage: ((e: { data: Message }) => void) | null;
  addEventListener?(type: string, fn: (e: { message?: string; error?: unknown; reason?: unknown; preventDefault?(): void }) => void): void;
  postMessage(message: unknown): void;
  requestAnimationFrame?(fn: (t: number) => void): number;
}

declare const self: WorkerScope;

/** The wasm exports the Worker uses beyond the binding symbols. */
export interface WebExtras {
  og_web_put_file(path: string, bytes: Uint8Array): void;
  og_web_fail_file(path: string, reason: string): void;
  og_web_put_texture(id: string, width: number, height: number, rgba: Uint8Array, srgb: boolean): void;
  og_web_put_texture_encoded(id: string, format: string, width: number, height: number, srgb: boolean, levels: Uint8Array): string;
  og_web_texture_target(engine: number, manifest: string, id: string, bytes: Uint8Array): string;
  og_web_set_fetcher(fetch: (path: string) => void): void;
  og_web_set_inflater?(inflate: (key: number, bytes: Uint8Array) => void): void;
  og_web_put_inflated?(key: number, bytes: Uint8Array): void;
  og_web_fail_inflated?(key: number, reason: string): void;
  og_web_bundle_files(manifest: string): string;
  og_web_texture_ktx2(manifest: string, id: string, bytes: Uint8Array): Uint8Array;
  og_web_pack_ktx2?(engine: number | undefined, manifest: string, pack: Uint8Array): Array<string | Uint8Array>;
  og_web_attach_canvas(engine: number, canvas: OffscreenCanvas, webgl: boolean): Promise<string>;
  og_web_set_viewport(engine: number, viewport: string): string;
  og_web_resize?(engine: number, width: number, height: number): void;
  og_web_lanes?(): string;
  /** The first Rust panic recorded (JSON: kind, message, location, tick, lane), or `null`. */
  og_web_panic?(): string;
  /** Test hook: the next item at a lanes site panics (in a lane when the call splits). */
  og_web_lanes_fault?(site: string): boolean;
}

/** Where the runtime asks for a bundle's files: `<root>/cooked/<file>` (crates/og-runtime/src/assets.rs). */
export function cookedDir(root: string): string {
  return `${root.replace(/\/+$/, "")}/cooked`;
}

/** Bytes received so far by the boot, against what it expects. */
export class Received {
  loaded = 0;
  total: number;
  constructor(total = 0) {
    this.total = total;
  }
  /** Fraction in [0, 1]; grows the total when more arrived than expected. */
  get value(): number {
    if (this.loaded > this.total) this.total = this.loaded;
    return this.total > 0 ? this.loaded / this.total : 0;
  }
}

/**
 * How long one file's fetch may take, its body included, before it fails:
 * generous, since a pack can be large on a slow network; only a fetch that
 * hangs reaches it.
 */
export const FETCH_DEADLINE_MS = 300_000;

async function bytesOf(url: string, received?: Received, onChunk?: () => void): Promise<Uint8Array> {
  const r = await fetch(url, typeof AbortSignal?.timeout === "function" ? { signal: AbortSignal.timeout(FETCH_DEADLINE_MS) } : {});
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  if (!received || !r.body) {
    const bytes = new Uint8Array(await r.arrayBuffer());
    if (received) received.loaded += bytes.length;
    onChunk?.();
    return bytes;
  }
  const reader = r.body.getReader();
  const chunks: Uint8Array[] = [];
  let length = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    length += value.length;
    received.loaded += value.length;
    onChunk?.();
  }
  const out = new Uint8Array(length);
  let at = 0;
  for (const c of chunks) {
    out.set(c, at);
    at += c.length;
  }
  return out;
}

export interface Preloaded {
  readonly files: number;
  readonly failed: number;
  /** Basis textures transcoded and handed to the wasm. */
  readonly textures: number;
  /**
   * The encodings they reached the renderer in, joined by `+` in the order
   * first used (`bc7+bc5` on desktop, `astc4x4` on Apple and most Android,
   * `etc2+eacrg11` elsewhere, `rgba8` without a renderer or compression).
   */
  readonly textureFormat: string;
}

/** A fetched bundle whose Basis textures wait for the renderer's formats. */
export interface Fetched {
  readonly files: number;
  readonly failed: number;
  readonly manifest: string | null;
  /** Texture blobs by asset id. */
  readonly textures: ReadonlyArray<{ readonly id: string; readonly bytes: Uint8Array }>;
  /** Outputs of streaming groups, left for their packs (fetched when a group loads). */
  readonly streamed: number;
}

/**
 * Fetch the bundle under `root` into the wasm: the manifest and every blob
 * of the first group (everything no streaming group holds, the proxies
 * included). Its textures are transcoded by `transcodeBundle` once the
 * renderer is attached, to the encoding the renderer takes. A streaming
 * group's outputs are not fetched: the engine reads its pack
 * (`cooked/stream/<name>.ogb`) through the fetcher when the group loads;
 * nor are the chunks of a sound cooked in chunks, which the engine reads
 * through the fetcher ahead of the playheads. The engine asks for at most
 * four such files at a time (og-runtime's `stream::fetch`): the chunks a
 * playhead needs soonest first, then the packs by their group's priority.
 */
export async function preloadBundle(
  glue: WebExtras,
  root: string,
  progress: (value: number, stage: string) => void,
  received?: Received,
): Promise<Fetched> {
  const dir = cookedDir(root);
  const manifestPath = `${dir}/.cook.ogt`;
  let manifest: string;
  try {
    manifest = new TextDecoder().decode(await bytesOf(manifestPath));
  } catch (e) {
    glue.og_web_fail_file(manifestPath, `cannot fetch ${manifestPath} (${e instanceof Error ? e.message : String(e)}); run og cook and serve cooked/ next to the page`);
    return { files: 0, failed: 1, manifest: null, textures: [], streamed: 0 };
  }
  glue.og_web_put_file(manifestPath, new TextEncoder().encode(manifest));
  const all = JSON.parse(glue.og_web_bundle_files(manifest)) as Array<{ id: string; file: string; type: string; group?: string }> | { error: string };
  if (!Array.isArray(all)) throw new Error(`${manifestPath}: ${all.error}`);
  // A long sound's chunks (og.SoundChunk) are fetched as its playheads near them.
  const listed = all.filter((o) => o.group === undefined && o.type !== "og.SoundChunk");
  let fetched = 0;
  let failed = 0;
  const blobs = new Map<string, Uint8Array>();
  await Promise.all(
    listed.map(async (o) => {
      const path = `${dir}/${o.file}`;
      try {
        const bytes = await bytesOf(path, received, received ? () => progress(received.value, "fetch") : undefined);
        glue.og_web_put_file(path, bytes);
        blobs.set(o.id, bytes);
      } catch (e) {
        failed++;
        glue.og_web_fail_file(path, `cannot fetch ${path} (${e instanceof Error ? e.message : String(e)})`);
      }
      ++fetched;
      if (!received) progress(0.1 + 0.7 * (fetched / Math.max(1, listed.length)), "fetch");
    }),
  );
  const textures = listed.filter((o) => o.type === "og.Texture" && blobs.has(o.id)).map((o) => ({ id: o.id, bytes: blobs.get(o.id)! }));
  return { files: listed.length - failed, failed, manifest, textures, streamed: all.length - listed.length };
}

/**
 * Transcode a fetched bundle's Basis textures for `engine`'s renderer
 * (`null`: none attached, RGBA8): each to the encoding the wasm names for
 * its kind on this device (`og_web_texture_target`: BC7 and BC5 on desktop
 * GPUs, ASTC 4x4 on Apple and most Android, ETC2 and EAC RG11 elsewhere,
 * RGBA8 last), every mip level handed over; RGBA8 hands level 0 and the GPU
 * builds the mips.
 */
export async function transcodeBundle(
  glue: WebExtras,
  engine: number | null,
  fetched: Fetched,
  basis: { js: string; wasm: string } | undefined,
  progress: (value: number, stage: string) => void,
  received?: Received,
): Promise<Preloaded> {
  const used: string[] = [];
  let transcoder: BasisTranscoder | null = null;
  let transcoded = 0;
  const manifest = fetched.manifest;
  for (const [i, t] of fetched.textures.entries()) {
    const ktx2 = manifest === null ? new Uint8Array() : glue.og_web_texture_ktx2(manifest, t.id, t.bytes);
    if (ktx2.length > 0) {
      transcoder ??= basis ? await loadBasisTranscoder(basis.js, basis.wasm) : await loadBasisTranscoder();
      const target = engine === null ? "rgba8" : glue.og_web_texture_target(engine, manifest!, t.id, t.bytes);
      const out = transcodeKtx2(transcoder, ktx2, isTextureFormat(target) ? target : "rgba8");
      if (out.format === "rgba8") {
        glue.og_web_put_texture(t.id, out.width, out.height, out.levels[0]!, out.srgb);
      } else {
        const refused = glue.og_web_put_texture_encoded(t.id, out.format, out.width, out.height, out.srgb, concat(out.levels));
        if (refused) throw new Error(refused);
      }
      if (!used.includes(out.format)) used.push(out.format);
      transcoded++;
    }
    if (!received) progress(0.8 + 0.2 * ((i + 1) / fetched.textures.length), "transcode");
  }
  return { files: fetched.files, failed: fetched.failed, textures: transcoded, textureFormat: used.length ? used.join("+") : "rgba8" };
}

function isTextureFormat(name: string): name is TextureFormat {
  return (TEXTURE_FORMATS as readonly string[]).includes(name);
}

function concat(parts: readonly Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

/**
 * What the engine's fetcher does with a file it asks for after the boot: a
 * streaming group's pack has its Basis textures transcoded, to the encoding
 * `engine()`'s renderer takes for each (RGBA8 without one), and handed over
 * first, so that every texture in it is ready when the engine reads the
 * pack; any other file is handed over as fetched.
 */
export function installFetcher(glue: WebExtras, manifest: string, basis: { js: string; wasm: string } | undefined, engine: () => number | null): void {
  let transcoder: Promise<BasisTranscoder> | null = null;
  const put = async (path: string, bytes: Uint8Array) => {
    const triples = /\/stream\/[^/]+\.ogb$/.test(path) && glue.og_web_pack_ktx2 ? glue.og_web_pack_ktx2(engine() ?? undefined, manifest, bytes) : [];
    for (let i = 0; i + 2 < triples.length; i += 3) {
      let loading = transcoder;
      if (!loading) {
        const started = basis ? loadBasisTranscoder(basis.js, basis.wasm) : loadBasisTranscoder();
        transcoder = loading = started;
        // A load that failed is not kept: the next pack tries again.
        started.catch(() => {
          if (transcoder === started) transcoder = null;
        });
      }
      const id = triples[i] as string;
      const target = triples[i + 2] as string;
      const out = transcodeKtx2(await loading, triples[i + 1] as Uint8Array, isTextureFormat(target) ? target : "rgba8");
      if (out.format === "rgba8") {
        glue.og_web_put_texture(id, out.width, out.height, out.levels[0]!, out.srgb);
      } else {
        const refused = glue.og_web_put_texture_encoded(id, out.format, out.width, out.height, out.srgb, concat(out.levels));
        if (refused) throw new Error(refused);
      }
    }
    glue.og_web_put_file(path, bytes);
  };
  glue.og_web_set_fetcher((path: string) => {
    bytesOf(path)
      .then((bytes) => put(path, bytes))
      .catch((e: unknown) => glue.og_web_fail_file(path, `cannot fetch ${path} (${e instanceof Error ? e.message : String(e)})`));
  });
}

/**
 * Gives the engine the browser's DecompressionStream for zlib bytes (terrain
 * tiles): it inflates natively off the engine's work and answers by key; an
 * engine without it inflates in wasm within a frame budget.
 */
export function installInflater(glue: Pick<WebExtras, "og_web_set_inflater" | "og_web_put_inflated" | "og_web_fail_inflated">): boolean {
  const Stream = (globalThis as { DecompressionStream?: new (format: string) => TransformStream<Uint8Array, Uint8Array> }).DecompressionStream;
  const { og_web_set_inflater: set, og_web_put_inflated: put, og_web_fail_inflated: fail } = glue;
  if (!Stream || !set || !put || !fail) return false;
  set((key: number, bytes: Uint8Array) => {
    new Response(new Blob([new Uint8Array(bytes)]).stream().pipeThrough(new Stream("deflate")))
      .arrayBuffer()
      .then(
        (b) => put(key, new Uint8Array(b)),
        (e: unknown) => fail(key, e instanceof Error ? e.message : String(e)),
      );
  });
  return true;
}

/**
 * A `capture` or `sweep` answered null: on the web the readback finishes on
 * a later call, and a sweep goes on from the frames it has each time it is
 * asked again. (A sweep refused outright, an unknown entity, answers null
 * too: it is asked again until its budget runs out.)
 */
function isPendingCapture(frame: string, answer: string): boolean {
  try {
    const request = JSON.parse(frame) as { method?: string };
    if (request.method !== "capture" && request.method !== "sweep") return false;
    const response = JSON.parse(answer) as { result?: unknown; error?: unknown };
    return response.error === undefined && response.result === null;
  } catch {
    return false;
  }
}

/** Display frames a pending `capture` is asked again on before it is given up on. */
const CAPTURE_RETRIES = 60;

/** Frames a `sweep` may wait: each value needs at least one call, at most 64 values. */
function retriesOf(frame: string): number {
  try {
    const request = JSON.parse(frame) as { method?: string; params?: { values?: unknown[] } };
    if (request.method === "sweep") return 2 * (request.params?.values?.length ?? 0) + CAPTURE_RETRIES;
  } catch {
    /* not a sweep */
  }
  return CAPTURE_RETRIES;
}

/** The most log lines the Worker keeps for a crash card. */
export const LOG_LINES = 200;

/** Played frames whose timings the Worker keeps (`timings`). */
const FRAME_RECORDS = 4096;

/** How often, at most, a playing Worker tells the shell it is alive (`beat`), ms. */
export const BEAT_MS = 1000;

/**
 * What a failure was: a Rust panic the wasm recorded before it trapped
 * (`og_web_panic`: its message, location, tick and lane), or else `message`
 * at the last tick a frame reached.
 */
export function hostErrorOf(panic: string | undefined, message: string, tick: number | null): HostError {
  try {
    const p = JSON.parse(panic ?? "null") as { kind?: string; message?: string; location?: string; tick?: number; lane?: string | null } | null;
    if (p && p.kind === "panic") {
      return { kind: "panic", message: p.message ?? message, tick: typeof p.tick === "number" ? p.tick : tick, lane: p.lane ?? null, location: p.location || null };
    }
  } catch {
    /* not JSON: no panic report */
  }
  return { kind: "error", message, tick, lane: null, location: null };
}

/** One line for the crash card and the log: `panic at tick 41 in a lane (cull): … (file:line:col)`. */
export function describeHostError(e: HostError): string {
  if (e.kind !== "panic") return e.message;
  return `panic at tick ${e.tick ?? "?"}${e.lane ? ` in a lane (${e.lane})` : ""}: ${e.message}${e.location ? ` (${e.location})` : ""}`;
}

/** Run `game` in this Worker, driven by the shell's messages. */
export function startWorker(game: GameToken): void {
  let engine: Engine | null = null;
  let ring: SharedArrayBuffer | undefined;
  let rpc: RpcRings | undefined;
  let canvas: OffscreenCanvas | undefined;
  let rendering = false;
  let playing = false;
  let scheduled = false;
  let ready = false;
  let crashed = false;
  let firstFrame = false;
  let last = 0;
  /** The tick the last frame reached (a crash's tick when no panic names one). */
  let lastTick: number | null = null;
  let lastBeat = 0;
  let glueRef: (WebGlue & WebExtras & WebAudioGlue) | null = null;
  let audio: WorkerAudio | null = null;
  /** Captures waiting for their readback (requested on tick N, answered on N + 1). */
  const retries: Array<{ frame: string; left: number; viaRing: boolean }> = [];

  // The console and the engine's log reach the shell (the crash card's last lines).
  const log = (text: string) => self.postMessage({ type: "log", lines: [text] });
  const consoleRef = (globalThis as unknown as { console?: Record<string, (...a: unknown[]) => void> }).console;
  if (consoleRef) {
    for (const level of ["error", "warn", "log", "info"] as const) {
      const original = consoleRef[level]?.bind(consoleRef);
      consoleRef[level] = (...args: unknown[]) => {
        original?.(...args);
        try {
          log(`${level === "log" || level === "info" ? "" : `${level}: `}${args.map((a) => (a instanceof Error ? (a.stack ?? a.message) : typeof a === "string" ? a : JSON.stringify(a))).join(" ")}`);
        } catch {
          /* an unserializable argument is not worth a crash */
        }
      };
    }
  }
  const flushLog = () => {
    if (!engine) return;
    const lines = engine.takeLog();
    if (lines.length > 0) self.postMessage({ type: "log", lines: lines.map((l) => `tick ${l.tick}: ${l.warning ? "warning: " : ""}${l.text}`) });
  };
  const crash = (message: string) => {
    if (crashed) return;
    crashed = true;
    playing = false;
    let panic: string | undefined;
    try {
      // A trap after a Rust panic: the panic hook recorded what it said, where and when.
      panic = glueRef?.og_web_panic?.();
    } catch {
      /* the module is past answering */
    }
    const error = hostErrorOf(panic, message, lastTick);
    try {
      flushLog();
    } catch {
      /* the engine may be the thing that broke */
    }
    const text = describeHostError(error);
    self.postMessage(ready ? { type: "crash", message: text, error } : { type: "error", message: text, error });
  };
  self.addEventListener?.("error", (e) => crash(e.message ?? String(e.error)));
  self.addEventListener?.("unhandledrejection", (e) => crash(e.reason instanceof Error ? e.reason.message : String(e.reason)));

  /** Real time not yet simulated after the last frame (its answer's `behind`), ms. */
  let behind = 0;
  /** The text target last sent to the shell, as JSON. */
  let textTarget = "null";
  /**
   * The played frames' timings, the last FRAME_RECORDS of them (the shell's
   * `timings` call, which `og run --device` reads): the display interval
   * (this frame's timestamp minus the last one's) and the engine's work
   * (`engine.frame`: input, ticks, render submission), ms.
   */
  const intervals = new Float32Array(FRAME_RECORDS);
  const work = new Float32Array(FRAME_RECORDS);
  let recorded = 0;
  /**
   * An event with the page's time becomes one with its `at` within the tick
   * that reads it (input.ts stampAt): `now` is this Worker's clock now (a
   * frame's timestamp, or a message's arrival), on the same timeline as `last`.
   */
  const stamped = (event: InjectedInput, now: number): InjectedInput => {
    if (!engine || !("time" in event) || event.time === undefined) return event;
    const { time, ...rest } = event;
    const ahead = behind + (last ? Math.max(0, now - last) : 0);
    return { ...rest, at: stampAt(time, performance.timeOrigin + now, ahead, 1000 / engine.game.tick) } as InjectedInput;
  };
  const pumpInput = (now: number) => {
    if (engine && ring) for (const event of drainRing(ring)) engine.probe.input(stamped(event, now));
  };

  /** What a frame's answer tells the shell: the UI's focused text control (text-input.ts), a copy, and the accessibility tree when it changed (a11y.ts). */
  const relay = (answered: ReturnType<Engine["frame"]>) => {
    if (!answered.ok) return;
    const target = JSON.stringify(answered.text ?? null);
    if (target !== textTarget) {
      textTarget = target;
      self.postMessage({ type: "text", target: answered.text ?? null });
    }
    if (answered.copy !== undefined) self.postMessage({ type: "copy", text: answered.copy });
    if (answered.a11y !== undefined) self.postMessage({ type: "a11y", tree: answered.a11y });
  };

  const answer = (frame: string, viaRing: boolean) => {
    if (!viaRing || !rpc || !writeFrame(rpc.answers, frame)) self.postMessage({ type: "rpc", frame });
  };

  /** A JSON-RPC error answer to a request frame (its id, or null when it has none). */
  const refusal = (frame: string, message: string) => {
    const id = (() => {
      try {
        return (JSON.parse(frame) as { id?: unknown }).id ?? null;
      } catch {
        return null;
      }
    })();
    return JSON.stringify({ jsonrpc: "2.0", id, error: { code: -32603, message } });
  };

  /** One request frame: answered now, or kept for a retry when it is a capture whose readback has not landed. */
  const serve = (frame: string, viaRing: boolean) => {
    if (!engine) return answer(refusal(frame, "the engine has not started"), viaRing);
    let out: string;
    try {
      out = engine.call(frame);
    } catch (e) {
      answer(refusal(frame, e instanceof Error ? e.message : String(e)), viaRing);
      crash(e instanceof Error ? e.message : String(e));
      return;
    }
    if (rendering && isPendingCapture(frame, out)) {
      retries.push({ frame, left: retriesOf(frame), viaRing });
      schedule();
    } else answer(out, viaRing);
  };

  const retry = () => {
    if (!engine) return;
    for (let i = retries.length - 1; i >= 0; i--) {
      const r = retries[i]!;
      const out = engine.call(r.frame);
      r.left--;
      if (!isPendingCapture(r.frame, out) || r.left <= 0) {
        retries.splice(i, 1);
        answer(out, r.viaRing);
      }
    }
  };

  const drainRpc = () => {
    if (engine && rpc) for (const frame of readFrames(rpc.requests)) serve(frame, true);
  };

  const frame = (now: number) => {
    scheduled = false;
    if (!engine || crashed) return;
    try {
      pumpInput(now);
      // Retries first: a capture asked this frame is requested now and answered on a later frame.
      retry();
      drainRpc();
      // The clock holds while a capture waits for its readback.
      if (playing && retries.length === 0) {
        const began = performance.now();
        const answered = engine.frame(last ? now - last : 0);
        if (!answered.ok) {
          crash(answered.reason ?? "the engine refused a frame");
          return;
        }
        if (last) {
          intervals[recorded % FRAME_RECORDS] = now - last;
          work[recorded % FRAME_RECORDS] = performance.now() - began;
          recorded++;
        }
        behind = answered.behind ?? 0;
        relay(answered);
        if (typeof answered.tick === "number") lastTick = answered.tick;
        if (!firstFrame && answered.rendered !== false) {
          firstFrame = true;
          self.postMessage({ type: "frame", first: true });
        }
        audio?.pump();
      }
      flushLog();
    } catch (e) {
      crash(e instanceof Error ? (e.stack ?? e.message) : String(e));
      return;
    }
    // Alive: the shell's frame deadline hears from a playing Worker about once a second.
    if (playing && now - lastBeat >= BEAT_MS) {
      lastBeat = now;
      self.postMessage({ type: "beat", tick: lastTick });
    }
    last = now;
    if (playing || retries.length > 0) schedule();
  };
  /**
   * The next frame on the display (requestAnimationFrame in the Worker); a
   * display that stops sending frames (a hidden page) gets a timer instead,
   * so a capture still lands.
   */
  const schedule = () => {
    if (scheduled) return;
    scheduled = true;
    let ran = false;
    const run = (t: number) => {
      if (ran) return;
      ran = true;
      frame(t);
    };
    if (self.requestAnimationFrame) self.requestAnimationFrame(run);
    if (!playing || !self.requestAnimationFrame) setTimeout(() => run(performance.now()), self.requestAnimationFrame ? 100 : 16);
  };

  /** Requests are answered between frames, whether the clock plays or not. */
  const requests = async () => {
    while (rpc && !crashed) {
      await waitForFrames(rpc.requests, 1000);
      if (!engine) continue;
      try {
        pumpInput(performance.now());
        drainRpc();
      } catch (e) {
        crash(e instanceof Error ? e.message : String(e));
      }
      // Yield to the event loop: frames, fetches and messages run between requests.
      await new Promise((r) => setTimeout(r, 0));
    }
  };

  /** One rendered frame without a tick: the first frame behind the play gate. */
  const present = () => {
    if (!engine || playing) return;
    try {
      const answered = engine.frame(0);
      if (!answered.ok) return crash(answered.reason ?? "the engine refused a frame");
      relay(answered);
      if (!firstFrame && answered.rendered !== false) {
        firstFrame = true;
        self.postMessage({ type: "frame", first: true });
      }
      flushLog();
    } catch (e) {
      crash(e instanceof Error ? e.message : String(e));
    }
  };

  const calls: Record<string, (e: Engine, args: unknown[]) => unknown> = {
    advance: (e, [n]) => e.advance(Number(n)),
    command: (e, [action, args]) => e.command(String(action), (args as string[]) ?? []),
    hash: (e) => e.hash(),
    /** What the lanes did so far (crates/og-runtime/src/lanes.rs `report`). */
    lanes: () => JSON.parse(glueRef?.og_web_lanes?.() ?? "null"),
    /** Test hook: the next item at lanes site `site` panics (og_web_lanes_fault); false when the module has no such hook or site. */
    fault: (_, [site]) => glueRef?.og_web_lanes_fault?.(String(site)) ?? false,
    frame: (e, [ms]) => {
      const answered = e.frame(Number(ms));
      relay(answered);
      return answered;
    },
    // The frames played since the last reset (at most FRAME_RECORDS, oldest first); `true` resets.
    timings: (_, [reset]) => {
      const n = Math.min(recorded, FRAME_RECORDS);
      const at = (a: Float32Array) => Array.from({ length: n }, (_, i) => Math.round(a[(recorded - n + i) % FRAME_RECORDS]! * 1000) / 1000);
      const answer = { frames: recorded, intervals: at(intervals), work: at(work) };
      if (reset) recorded = 0;
      return answer;
    },
    probe: (e, [member, ...rest]) => {
      const f = (e.probe as unknown as Record<string, unknown>)[String(member)];
      if (typeof f !== "function") throw new TypeError(`probe has no member '${String(member)}'`);
      return (f as (...a: unknown[]) => unknown)(...rest);
    },
  };

  self.onmessage = async ({ data }) => {
    if (data.type === "start") {
      try {
        const received = new Received((data.expect?.wasm ?? 0) + (data.expect?.cooked ?? 0));
        const progress = (value: number, stage: string) =>
          self.postMessage({ type: "progress", value: Math.min(1, value), stage, loaded: received.loaded, total: received.total });
        progress(0, "wasm");
        // The wasm streams through a counting body, so its bytes move the bar while it compiles.
        const counted = async (url: string, expected: number | undefined): Promise<unknown> => {
          if (!expected || typeof Response !== "function" || typeof ReadableStream !== "function") return url;
          const r = await fetch(url);
          if (!r.ok || !r.body) throw new Error(`cannot fetch ${url} (HTTP ${r.status})`);
          const reader = r.body.getReader();
          const body = new ReadableStream<Uint8Array>({
            async pull(controller) {
              const { done, value } = await reader.read();
              if (done) return controller.close();
              received.loaded += value.length;
              progress(received.value, "wasm");
              controller.enqueue(value);
            },
          });
          return new Response(body, { headers: { "content-type": "application/wasm" } });
        };
        // Lanes: the atomics build on a pool of Workers sharing its memory. A
        // variant that does not load or whose pool does not start falls back
        // to the single-threaded build, the same engine with the same API.
        let lanes: { on: boolean; threads: number; why: string } = { on: false, threads: 1, why: data.lanesWhy ?? "the page offered no lanes variant" };
        let glue: WebGlue & WebExtras & WebAudioGlue | null = null;
        if (data.lanes) {
          // The bar's total counts the lanes wasm in place of the
          // single-threaded one; after a failure it counts what the lanes
          // wasm downloaded plus the single-threaded wasm, once on every path.
          const total = received.total;
          const loaded = received.loaded;
          const lanesBytes = data.lanes.bytes ?? data.expect?.wasm;
          try {
            const g = (await import(data.lanes.glue)) as WebGlue & WebExtras & WebAudioGlue;
            if (data.expect) received.total = total + (lanesBytes ?? 0) - (data.expect.wasm ?? 0);
            await loadWebHost(g, await counted(data.lanes.wasm, lanesBytes), data.lanes.threads);
            glue = g;
            lanes = { on: true, threads: data.lanes.threads, why: "" };
          } catch (e) {
            // The lanes glue is discarded: loadWebHost bound nothing and told the engine nothing.
            if (data.expect) received.total = total + (received.loaded - loaded);
            lanes = { on: false, threads: 1, why: `the lanes variant did not start (${e instanceof Error ? e.message : String(e)}); single-threaded` };
          }
        }
        if (!glue) {
          glue = (await import(data.glue)) as WebGlue & WebExtras & WebAudioGlue;
          await loadWebHost(glue, await counted(data.wasm, data.expect?.wasm));
        }
        glueRef = glue;
        // Terrain tiles are inflated by the browser off the engine's thread where it can.
        installInflater(glue);
        progress(data.expect ? received.value : 0.1, "bundle");
        let fetched: Fetched | null = null;
        if (data.root !== undefined) {
          fetched = await preloadBundle(glue, data.root, progress, data.expect ? received : undefined);
          installFetcher(glue, fetched.manifest ?? "", data.basis, () => (rendering && engine ? engine.id : null));
        }
        const created = createEngine(game, {
          ...(data.seed === undefined ? {} : { seed: data.seed }),
          ...(data.root === undefined ? {} : { root: data.root }),
          ...(data.profile === undefined ? {} : { profile: data.profile }),
          ...(data.locale ? { locale: data.locale } : {}),
          deferStart: true,
          a11y: true,
        });
        if (!created.ok) throw new Error(created.reason);
        engine = created.value;
        // og.platform: calls go to the page, which answers them (the Player, or the browser's storage).
        // The time is the browser's until the Player's answers carry its own (docs/platform-asks.md).
        engine.setClock(systemClock("browser"));
        engine.attachPlatform(toPage("none", []));
        if (data.viewport) glue.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
        ring = data.ring;
        rpc = data.rpc;
        let tier = "none";
        if (data.canvas) {
          canvas = data.canvas;
          tier = await glue.og_web_attach_canvas(engine.id, data.canvas, !!data.webgl);
          rendering = true;
        }
        // The textures go to the renderer's encodings, so after it is attached (the game starts later).
        const loaded = fetched ? await transcodeBundle(glue, rendering ? engine.id : null, fetched, data.basis, progress, data.expect ? received : undefined) : null;
        progress(1, "ready");
        ready = true;
        self.postMessage({
          type: "ready",
          engine: engine.id,
          tier,
          transport: ring ? "ring" : "message",
          rpc: rpc ? "ring" : "message",
          textureFormat: loaded?.textureFormat ?? null,
          files: loaded?.files ?? 0,
          textures: loaded?.textures ?? 0,
          lanes,
        });
        void requests();
        if (data.autoplay) {
          playing = true;
          schedule();
        } else if (rendering) {
          if (data.prime && engine.probe.tick() === 0) {
            const primed = engine.advance(1);
            if (!primed.ok) throw new Error(primed.reason);
          }
          present();
        }
      } catch (e) {
        crash(e instanceof Error ? e.message : String(e));
      }
      return;
    }
    if (!engine || crashed) {
      // A request is answered even now, so that nothing on the page waits for an answer that never comes.
      const why = crashed ? "the engine stopped" : "the engine has not started";
      if (data.type === "call") self.postMessage({ type: "result", id: data.id, error: why });
      if (data.type === "rpc") answer(refusal(data.frame, why), false);
      return;
    }
    if (data.type === "input") engine.probe.input(stamped(data.event, performance.now()));
    if (data.type === "viewport" && glueRef) glueRef.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
    if (data.type === "resize" && glueRef) {
      if (canvas) {
        canvas.width = Math.max(1, data.width);
        canvas.height = Math.max(1, data.height);
      }
      glueRef.og_web_resize?.(engine.id, data.width, data.height);
      glueRef.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
      if (!playing) present();
    }
    if (data.type === "audio-latency") audio?.latency(data.latency);
    if (data.type === "audio" && glueRef && !audio) {
      const attached = WorkerAudio.attach(glueRef, engine.id, data);
      if (attached.ok) audio = attached.value;
      else log(`warn: audio: ${attached.reason}`);
    }
    if (data.type === "rpc") serve(data.frame, false);
    if (data.type === "platform-answer") engine.answerPlatform(data.request, data.answer);
    if (data.type === "platform-host") engine.attachPlatform(toPage(data.host, data.capabilities));
    if (data.type === "play") {
      playing = data.on;
      last = 0;
      if (playing) schedule();
    }
    if (data.type === "call") {
      try {
        pumpInput(performance.now());
        const call = calls[data.method];
        if (!call) throw new Error(`unknown call '${data.method}'`);
        self.postMessage({ type: "result", id: data.id, value: call(engine, data.args) });
      } catch (e) {
        self.postMessage({ type: "result", id: data.id, error: e instanceof Error ? e.message : String(e) });
      }
    }
  };
}
