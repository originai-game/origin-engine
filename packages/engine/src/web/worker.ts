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
// message } before ready; { type: "frame", first, boot } after the first
// rendered frame (`boot`: BootPhases, where the Worker's boot went); { type: "crash", message, error } when the engine fails after
// ready (a trap, a Rust panic on the engine's thread or in a lane, a failing
// system, an uncaught error; `error` is the structured HostError of shell.ts:
// kind, message, tick, lane, location); { type: "beat", tick } about once a
// second while frames play, which the shell's frame deadline watches;
// { type: "log", lines } with
// the engine's log and the Worker's console (the crash card shows the last
// 200); { type: "text", target } when the UI's focused text control changes
// and { type: "copy", text } when a copy took text (text-input.ts); { type: "saved", scenes } when the inspector saved scenes this page cannot write (page.ts routes them: wire.ts routeSavedScenes); { type: "haptics", patterns } when the UI's feedback played haptics (haptics.ts); { type: "rumble", rumble } when og.input.rumble asked for one (gamepad.ts playRumble); { type: "a11y", tree } when the UI's
// accessibility tree changed (a11y.ts); { type: "load-failure", file, reason } when a file
// of the bundle did not arrive (loadFailure: field telemetry's load failures); { type: "cursor", cursor } when the UI's pointer shape changed (og.ui.frameCursor); { type: "pointer-lock", wanted } when og.PointerLock.wanted
// changed (the shell requests the lock on the next press on the canvas, or exits it);
// { type: "result", id, value | error } for `call` (`advance`,
// `command`, `hash`, `frame`, `probe`, `timings`: the played frames'
// display intervals and engine work, and `telemetry`: their counts since the
// last take, frames.ts, with `start`'s `telemetry: true`); { type: "rpc",
// frame } for an answer that did not cross the ring.
//
// Requests on the ring are served between frames by the request loop
// whether or not the clock is playing (a page whose display frames stop, a
// hidden tab or frame, still answers), and the loop always waits for the
// next request instead of spinning: a waiting request used to starve the
// Worker's animation frames while the clock played, so isolated pages never
// answered.

import { boundHost, createEngine, systemClock, type Engine, type GameToken } from "../runtime.ts";
import type { PlatformAnswer, PlatformHost, PlatformNotice } from "../platform-local.ts";
import { loadWebHost, type WebGlue } from "./host.ts";
import { FrameCounter } from "./frames.ts";
import { FramePacer } from "./pacing.ts";
import { drainRing, stampAt, type InjectedInput } from "./input.ts";
import { WorkerAudio, type AudioLatencyMessage, type AudioMessage, type WebAudioGlue } from "./audio.ts";
import { readFrames, waitForFrames, writeFrame, type RpcRings } from "./rpc-ring.ts";
import { chooseDecoder, FALLBACK_URL, VideoStreams } from "./video.ts";
import { loadBasisTranscoder, transcodeKtx2, TEXTURE_FORMATS, type BasisTranscoder, type TextureFormat } from "./basis.ts";
import type { HostError } from "./shell.ts";
import { PlatformCounter, settle, SETTLE_FIRST_MS, SETTLE_MS } from "./settle.ts";
import { Trace, unixMs } from "../trace.ts";
import * as pipelines from "./pipelines.ts";
import type { PipelineChoice, PipelineStats } from "./pipelines.ts";
import * as gpuErrors from "./gpu-errors.ts";

// Fork s61: a lost WebGPU device and its first validation errors are said in the console (the
// renderer is made inside the wasm, so its device is met here, where the Worker asks for it).
// Fork s60: GPUDevice makes the renderer's frame pipelines asynchronously (pipelines.ts), installed
// on the prototype before the wasm's glue loads.
// A GPUInternalError popped from an error scope reaches wgpu as one it can read (gpu-errors.ts).
/** Called with the adapter as the renderer asks it for its device (before any pipeline is made): auto's pick. */
let onAdapter: ((adapter: { info?: { vendor?: string; architecture?: string } }) => void) | null = null;
{
  const g = globalThis as Partial<gpuErrors.ErrorClasses> & { GPUDevice?: { prototype: unknown } };
  const device = g.GPUDevice?.prototype;
  if (device) pipelines.install(device as Parameters<typeof pipelines.install>[0]);
  if (device && g.GPUInternalError && g.GPUValidationError)
    gpuErrors.install(device as Parameters<typeof gpuErrors.install>[0], { GPUInternalError: g.GPUInternalError, GPUValidationError: g.GPUValidationError });
  const proto = (globalThis as { GPUAdapter?: { prototype: { requestDevice(...a: unknown[]): Promise<GPUDeviceLike> } } }).GPUAdapter?.prototype;
  type GPUDeviceLike = { lost: Promise<{ reason: string; message: string }>; queue: GpuQueue; addEventListener(t: string, f: (e: { error?: { message?: string } }) => void): void };
  if (proto) {
    const requestDevice = proto.requestDevice;
    proto.requestDevice = async function (this: unknown, ...a: unknown[]) {
      onAdapter?.(this as { info?: { vendor?: string; architecture?: string } });
      const device = await requestDevice.apply(this, a);
      gpuQueue = device.queue;
      void device.lost.then((l) => console.error(`[og] WebGPU device lost: reason=${l.reason} message=${l.message}`));
      let errors = 0;
      device.addEventListener("uncapturederror", (e) => {
        if (errors++ < 20) console.error(`[og] WebGPU error: ${e.error?.message}`);
      });
      return device;
    };
  }
}

type GpuQueue = { submit(buffers: unknown[]): void; onSubmittedWorkDone(): Promise<unknown> };
/** The renderer's device queue (set when the wasm asks for its device). */
let gpuQueue: GpuQueue | null = null;
/**
 * Warmed pipelines the GPU process has not finished yet. A synchronous createRenderPipeline returns at once in
 * the Worker; Chrome sends it to the GPU process only when something flushes its wire (a submit), and the GPU
 * process compiles it there, in order, on the task that also runs every frame's and readback's commands. Behind
 * the gate nothing submitted, so the whole warm-up reached the GPU process with the tap's first frame and froze
 * it (11 s on a Pixel 10, warm cache; 15 s for a probe capture on a cold Metal cache). So each warm call is
 * flushed (an empty submit) and the next one waits until the queue's work so far, its compile included, is done:
 * the warm-up compiles behind the gate, and a tap or a capture waits for at most WARM_AHEAD compiles.
 */
let warmsInFlight = 0;
/**
 * The warm calls whose compile may be in flight at once. One was the bound until Firefox: Gecko sends a
 * pipeline to its GPU process at once (no wire to flush), but answers onSubmittedWorkDone about 100 ms
 * after an empty submit even on an idle device, with outliers of seconds, so a warm-up paced one a
 * time made 6 pipelines in 10 s behind the gate and the tap met the rest (its first frame 1.2 s,
 * studio-win, Firefox 159). Three keep Firefox's warm-up going while a tap or a capture in Chrome
 * still waits for at most three compiles.
 */
const WARM_AHEAD = 3;
/** Whether the warm-up waits for a compile in flight before its next call. */
const warmBusy = () => warmsInFlight >= WARM_AHEAD;
/** Runs one warm call; until its compile is done in the GPU process, it counts in `warmsInFlight`. */
function warmPaced(warm: () => boolean): boolean {
  const more = warm();
  if (more && gpuQueue) {
    warmsInFlight++;
    const done = () => {
      warmsInFlight--;
    };
    gpuQueue.submit([]);
    gpuQueue.onSubmittedWorkDone().then(done, done);
  }
  return more;
}

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
  seed?: number;
  ring?: SharedArrayBuffer;
  rpc?: RpcRings;
  autoplay?: boolean;
  /** Fork s61: the page draws its own loading screen and hears the game's stages (worker globals ogBootStage, ogBootIdle). */
  stages?: boolean;
  /**
   * Fork s60: project.ogt `publish { pipelines }`, absent for `auto` (pipelines.ts `choose`): `sync`; `async`, the frame pipelines with createRenderPipelineAsync;
   * `prewarm`, only the warm-up behind the gate's; absent: synchronously.
   */
  pipelines?: "sync" | "async" | "prewarm";
  /** The URL base whose `cooked/` bundle the engine opens (`.` next to the page). */
  root?: string;
  /** The Basis transcoder's script and module. */
  basis?: { js: string; wasm: string };
  /** The rav1d fallback decoder's module URL (dist/av1/og-av1.wasm by default). */
  av1?: string;
  /** The project's physics profile (project.ogt `profile`). */
  profile?: "2d" | "3d";
  /** project.ogt's text (CreateOptions.project). */
  project?: string;
  /** The locale og.i18n is asked for (the page's `navigator.language`). */
  locale?: string;
  /** The play surface in CSS pixels, its DPR and safe area: what the UI lays out in. */
  viewport?: Viewport;
  /** Bytes the boot will fetch (the wasm, the cooked bundle), for real progress; omitted: counted as they arrive. */
  expect?: { wasm?: number; cooked?: number };
  /**
   * Without autoplay: take the first tick before the first frame, so the frame behind a play gate shows the
   * game's first screen (its UI is built by ticks), then the ticks that apply the platform's answers to it (settle.ts).
   */
  prime?: boolean;
  /** Record the run's trace from the boot on (src/trace.ts; the shell's `trace()` reads it). */
  trace?: boolean;
  /** Count the played frames for field telemetry (frames.ts; the `telemetry` call takes them). */
  telemetry?: boolean;
  /** Ask for the low-power GPU adapter: the player's power preference from the last load (og.render.setDisplay). */
  lowPower?: boolean;
}

type Viewport = { width: number; height: number; dpr: number; safeArea?: Record<string, number>; covered?: Record<string, number>; coarsePointer?: boolean; reducedMotion?: boolean; highContrast?: boolean; reducedTransparency?: boolean; macKeys?: boolean };

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
  | { type: "platform-notice"; notice: PlatformNotice }
  | { type: "platform-host"; host: string; capabilities: string[]; time?: number }
  | { type: "room-events"; events: unknown[] }
  | { type: "room-host"; host: string; join?: string }
  | { type: "fullscreen"; on: boolean }
  | { type: "mouse-report"; active: boolean; unsupported: boolean }
  | { type: "keyboard-layout"; labels: { code: string; label: string }[] }
  | AudioMessage
  | AudioLatencyMessage;

/**
 * Rooms (docs/design.md §2.8): the engine's room ops (og_room_take) go to the
 * page as `room` messages, which the shell's room host carries (the Player's
 * room.*, docs/platform-asks.md #16); the page's `room-events` come back
 * through og_room_deliver, in order, and `room-host` names what carries them.
 * Ops are taken after every frame and after events are delivered, so
 * lockstep's inputs leave on the frame that made them.
 */
function pumpRoom(engine: Engine): void {
  const host = boundHost();
  if (!host) return;
  const ops = JSON.parse(host.abi.og_room_take(engine.id)) as unknown[];
  if (ops.length > 0) self.postMessage({ type: "room", ops });
}

/** og.platform's host in the Worker: the calls go to the page (counted), whose answers come back as `platform-answer`. */
function toPage(host: string, capabilities: readonly string[], counter: PlatformCounter): PlatformHost {
  return {
    host,
    capabilities,
    handle: (calls) => {
      counter.sent(calls);
      self.postMessage({ type: "platform", calls });
    },
  };
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
  /** Streamed video's decoders (video.ts); absent in an older engine module. */
  og_web_install_video?(
    stream: (key: number, width: number, height: number, table: Uint32Array, data: Uint8Array) => boolean,
    frame: (stream: number, player: number, frame: number, looped: boolean) => VideoFrame | null,
    decoder: string,
    hardware: boolean,
    maxWidth: number,
    maxHeight: number,
  ): void;
  og_web_set_transcoder?(transcode: (id: string, ktx2: Uint8Array, target: string) => void): void;
  og_web_fail_texture?(id: string, reason: string): void;
  og_web_set_inflater?(inflate: (key: number, bytes: Uint8Array) => void): void;
  og_web_put_inflated?(key: number, bytes: Uint8Array): void;
  og_web_fail_inflated?(key: number, reason: string): void;
  og_web_set_image_decoder?(decode: (key: number, bytes: Uint8Array) => void): void;
  og_web_put_image?(key: number, width: number, height: number, rgba: Uint8Array): void;
  og_web_fail_image?(key: number, reason: string): void;
  og_web_bundle_files(manifest: string): string;
  og_web_texture_ktx2(manifest: string, id: string, bytes: Uint8Array): Uint8Array;
  og_web_pack_ktx2?(engine: number | undefined, manifest: string, pack: Uint8Array): Array<string | Uint8Array>;
  og_web_attach_canvas(engine: number, canvas: OffscreenCanvas, lowPower: boolean): Promise<string>;
  /** The page went fullscreen or left it (og.render.display's `fullscreen`; the setting follows). */
  og_web_fullscreen?(engine: number, on: boolean): void;
  /** What the browser granted the pointer lock: raw movement now, and whether it refused unadjustedMovement (og.input.mouse). */
  og_web_mouse_report?(engine: number, active: boolean, unsupported: boolean): void;
  og_web_set_viewport(engine: number, viewport: string): string;
  og_web_resize?(engine: number, width: number, height: number): void;
  og_web_lanes?(): string;
  /** What the engine holds (the probe's `memory`, ProbeMemory), as JSON. */
  og_web_memory?(engine: number): string;
  /** The device the attached renderer runs on: its WebGPU limits and features, whether it is a fallback adapter (JSON, null before attaching). */
  og_web_gpu?(engine: number): string;
  /** Make one pipeline no frame used yet (og-render's Renderer::warm); false when none is left. */
  og_web_warm?(engine: number): boolean;
  /** The display's refresh interval the Worker measured, ms (automatic quality's budget at no frame cap). */
  og_web_display_interval?(engine: number, ms: number): void;
  /** Brackets a probe-driven frame, excluding it from automatic display quality. */
  og_web_probe_frame(engine: number, on: boolean): void;
  /** Test hook: destroy the GPU device; the renderer is made again on a new one and the game goes on. */
  og_web_lose_device?(engine: number): boolean;
  /** Fork s60: `span(on)` around the renderer's frame pipelines (og-render set_frame_pipelines_hook), which pipelines.ts makes asynchronously. */
  og_web_frame_pipelines?(span: ((on: boolean) => void) | null): void;
  /** Fork s63: the texture bytes a frame uploads at most (0: no limit). */
  og_web_upload_budget?(bytes: number): void;
  /** Fork s63: the textures still drawn from their stand-ins after the last frame. */
  og_web_textures_deferred?(): number;
  /** Fork s70: the page's loading screen covers the canvas: frames prepare without drawing (og-render set_covered). */
  og_web_covered?(on: boolean): void;
  /** `og render`'s every-frame capture (development builds): the display frame drawn last as RGBA8 rows at the canvas's size; empty until its readback lands. */
  og_web_render_frame?(engine: number): Uint8Array;
  /** The first Rust panic recorded (JSON: kind, message, location, tick, lane), or `null`. */
  og_web_panic?(): string;
  /** Test hook: the next item at a lanes site panics (in a lane when the call splits). */
  og_web_lanes_fault?(site: string): boolean;
  /** The trace (crates/og-runtime/src/trace.rs): on or off for an engine (0xffffffff: the loads before one exists). */
  og_web_trace?(engine: number, on: boolean): void;
  /** The trace's spans since the last take: `{ now, events }`, Chrome trace events on the engine's clock. */
  og_web_trace_take?(engine: number, last: boolean): string;
}

/** The trace's key for loads made before the engine exists (og-runtime's `BEFORE_ENGINE`). */
const TRACE_BEFORE_ENGINE = 0xffffffff;

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

/** How many of the first group's files the boot fetches at once. */
export const FETCH_CONCURRENCY = 24;

/**
 * The device the engine renders with, for field telemetry: its limits,
 * features and fallback flag from the wasm (`og_web_gpu`), and its adapter's
 * vendor and architecture from the device itself (the canvas's WebGPU
 * context's configured device, `GPUDevice.adapterInfo`), which wgpu does not
 * carry. Null without a renderer; vendor and architecture empty where the
 * browser has no `getConfiguration` or `adapterInfo`.
 */
export function deviceReport(glue: { og_web_gpu?(engine: number): string }, engine: number, canvas: OffscreenCanvas | undefined): Record<string, unknown> | null {
  let report: Record<string, unknown> | null = null;
  try {
    report = JSON.parse(glue.og_web_gpu?.(engine) ?? "null") as Record<string, unknown> | null;
  } catch {
    return null;
  }
  if (!report) return null;
  let info: { vendor?: unknown; architecture?: unknown; isFallbackAdapter?: unknown } = {};
  try {
    const context = (canvas as unknown as { getContext(kind: string): { getConfiguration?(): { device?: { adapterInfo?: typeof info } } | null } | null } | undefined)?.getContext("webgpu");
    info = context?.getConfiguration?.()?.device?.adapterInfo ?? {};
  } catch {
    /* an older browser: limits and features only */
  }
  return { info: { vendor: info.vendor, architecture: info.architecture, isFallbackAdapter: report["fallback"] === true || info.isFallbackAdapter === true }, limits: report["limits"], features: report["features"] };
}

/**
 * A file of the bundle that did not arrive, told to the page (`{ type:
 * "load-failure", file, reason }`, field telemetry's load failures): `file`
 * is its path under `cooked/` (no origin), `reason` one of `http_<status>`,
 * `timeout` or `network` (no message text).
 */
export function loadFailure(url: string, e: unknown): { type: "load-failure"; file: string; reason: string } {
  const text = e instanceof Error ? `${e.name} ${e.message}` : String(e);
  const status = /HTTP (\d{3})/.exec(text)?.[1];
  const cooked = url.replace(/[?#].*$/, "").split("/cooked/");
  return {
    type: "load-failure",
    file: (cooked.length > 1 ? `cooked/${cooked[cooked.length - 1]}` : url.slice(url.lastIndexOf("/") + 1)).slice(0, 160),
    reason: status ? `http_${status}` : /timeout|abort/i.test(text) ? "timeout" : "network",
  };
}
const tellLoadFailure = (url: string, e: unknown) => {
  try {
    (globalThis as { postMessage?: (m: unknown) => void }).postMessage?.(loadFailure(url, e));
  } catch {
    /* no page to tell */
  }
};

/** Fork s63: the texture bytes one frame uploads at most (4 MB: a 32 MB frame was one 1.3-2.5 s GPU-process task on an RTX 3090). */
export const UPLOAD_BUDGET = 4 * 1024 * 1024;

/**
 * Fork s61: each boot stage's share of the progress bar, so it does not sit at 100 % once the
 * bytes are in: the download (wasm, bundle, fetch) 0-70 %, the Basis transcode 70-85 %, the first
 * tick 85-92 %, the first frame (its pipelines) 92-100 %.
 */
export const BOOT_SPANS: Readonly<Record<string, readonly [number, number]>> = {
  wasm: [0, 0.7],
  bundle: [0, 0.7],
  fetch: [0, 0.7],
  transcode: [0.7, 0.85],
  ready: [0.85, 0.85],
  prime: [0.85, 0.92],
  present: [0.92, 1],
};

/** A stage's fraction (0 to 1) as a place on the whole bar. */
export function onBar(stage: string, value: number): number {
  const [a, b] = BOOT_SPANS[stage] ?? [0, 1];
  return a + (b - a) * Math.max(0, Math.min(1, value));
}

async function bytesOf(url: string, received?: Received, onChunk?: () => void): Promise<Uint8Array> {
  // Fork s61: a fetch that failed (not refused: HTTP 4xx) is tried again, up to 5 times, backing off;
  // its bytes already counted come off the progress first.
  for (let attempt = 0; ; attempt++) {
    const before = received ? received.loaded : 0;
    try {
      return await bytesOnce(url, received, onChunk);
    } catch (e) {
      if (received) received.loaded = before;
      if (attempt >= 5 || /HTTP 4\d\d/.test(String(e))) throw e;
      console.warn(`fetch ${url} failed (${e instanceof Error ? e.message : String(e)}), retry ${attempt + 1}`);
      await new Promise((ok) => setTimeout(ok, 500 * 2 ** attempt));
    }
  }
}

async function bytesOnce(url: string, received?: Received, onChunk?: () => void): Promise<Uint8Array> {
  // A cooked file's name may hold a `#` (`m.gltf.tex0#mips.ogb`, a flipbook's
  // `#array#layer<n>`): sent as is, the browser takes the rest as a fragment.
  const r = await fetch(url.replaceAll("#", "%23"), typeof AbortSignal?.timeout === "function" ? { signal: AbortSignal.timeout(FETCH_DEADLINE_MS) } : {});
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

/**
 * Streamed video's decoders for this Worker (video.ts): the decoder this
 * browser plays AV1 with, reported by og.video.capabilities; the rav1d
 * fallback module (`av1`, dist/av1/og-av1.wasm by default) is fetched only
 * when WebCodecs cannot decode AV1, by its own Worker, when a stream first
 * plays.
 */
export async function installVideo(glue: WebExtras, av1: string | undefined): Promise<VideoStreams | null> {
  if (!glue.og_web_install_video) return null;
  const choice = await chooseDecoder();
  const streams = new VideoStreams(choice, av1 ?? FALLBACK_URL);
  glue.og_web_install_video(
    (key, width, height, table, data) => streams.stream(key, width, height, 0, table, data),
    (stream, player, frame, looped) => streams.want(player, stream, frame, looped),
    choice.name,
    choice.hardware,
    choice.maxWidth,
    choice.maxHeight,
  );
  return streams;
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
 * through the fetcher ahead of the playheads, nor a streamed video's units
 * (og.VideoStream), which og.load.video asks the fetcher for, nor a
 * streamed texture's whole chain (`<id>#mips`, `texture { stream true }`):
 * the bundle carries its mip tail as `<id>`, and the engine reads `#mips`
 * through the fetcher when the texture is big on screen. The engine asks for at most
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
    tellLoadFailure(manifestPath, e);
    glue.og_web_fail_file(manifestPath, `cannot fetch ${manifestPath} (${e instanceof Error ? e.message : String(e)}); run og cook and serve cooked/ next to the page`);
    return { files: 0, failed: 1, manifest: null, textures: [], streamed: 0 };
  }
  glue.og_web_put_file(manifestPath, new TextEncoder().encode(manifest));
  const all = JSON.parse(glue.og_web_bundle_files(manifest)) as Array<{ id: string; file: string; type: string; group?: string }> | { error: string };
  if (!Array.isArray(all)) throw new Error(`${manifestPath}: ${all.error}`);
  // A long sound's chunks (og.SoundChunk) are fetched as its playheads near
  // them, a streamed texture's whole chain (`#mips`) when it is big on screen.
  const listed = all.filter((o) => o.group === undefined && o.type !== "og.SoundChunk" && o.type !== "og.VideoStream" && !o.id.endsWith("#mips"));
  let fetched = 0;
  let failed = 0;
  const blobs = new Map<string, Uint8Array>();
  // At most FETCH_CONCURRENCY in flight: a bundle of thousands of files
  // fetched all at once fails some with "Failed to fetch" (Chrome runs out
  // of sockets and buffers).
  let next = 0;
  const one = async (o: (typeof listed)[number]) => {
    const path = `${dir}/${o.file}`;
    try {
      const bytes = await bytesOf(path, received, received ? () => progress(received.value, "fetch") : undefined);
      glue.og_web_put_file(path, bytes);
      blobs.set(o.id, bytes);
    } catch (e) {
      failed++;
      tellLoadFailure(path, e);
      glue.og_web_fail_file(path, `cannot fetch ${path} (${e instanceof Error ? e.message : String(e)})`);
    }
    ++fetched;
    if (!received) progress(0.1 + 0.7 * (fetched / Math.max(1, listed.length)), "fetch");
  };
  await Promise.all(
    Array.from({ length: Math.min(FETCH_CONCURRENCY, listed.length) }, async () => {
      while (next < listed.length) await one(listed[next++]!);
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
  void received;
  const used: string[] = [];
  let transcoder: BasisTranscoder | null = null;
  let transcoded = 0;
  const total = fetched.textures.length;
  const put = (id: string, out: { format: string; width: number; height: number; srgb: boolean; levels: Uint8Array }) => {
    if (out.format === "rgba8") {
      glue.og_web_put_texture(id, out.width, out.height, out.levels, out.srgb);
    } else {
      const refused = glue.og_web_put_texture_encoded(id, out.format, out.width, out.height, out.srgb, out.levels);
      if (refused) throw new Error(refused);
    }
    if (!used.includes(out.format)) used.push(out.format);
    transcoded++;
    // Fork s61: by texture, as a fraction of the stage (the Worker places it on the bar).
    if (transcoded % 8 === 0 || transcoded === total) progress(transcoded / Math.max(1, total), "transcode");
  };
  // The wasm keeps the bundle it parsed last (og-host-web `bundle_of`, s58): the manifest's text
  // crosses once, and "" names it in every later call.
  let manifest = fetched.manifest;
  const jobs: BasisJob[] = [];
  for (const t of fetched.textures) {
    const ktx2 = manifest === null ? new Uint8Array() : glue.og_web_texture_ktx2(manifest, t.id, t.bytes);
    if (manifest !== null) manifest = "";
    if (ktx2.length > 0) jobs.push({ id: t.id, ktx2, target: engine === null ? "rgba8" : glue.og_web_texture_target(engine, "", t.id, t.bytes) });
  }
  // Fork s62: a bundle of many textures is transcoded by a pool of Workers (copies of this one)
  // in parallel; a pool that does not start leaves the rest to this Worker.
  const left = jobs.length >= POOL_MIN_JOBS ? await transcodeInPool(jobs, basis, put) : jobs;
  for (const job of left) {
    transcoder ??= basis ? await loadBasisTranscoder(basis.js, basis.wasm) : await loadBasisTranscoder();
    const out = transcodeKtx2(transcoder, job.ktx2, isTextureFormat(job.target) ? job.target : "rgba8");
    put(job.id, { ...out, levels: out.format === "rgba8" ? out.levels[0]! : concat(out.levels) });
  }
  return { files: fetched.files, failed: fetched.failed, textures: transcoded, textureFormat: used.length ? used.join("+") : "rgba8" };
}

interface BasisJob {
  readonly id: string;
  readonly ktx2: Uint8Array;
  readonly target: string;
}
type BasisAnswer = { type: "og-basis"; id: number; error?: string; format: string; width: number; height: number; srgb: boolean; levels: Uint8Array };

/** Fork s62: below this many Basis textures the boot transcodes them itself. */
const POOL_MIN_JOBS = 24;

/** Fork s62: the transcoding Workers: up to 4, leaving two cores to the page and this Worker. */
function poolSize(): number {
  const cores = (globalThis as { navigator?: { hardwareConcurrency?: number } }).navigator?.hardwareConcurrency ?? 1;
  return Math.max(0, Math.min(4, cores - 2));
}

/**
 * Fork s62: transcode `jobs` on a pool of Workers running this module (its `og-basis` message,
 * `basisJob`), each answer handed to `put` as it lands. Answers the jobs no Worker did (all of
 * them when the pool cannot start), for this Worker to do.
 */
async function transcodeInPool(
  jobs: readonly BasisJob[],
  basis: { js: string; wasm: string } | undefined,
  put: (id: string, out: { format: string; width: number; height: number; srgb: boolean; levels: Uint8Array }) => void,
): Promise<BasisJob[]> {
  const n = poolSize();
  const url = (globalThis as { location?: { href: string } }).location?.href;
  if (n < 2 || !url || typeof Worker !== "function") return [...jobs];
  const pool: Worker[] = [];
  const left: BasisJob[] = [];
  let next = 0;
  let failure: unknown = null;
  try {
    await Promise.all(
      Array.from({ length: n }, async () => {
        const w = new Worker(url, { type: "module" });
        pool.push(w);
        // Each Worker takes the next job when it answered the last; one that fails gives its job back.
        for (;;) {
          if (failure !== null || next >= jobs.length) return;
          const k = next++;
          const job = jobs[k]!;
          const answer = await new Promise<BasisAnswer | null>((resolve) => {
            w.onmessage = (e: MessageEvent) => resolve(e.data as BasisAnswer);
            w.onerror = () => resolve(null);
            const ktx2 = job.ktx2.slice();
            w.postMessage({ type: "og-basis", id: k, ktx2, target: job.target, basis }, [ktx2.buffer]);
          });
          if (!answer || answer.error) {
            left.push(job);
            if (!answer) return; // this Worker did not load: the others go on
            continue;
          }
          try {
            put(job.id, answer);
          } catch (e) {
            failure = e;
          }
        }
      }),
    );
  } finally {
    for (const w of pool) w.terminate();
  }
  if (failure !== null) throw failure;
  // The jobs no Worker reached (every one failed to load) are this Worker's.
  return [...left, ...jobs.slice(next)];
}

/** Fork s62: this Worker as one of the pool: a Basis transcode, answered with its levels (transferred). */
let poolTranscoder: Promise<BasisTranscoder> | null = null;
async function basisJob(data: { id: number; ktx2: Uint8Array; target: string; basis?: { js: string; wasm: string } }): Promise<void> {
  try {
    poolTranscoder ??= data.basis ? loadBasisTranscoder(data.basis.js, data.basis.wasm) : loadBasisTranscoder();
    const out = transcodeKtx2(await poolTranscoder, data.ktx2, isTextureFormat(data.target) ? data.target : "rgba8");
    const levels = out.format === "rgba8" ? out.levels[0]! : concat(out.levels);
    (self as unknown as { postMessage(m: unknown, transfer: Transferable[]): void }).postMessage({ type: "og-basis", id: data.id, format: out.format, width: out.width, height: out.height, srgb: out.srgb, levels }, [levels.buffer as ArrayBuffer]);
  } catch (e) {
    self.postMessage({ type: "og-basis", id: data.id, error: e instanceof Error ? e.message : String(e) });
  }
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
  const loaded = (): Promise<BasisTranscoder> => {
    let loading = transcoder;
    if (!loading) {
      const started = basis ? loadBasisTranscoder(basis.js, basis.wasm) : loadBasisTranscoder();
      transcoder = loading = started;
      // A load that failed is not kept: the next pack tries again.
      started.catch(() => {
        if (transcoder === started) transcoder = null;
      });
    }
    return loading;
  };
  const transcode = async (id: string, ktx2: Uint8Array, target: string) => {
    const out = transcodeKtx2(await loaded(), ktx2, isTextureFormat(target) ? target : "rgba8");
    if (out.format === "rgba8") {
      glue.og_web_put_texture(id, out.width, out.height, out.levels[0]!, out.srgb);
    } else {
      const refused = glue.og_web_put_texture_encoded(id, out.format, out.width, out.height, out.srgb, concat(out.levels));
      if (refused) throw new Error(refused);
    }
  };
  const put = async (path: string, bytes: Uint8Array) => {
    const triples = /\/stream\/[^/]+\.ogb$/.test(path) && glue.og_web_pack_ktx2 ? glue.og_web_pack_ktx2(engine() ?? undefined, manifest, bytes) : [];
    for (let i = 0; i + 2 < triples.length; i += 3) {
      await transcode(triples[i] as string, triples[i + 1] as Uint8Array, triples[i + 2] as string);
    }
    glue.og_web_put_file(path, bytes);
  };
  // A device made again after a loss may take other encodings (og-host-web's
  // texture hook): the engine hands back each texture transcoded for the old one.
  glue.og_web_set_transcoder?.((id, ktx2, target) => {
    transcode(id, ktx2, target).catch((e: unknown) => glue.og_web_fail_texture?.(id, e instanceof Error ? e.message : String(e)));
  });
  glue.og_web_set_fetcher((path: string) => {
    bytesOf(path)
      .then((bytes) => put(path, bytes))
      .catch((e: unknown) => {
        tellLoadFailure(path, e);
        glue.og_web_fail_file(path, `cannot fetch ${path} (${e instanceof Error ? e.message : String(e)})`);
      });
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
 * Gives the engine the browser's image decoders for og.load.imageData: a
 * PNG or JPEG becomes an ImageBitmap off the engine's work
 * (`createImageBitmap`, no colour conversion, alpha not premultiplied) and
 * its RGBA8 texels are read back through an OffscreenCanvas, answered by
 * key. Without it the engine has no decoder and such a load fails.
 */
export function installImageDecoder(glue: Pick<WebExtras, "og_web_set_image_decoder" | "og_web_put_image" | "og_web_fail_image">): boolean {
  const g = globalThis as { createImageBitmap?: (b: Blob, o?: Record<string, string>) => Promise<ImageBitmap>; OffscreenCanvas?: new (w: number, h: number) => OffscreenCanvas };
  const { og_web_set_image_decoder: set, og_web_put_image: put, og_web_fail_image: fail } = glue;
  if (!g.createImageBitmap || !g.OffscreenCanvas || !set || !put || !fail) return false;
  const decode = g.createImageBitmap;
  const Canvas = g.OffscreenCanvas;
  set((key: number, bytes: Uint8Array) => {
    decode(new Blob([new Uint8Array(bytes)]), { premultiplyAlpha: "none", colorSpaceConversion: "none" })
      .then((bitmap) => {
        const { width, height } = bitmap;
        const ctx = new Canvas(width, height).getContext("2d", { willReadFrequently: true }) as OffscreenCanvasRenderingContext2D | null;
        if (!ctx) throw new Error("no 2D context to read the image's texels");
        ctx.drawImage(bitmap, 0, 0);
        bitmap.close();
        put(key, width, height, new Uint8Array(ctx.getImageData(0, 0, width, height).data.buffer));
      })
      .catch((e: unknown) => fail(key, `the browser could not decode it (${e instanceof Error ? e.message : String(e)})`));
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
  // Every null result is written `"result":null` (the probe's compact JSON);
  // a frame's answer is megabytes of base64 not worth parsing to learn it is not null.
  if (!answer.includes('"result":null')) return false;
  try {
    const request = JSON.parse(frame) as { method?: string };
    if (request.method !== "capture" && request.method !== "sweep") return false;
    const response = JSON.parse(answer) as { result?: unknown; error?: unknown };
    return response.error === undefined && response.result === null;
  } catch {
    return false;
  }
}

/**
 * Fork s75: the warmed pipelines compiling at once (s60/s70 warm). Chrome compiles
 * `createRenderPipelineAsync` off its GPU-process main thread, so two at a time left
 * it idle: the warm tail after the 1072-cell boot was 4-8 s at two.
 */
const WARM_COMPILING = 6;
/**
 * How long a pending `capture` is asked again, once a display frame, before it is given up on (ms).
 * Wall-clock, not frames: at 60 frames a playing page gave up after 1 s, while a GPU process with
 * a cold shader cache (Edge on D3D12) was still compiling the pipelines queued before the readback.
 */
const CAPTURE_MS = 20_000;

/** How long a request may wait for its readbacks: a `sweep` 250 ms more for each value (at most 64). */
function waitOf(frame: string): number {
  try {
    const request = JSON.parse(frame) as { method?: string; params?: { values?: unknown[] } };
    if (request.method === "sweep") return 250 * (request.params?.values?.length ?? 0) + CAPTURE_MS;
  } catch {
    /* not a sweep */
  }
  return CAPTURE_MS;
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

/**
 * Where the boot went, ms on the Worker's clock: each phase's duration, in
 * order (`wasm`: the engine module fetched and instantiated, the lanes
 * pool started; `bundle`: the first group fetched; `create`: the engine and
 * the game's declarations; `attach`: the renderer created on the canvas;
 * `transcode`: the Basis textures; `prime`: the tick behind the gate;
 * `settle`: the ticks that applied the platform's answers behind the gate;
 * `present`: the first frame rendered, its pipelines compiled), and `at`,
 * the Worker's time at the first frame from its own start.
 */
export type BootPhases = Partial<Record<"wasm" | "bundle" | "create" | "attach" | "transcode" | "prime" | "settle" | "present", number>> & {
  at?: number;
  /** Fork s60: the render pipelines made each way (pipelines.ts), at the first frame and again when the warm-up ends or play stops it, with the mode chosen and why (`auto`'s pick) and `bypassed` when the frame pipelines lost createRenderPipelineAsync. */
  pipelines?: PipelineStats & Partial<PipelineChoice> & { bypassed?: string };
};

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
  /** What the page was asked and answered: the ticks behind the gate apply its answers (settle.ts). */
  const platformCounter = new PlatformCounter();
  /** Where the boot went (BootPhases), sent with the first frame. */
  const boot: BootPhases = {};
  let phaseAt = performance.now();
  const phase = (name: Exclude<keyof BootPhases, "at" | "pipelines">) => {
    const now = performance.now();
    boot[name] = Math.round((boot[name] ?? 0) + now - phaseAt);
    phaseAt = now;
    console.info(`[og] boot stage "${name}" done in ${boot[name]} ms`);
  };
  const firstFrameMessage = () => (console.info(`[og] first frame at ${Math.round(performance.now())} ms`), { type: "frame", first: true, boot: { ...boot, at: Math.round(performance.now()), pipelines: pipelines.stats() } });
  /** Fork s60: the pipelines' counts once `warmed` pipelines are made, said loudly when the frame pipelines bypassed createRenderPipelineAsync. */
  const reportPipelines = (warmed: number) => {
    const stats = pipelines.stats();
    const bypassed = pipelines.bypassed(stats, warmed);
    if (bypassed) console.warn(`[og] renderer: ${bypassed}`);
    self.postMessage({ type: "pipelines", stats: bypassed ? { ...stats, bypassed } : stats });
  };
  /** What the trace's document records beside its events: the renderer's tier and the lanes. */
  let lastTier = "none";
  let lanesState: unknown = null;
  let glueRef: (WebGlue & WebExtras & WebAudioGlue) | null = null;
  let audio: WorkerAudio | null = null;
  /** Captures waiting for their readback (requested on tick N, answered on N + 1). */
  const retries: Array<{ frame: string; until: number; viaRing: boolean }> = [];

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
  /** og.PointerLock.wanted as last sent to the shell. */
  let lockWanted = false;
  let lockRaw = true;
  /** The canvas's CSS cursor as the last frame answer set it. */
  let cursor = "default";
  /** The page asks the last frame relayed ([fullscreen, lowPower]), so only a change is posted. */
  let pageAsks = "";
  /**
   * The played frames' timings, the last FRAME_RECORDS of them (the shell's
   * `timings` call, which `og run --device` reads): the display interval
   * (this frame's timestamp minus the last one's) and the engine's work
   * (`engine.frame`: input, ticks, render submission), ms.
   */
  const intervals = new Float32Array(FRAME_RECORDS);
  const work = new Float32Array(FRAME_RECORDS);
  let recorded = 0;
  /** Field telemetry's frame counts (frames.ts), when the page's build has telemetry on; null otherwise. */
  let frameCounter: FrameCounter | null = null;
  /**
   * An event with the page's time becomes one with its `at` within the tick
   * that reads it (input.ts stampAt): `now` is this Worker's clock now (a
   * frame's timestamp, or a message's arrival), on the same timeline as `last`.
   */
  const stamped = (event: InjectedInput, now: number): InjectedInput => {
    if (!engine || !("time" in event) || event.time === undefined) return event;
    const { time, ...rest } = event;
    const ahead = behind + (last ? Math.max(0, now - last) : 0);
    return { ...rest, at: stampAt(time, performance.timeOrigin + now, ahead, 1000 / engine.rate) } as InjectedInput;
  };
  const pumpInput = (now: number) => {
    if (engine && ring) for (const event of drainRing(ring)) engine.probe.input(stamped(event, now));
  };

  /** What a frame's answer tells the shell: the UI's focused text control (text-input.ts), a copy, the accessibility tree when it changed (a11y.ts), and og.PointerLock.wanted when it changed. */
  /** The run's trace while one is recorded (src/trace.ts): taken after every frame, so the engine holds a frame's spans at most. */
  let trace: Trace | null = null;
  const takeTrace = (last = false) => {
    if (!trace || !engine || !glueRef?.og_web_trace_take) return;
    trace.append(glueRef.og_web_trace_take(engine.id, last), unixMs());
  };
  const traceOn = (on: boolean) => {
    if (!engine || !glueRef?.og_web_trace) return false;
    if (on && !trace) trace = new Trace({ producer: "og web trace", process: "og web" });
    if (!on) trace = null;
    glueRef.og_web_trace(engine.id, on);
    return true;
  };
  const relay = (answered: ReturnType<Engine["frame"]>) => {
    takeTrace();
    if (!answered.ok) return;
    const target = JSON.stringify(answered.text ?? null);
    if (target !== textTarget) {
      textTarget = target;
      self.postMessage({ type: "text", target: answered.text ?? null });
    }
    if (answered.copy !== undefined) self.postMessage({ type: "copy", text: answered.copy });
    if (answered.saved?.scenes.length) self.postMessage({ type: "saved", scenes: answered.saved.scenes });
    if (answered.a11y !== undefined) self.postMessage({ type: "a11y", tree: answered.a11y });
    if (answered.haptics?.length) self.postMessage({ type: "haptics", patterns: answered.haptics });
    if (answered.rumble?.length) self.postMessage({ type: "rumble", rumble: answered.rumble });
    const shape = typeof answered.cursor === "string" ? answered.cursor : "default";
    if (shape !== cursor) {
      cursor = shape;
      self.postMessage({ type: "cursor", cursor });
    }
    const wanted = answered.pointerLock === true;
    const raw = answered.rawMouse !== false;
    if (wanted !== lockWanted || raw !== lockRaw) {
      lockWanted = wanted;
      lockRaw = raw;
      self.postMessage({ type: "pointer-lock", wanted, raw });
    }
    // The player's graphics settings that are the page's to carry out: fullscreen, and the power preference for the next load.
    const asks = JSON.stringify([answered.fullscreen === true, answered.lowPower === true]);
    if (asks !== pageAsks) {
      pageAsks = asks;
      self.postMessage({ type: "page", fullscreen: answered.fullscreen === true, lowPower: answered.lowPower === true });
    }
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
      // Fork s60: a probe call that renders (capture, sweep) draws its frame whole, never one with draws skipped.
      out = pipelines.drawnWhole(() => engine!.call(frame));
    } catch (e) {
      answer(refusal(frame, e instanceof Error ? e.message : String(e)), viaRing);
      crash(e instanceof Error ? e.message : String(e));
      return;
    }
    if (rendering && isPendingCapture(frame, out)) {
      retries.push({ frame, until: performance.now() + waitOf(frame), viaRing });
      schedule();
    } else answer(out, viaRing);
  };

  const retry = () => {
    if (!engine) return;
    for (let i = retries.length - 1; i >= 0; i--) {
      const r = retries[i]!;
      const out = pipelines.drawnWhole(() => engine!.call(r.frame));
      if (!isPendingCapture(r.frame, out) || performance.now() >= r.until) {
        retries.splice(i, 1);
        answer(out, r.viaRing);
      }
    }
  };

  const drainRpc = () => {
    if (engine && rpc) for (const frame of readFrames(rpc.requests)) serve(frame, true);
  };

  /** The frame cap (og.render.setDisplay, each frame's `maxFps`) and the display's measured interval (pacing.ts). */
  const pacer = new FramePacer();
  let maxFps = 0;
  let toldDisplayMs = 0;
  /** Fork s70: the page's loading screen covers the canvas; the pipelines warmed meanwhile, and whether any is left. */
  let covered = false, coveredWarm = 0, coveredWarmLeft = true;
  const frame = (now: number) => {
    scheduled = false;
    if (!engine || crashed) return;
    let drawn = true;
    try {
      // Input on every display frame, drawn or not: a capped frame's input is no later than the cap's interval.
      pumpInput(now);
      // Retries first: a capture asked this frame is requested now and answered on a later frame.
      retry();
      drainRpc();
      if (playing && retries.length === 0) {
        drawn = pacer.frame(now, maxFps);
        const d = pacer.displayMs;
        if (d > 0 && Math.abs(d - toldDisplayMs) > toldDisplayMs * 0.05) {
          toldDisplayMs = d;
          glueRef?.og_web_display_interval?.(engine.id, d);
        }
      }
      // The clock holds while a capture waits for its readback; a display
      // frame the cap skips is not drawn, and the next drawn one is handed
      // the time since the last drawn one, so the fixed tick runs on.
      if (playing && retries.length === 0 && drawn) {
        const began = performance.now();
        const answered = engine.frame(last ? now - last : 0);
        if (!answered.ok) {
          crash(answered.reason ?? "the engine refused a frame");
          return;
        }
        if (last) {
          intervals[recorded % FRAME_RECORDS] = now - last;
          work[recorded % FRAME_RECORDS] = performance.now() - began;
          frameCounter?.add(now - last, work[recorded % FRAME_RECORDS]!);
          recorded++;
        }
        behind = answered.behind ?? 0;
        if (typeof answered.maxFps === "number") maxFps = answered.maxFps;
        relay(answered);
        pumpRoom(engine);
        if (typeof answered.tick === "number") lastTick = answered.tick;
        // Fork s70: a covered frame sets no pass, so the pipelines a pass makes when it first sets one
        // (og-render warm) are made here, one a frame, at most WARM_COMPILING compiling (s75).
        if (covered && coveredWarmLeft && !warmBusy() && glueRef?.og_web_warm && pipelines.compiling() < WARM_COMPILING) {
          // Before the first frame: synchronous under prewarm (only warmBehindGate's are `warming`).
          const warm = glueRef.og_web_warm;
          coveredWarmLeft = warmPaced(() => warm(engine!.id));
          if (coveredWarmLeft) coveredWarm++;
          else reportPipelines(coveredWarm);
        }
        if (!firstFrame && answered.rendered !== false) {
          firstFrame = true;
          self.postMessage(firstFrameMessage());
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
    if (drawn) last = now;
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

  /**
   * Behind the gate, in idle time: the renderer makes the pipelines the first frame did not
   * draw with, one a task (og-render deferred.rs), until
   * none is left or play begins (a pipeline still unmade is made by the frame that first draws with it).
   */
  const warmBehindGate = () => {
    const warm = glueRef?.og_web_warm;
    if (!warm || !rendering) return;
    const began = performance.now();
    let made = 0;
    const step = () => {
      if (crashed || !engine) return;
      // Play stopped the warm-up: its counts so far, over the first frame's.
      if (playing) return reportPipelines(made);
      // Fork s60: with the pipelines asynchronous a warm call returns at once; at most WARM_COMPILING
      // (s75) compile at a time, so the ones a frame needs are not queued behind every pass's.
      if (pipelines.compiling() >= WARM_COMPILING || warmBusy()) {
        setTimeout(step, 50);
        return;
      }
      let more: boolean;
      try {
        more = warmPaced(() => pipelines.warming(() => warm(engine!.id)));
      } catch (e) {
        crash(e instanceof Error ? e.message : String(e));
        return;
      }
      if (more) {
        made++;
        setTimeout(step, 0);
      } else {
        if (made > 0) console.info(`renderer: ${made} pipelines made behind the gate in ${Math.round(performance.now() - began)} ms`);
        reportPipelines(made);
      }
    };
    setTimeout(step, 0);
  };
  /** While the first frame waits for the platform's first answers (settle.ts): a resize then draws nothing yet. */
  let settling = false;
  /** Textures still drawn from their stand-ins (s63). */
  const texturesWaiting = () => (glueRef?.og_web_textures_deferred?.() ?? 0) > 0;
  /** Whether the last frame drew less than the scene: pipelines compiling or draws skipped for them (s60), textures waiting (s63). */
  const frameWaits = () => pipelines.takeSkipped() > 0 || pipelines.compiling() > 0 || texturesWaiting();
  /** The first frame drawn again behind the gate until it draws the whole scene: how often, since when, the most pipelines compiling. */
  let waiting = { frames: 0, since: 0, most: 0, again: 0 as ReturnType<typeof setTimeout> | 0 };
  // Fork s61: a game that draws its loading screen with the page (og-config bootScreen "page") reports its
  // own stages there (`ogBootStage(id, fraction)`: og:stage on the page), and asks whether the frames still
  // wait for pipelines compiling (s60) or textures to upload (s63) before it calls its loading done (`ogBootIdle()`).
  const pageStages = () => {
    const g = globalThis as { ogBootStage?: (id: string, fraction?: number) => void; ogBootIdle?: () => boolean };
    // Fork s70: the page's loading screen hides the canvas until the game calls its loading done, so the
    // frames until then prepare without drawing (and make the pipelines a pass would make, warm above).
    covered = true;
    glueRef?.og_web_covered?.(true);
    g.ogBootStage = (id, fraction = 0) => {
      if (id === "done" && covered) {
        covered = false;
        glueRef?.og_web_covered?.(false);
        console.info(`renderer: the page uncovered the canvas at ${Math.round(performance.now())} ms (${coveredWarm} pipelines warmed meanwhile)`);
      }
      self.postMessage({ type: "og-stage", id, fraction });
    };
    g.ogBootIdle = () => pipelines.compiling() === 0 && !texturesWaiting();
  };
  /** One rendered frame without a tick: the first frame behind the play gate. */
  const present = () => {
    if (!engine || playing || settling) return;
    try {
      const answered = engine.frame(0);
      if (!answered.ok) return crash(answered.reason ?? "the engine refused a frame");
      relay(answered);
      if (!firstFrame && answered.rendered !== false) {
        // A frame that drew less than the scene is drawn again, each try its own task: once no
        // pipeline compiles (s60, polled without drawing: a frame meanwhile only slows them), or at
        // once while textures wait to upload (s63). The first frame is the first one drawn whole.
        if (frameWaits()) {
          if (!waiting.since) waiting.since = performance.now();
          waiting.frames++;
          const bar = () => {
            waiting.most = Math.max(waiting.most, pipelines.compiling());
            self.postMessage({ type: "progress", value: onBar("present", 1 - pipelines.compiling() / Math.max(1, waiting.most)), stage: "present" });
          };
          bar();
          const poll = () => {
            waiting.again = 0;
            if (pipelines.compiling() > 0) {
              bar();
              waiting.again = setTimeout(poll, 50);
            } else present();
          };
          if (!waiting.again) waiting.again = setTimeout(poll, pipelines.compiling() > 0 ? 50 : 0);
          flushLog();
          return;
        }
        if (waiting.since) console.info(`renderer: the first frame waited ${Math.round(performance.now() - waiting.since)} ms (${waiting.frames} frames behind the gate)`);
        firstFrame = true;
        phase("present");
        self.postMessage(firstFrameMessage());
        warmBehindGate();
      }
      flushLog();
    } catch (e) {
      crash(e instanceof Error ? e.message : String(e));
    }
  };

  const calls: Record<string, (e: Engine, args: unknown[]) => unknown> = {
    // A harness's ticks carry the room as the page's own frames do: lockstep's inputs leave with them.
    advance: (e, [n]) => {
      const answered = e.advance(Number(n));
      pumpRoom(e);
      return answered;
    },
    command: (e, [action, args]) => e.command(String(action), (args as string[]) ?? []),
    hash: (e) => e.hash(),
    /** What the engine holds (ProbeMemory: the wasm linear memory, the counted heap, the world, streaming, audio), in every build. */
    memory: (e) => JSON.parse(glueRef?.og_web_memory?.(e.id) ?? "null"),
    /**
     * The display frame drawn last as RGBA8 rows (the probe's Capture::frame,
     * `og render`'s every-frame capture), or null until its readback lands: the
     * first call turns the keeping of frames on; ask again before the next frame.
     */
    renderFrame: (e) => {
      const rgba = glueRef?.og_web_render_frame?.(e.id);
      return rgba && rgba.length > 0 ? rgba : null;
    },
    /** What the display draws now (og.render.display: the frame cap, render scale, automatic quality, device losses, GPU memory). */
    display: (e) => JSON.parse(boundHost()!.abi.og_render_display(e.id)),
    /** Test hook: destroy the GPU device (og_web_lose_device); the next frames make the renderer again and the game goes on. */
    loseDevice: (e) => glueRef?.og_web_lose_device?.(e.id) ?? false,
    /** What the lanes did so far (crates/og-runtime/src/lanes.rs `report`). */
    lanes: () => JSON.parse(glueRef?.og_web_lanes?.() ?? "null"),
    /** Test hook: the next item at lanes site `site` panics (og_web_lanes_fault); false when the module has no such hook or site. */
    fault: (_, [site]) => glueRef?.og_web_lanes_fault?.(String(site)) ?? false,
    frame: (e, [ms]) => {
      // Fork s60: a harness's frame is compared with the native one: nothing in it skipped for a pipeline compiling.
      const answered = pipelines.drawnWhole(() => {
        glueRef!.og_web_probe_frame(e.id, true);
        try {
          return e.frame(Number(ms));
        } finally {
          glueRef!.og_web_probe_frame(e.id, false);
        }
      });
      relay(answered);
      pumpRoom(e);
      return answered;
    },
    /** og.net on this page: og.net.status (lockstep running, its start tick, the seats, the first divergent tick) and the shared hash, as `og run --json` reports them natively (`lockstep`, `netHash`). */
    net: (e) => {
      const abi = boundHost()!.abi;
      return { status: JSON.parse(abi.og_net_status(e.id)), hash: JSON.parse(abi.og_net_hash(e.id)), room: JSON.parse(abi.og_platform_room_info(e.id)) };
    },
    // The frames played since the last reset (at most FRAME_RECORDS, oldest first); `true` resets.
    /** Field telemetry: the frames played since the last take (frames.ts FrameStats), counting from now on; null when the page's build has telemetry off. */
    telemetry: () => frameCounter?.take() ?? null,
    timings: (_, [reset]) => {
      const n = Math.min(recorded, FRAME_RECORDS);
      const at = (a: Float32Array) => Array.from({ length: n }, (_, i) => Math.round(a[(recorded - n + i) % FRAME_RECORDS]! * 1000) / 1000);
      const answer = { frames: recorded, intervals: at(intervals), work: at(work) };
      if (reset) recorded = 0;
      return answer;
    },
    /**
     * The run's trace (src/trace.ts): `start` begins one (dropping any), `stop` ends it, `take` (the
     * default) answers the document so far, a Chrome trace as `og run --trace` writes it; null when
     * none is recorded or the module cannot trace.
     */
    trace: (_, [action]) => {
      const a = String(action ?? "take");
      if (a === "start") {
        traceOn(false);
        return traceOn(true);
      }
      if (a === "stop") return traceOn(false);
      if (!trace) return null;
      takeTrace(true);
      return trace.document({ tier: lastTier, lanes: lanesState });
    },
    probe: (e, [member, ...rest]) => {
      const f = (e.probe as unknown as Record<string, unknown>)[String(member)];
      if (typeof f !== "function") throw new TypeError(`probe has no member '${String(member)}'`);
      return (f as (...a: unknown[]) => unknown)(...rest);
    },
  };

  self.onmessage = async ({ data }) => {
    // Fork s62: a copy of this Worker in the boot's Basis pool.
    if ((data as { type: string }).type === "og-basis") return basisJob(data as unknown as Parameters<typeof basisJob>[0]);
    if (data.type === "start") {
      try {
        if (data.telemetry) frameCounter = new FrameCounter();
        if (data.stages) pageStages();
        const received = new Received((data.expect?.wasm ?? 0) + (data.expect?.cooked ?? 0));
        // Fork s61: the bar runs on past the download (BOOT_SPANS): the bytes, then the transcode by
        // texture, the first tick, and the first frame's pipelines (present below).
        const progress = (value: number, stage: string) =>
          self.postMessage({ type: "progress", value: onBar(stage, value), fraction: Math.max(0, Math.min(1, value)), stage, loaded: received.loaded, total: received.total });
        progress(0, "wasm");
        phaseAt = performance.now();
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
        // Fork s63: at most UPLOAD_BUDGET texture bytes a frame; the rest in the frames after.
        glue.og_web_upload_budget?.(UPLOAD_BUDGET);
        // Fork s60: the frame pipelines compile off the browser's GPU-process main task (pipelines.ts)
        // as the build asks, or for auto (absent) on a mobile GPU, picked at the adapter: a phone's
        // browser compiles synchronous ones on its GPU process's main thread, while on Chrome/Metal a
        // cold shader cache compiles them asynchronously several times slower (docs/web-export.md §Pipelines).
        if (glue.og_web_frame_pipelines) {
          const asked = data.pipelines;
          glue.og_web_frame_pipelines(pipelines.frameSpan);
          onAdapter = (adapter) => {
            const choice = pipelines.choose(asked, adapter.info, (navigator as { userAgentData?: { mobile?: boolean } }).userAgentData?.mobile);
            pipelines.use(choice);
            console.info(`[og] pipelines ${choice.mode}: ${choice.reason}`);
          };
        }
        // Fork s70: frames prepare without drawing while the page's loading screen covers the canvas.
        if (covered) glue.og_web_covered?.(true);
        phase("wasm");
        // Terrain tiles are inflated by the browser off the engine's thread where it can.
        installInflater(glue);
        // Streamed video: WebCodecs where the browser decodes AV1, else rav1d's own wasm module (fetched only then).
        await installVideo(glue, data.av1);
        // Uncooked images (og.load.imageData) are decoded by the browser, never in the wasm.
        installImageDecoder(glue);
        progress(data.expect ? received.value : 0.1, "bundle");
        let fetched: Fetched | null = null;
        if (data.root !== undefined) {
          fetched = await preloadBundle(glue, data.root, progress, data.expect ? received : undefined);
          installFetcher(glue, fetched.manifest ?? "", data.basis, () => (rendering && engine ? engine.id : null));
        }
        phase("bundle");
        // A trace from the boot on records the boot scene's loads too.
        if (data.trace) glue.og_web_trace?.(TRACE_BEFORE_ENGINE, true);
        const created = createEngine(game, {
          ...(data.seed === undefined ? {} : { seed: data.seed }),
          ...(data.root === undefined ? {} : { root: data.root }),
          ...(data.profile === undefined ? {} : { profile: data.profile }),
          ...(data.project === undefined ? {} : { project: data.project }),
          ...(data.locale ? { locale: data.locale } : {}),
          deferStart: true,
          a11y: true,
        });
        if (!created.ok) throw new Error(created.reason);
        engine = created.value;
        if (data.trace) {
          traceOn(true);
          glue.og_web_trace?.(TRACE_BEFORE_ENGINE, false);
        }
        // og.platform: calls go to the page, which answers them (the Player, or the browser's storage).
        // The time is the browser's until the Player's answers carry its own (docs/platform-asks.md).
        engine.setClock(systemClock("browser"));
        engine.attachPlatform(toPage("none", [], platformCounter));
        if (data.viewport) glue.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
        ring = data.ring;
        rpc = data.rpc;
        phase("create");
        let tier = "none";
        if (data.canvas) {
          canvas = data.canvas;
          // WebGPU only: a rejection is the reason the renderer did not start (no fallback).
          tier = await glue.og_web_attach_canvas(engine.id, data.canvas, data.lowPower === true).catch((e: unknown) => {
            throw new Error(`WebGPU did not start: ${e instanceof Error ? e.message : String(e)}`);
          });
          rendering = true;
        }
        phase("attach");
        // Field telemetry: the device the renderer runs on, never a second adapter (gpuOf, shell/telemetry.ts).
        const gpu = data.telemetry && rendering ? deviceReport(glue, engine.id, canvas) : null;
        lastTier = tier;
        lanesState = lanes;
        // The textures go to the renderer's encodings, so after it is attached (the game starts later).
        const loaded = fetched ? await transcodeBundle(glue, rendering ? engine.id : null, fetched, data.basis, progress, data.expect ? received : undefined) : null;
        phase("transcode");
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
          ...(gpu ? { gpu } : {}),
        });
        void requests();
        if (data.autoplay) {
          playing = true;
          schedule();
        } else if (rendering) {
          let primed = false;
          if (data.prime && engine.probe.tick() === 0) {
            const first = engine.advance(1);
            if (!first.ok) throw new Error(first.reason);
            primed = true;
          }
          phase("prime");
          progress(1, "prime");
          // The platform's answers to the first tick's reads (the saved language, a slot's
          // meta) applied behind the gate: before the first frame while they come quickly,
          // then each settling tick drawn as it lands (settle.ts).
          const at = performance.now();
          const behindGate = {
            stop: () => playing || crashed,
            pending: () => engine?.platformPending() ?? 0,
            advance: () => (engine ? engine.advance(1) : { ok: false, reason: "no engine" }),
          };
          if (primed) {
            settling = true;
            try {
              await settle(platformCounter, behindGate.advance, { until: at + SETTLE_FIRST_MS, stop: behindGate.stop, pending: behindGate.pending });
            } finally {
              settling = false;
            }
          }
          phase("settle");
          present();
          if (primed && (platformCounter.waiting > 0 || platformCounter.fresh || behindGate.pending() > 0)) {
            void settle(platformCounter, behindGate.advance, { until: at + SETTLE_MS, stop: behindGate.stop, pending: behindGate.pending, drawn: present })
              .then((n) => n > 0 && console.info(`[og] behind the gate: ${n} settling tick(s) applied answers that landed after the first frame`))
              .catch((e: unknown) => crash(e instanceof Error ? e.message : String(e)));
          }
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
    if (data.type === "fullscreen") glueRef?.og_web_fullscreen?.(engine.id, data.on);
    if (data.type === "mouse-report") glueRef?.og_web_mouse_report?.(engine.id, data.active, data.unsupported);
    if (data.type === "keyboard-layout") boundHost()?.abi.og_input_set_keyboard_layout(JSON.stringify(data.labels));
    if (data.type === "audio-latency") audio?.latency(data.latency);
    if (data.type === "audio" && glueRef && !audio) {
      const attached = WorkerAudio.attach(glueRef, engine.id, data);
      if (attached.ok) audio = attached.value;
      else log(`warn: audio: ${attached.reason}`);
    }
    if (data.type === "rpc") serve(data.frame, false);
    if (data.type === "platform-answer") {
      engine.answerPlatform(data.request, data.answer);
      platformCounter.answered();
    }
    if (data.type === "platform-notice") engine.notifyPlatform(data.notice);
    if (data.type === "platform-host") engine.attachPlatform({ ...toPage(data.host, data.capabilities, platformCounter), time: data.time });
    if (data.type === "room-host") {
      boundHost()?.abi.og_room_set_host(engine.id, JSON.stringify({ host: data.host, ...(data.join ? { join: data.join } : {}) }));
      // A Play link's join leaves now, not with the next frame (the page may be behind its gate).
      pumpRoom(engine);
    }
    if (data.type === "room-events") {
      const abi = boundHost()?.abi;
      for (const event of data.events) abi?.og_room_deliver(engine.id, JSON.stringify(event));
      pumpRoom(engine);
    }
    if (data.type === "play") {
      playing = data.on;
      last = 0;
      pacer.reset();
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
