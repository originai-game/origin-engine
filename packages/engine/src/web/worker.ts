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
// of the bundle did not arrive (loadFailure: field telemetry's load failures); { type: "cursor", cursor, image } when the UI's pointer shape changed (og.ui.frameCursor, and its CSS url() image or null); { type: "pointer-lock", wanted } when og.PointerLock.wanted
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

import { boundHost, createEngine, isPublishBuild, systemClock, type Engine, type GameToken } from "../runtime.ts";
import { SourceMap } from "../sourcemap.ts";
import { useSourceMap } from "../spawn-site.ts";
import type { PlatformAnswer, PlatformHost, PlatformNotice } from "../platform-local.ts";
import { loadWebHost, type WebExtras, type WebGlue } from "./host.ts";
import { FrameCounter } from "./frames.ts";
import { BootClock, EngineTrace, FrameRecords, PageRelay, ProbeRequests } from "./worker-state.ts";
export type { BootPhases } from "./worker-state.ts";
import { FramePacer } from "./pacing.ts";
import { drainRing, stampAt, type InjectedInput } from "./input.ts";
import { WorkerAudio, type AudioLatencyMessage, type AudioMessage, type WebAudioGlue } from "./audio.ts";
import { waitForFrames, type RpcRings } from "./rpc-ring.ts";
import type { HostError } from "./shell.ts";
import { PlatformCounter, settle, SETTLE_FIRST_MS, SETTLE_MS } from "./settle.ts";
import * as pipelines from "./pipelines.ts";
import * as gpuErrors from "./gpu-errors.ts";
import { type Fetched, Received, TRACE_BEFORE_ENGINE, UPLOAD_BUDGET, basisJob, deviceReport, installFetcher, installImageDecoder, installInflater, installVideo, onBar, preloadBundle, transcodeBundle } from "./boot.ts";

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

type Viewport = { width: number; height: number; dpr: number; safeArea?: Record<string, number>; smallWidth?: number; smallHeight?: number; largeWidth?: number; largeHeight?: number; covered?: Record<string, number>; coarsePointer?: boolean; reducedMotion?: boolean; highContrast?: boolean; reducedTransparency?: boolean; macKeys?: boolean };

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
  /** A page's Worker's script URL; absent in bun's Worker, which hosts this module in the tests. */
  readonly location?: { readonly href: string; readonly pathname: string };
}

declare const self: WorkerScope;

/** This Worker's bundle's map, served beside it in a dev build, for the spawns' lines; nothing when it is not there. */
async function loadOwnSourceMap(): Promise<void> {
  const at = self.location;
  if (!at) return;
  const bundle = at.href;
  try {
    const r = await fetch(new URL(`${at.pathname.slice(at.pathname.lastIndexOf("/") + 1)}.map`, bundle));
    if (r.ok) useSourceMap(bundle, new SourceMap(await r.text()));
  } catch {
    // No map: spawned entities have no source.
  }
}

/**
 * Fork s75: the warmed pipelines compiling at once (s60/s70 warm). Chrome compiles
 * `createRenderPipelineAsync` off its GPU-process main thread, so two at a time left
 * it idle: the warm tail after the 1072-cell boot was 4-8 s at two.
 */
const WARM_COMPILING = 6;
/** The most log lines the Worker keeps for a crash card. */
export const LOG_LINES = 200;

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
  const post = (message: Record<string, unknown>) => self.postMessage(message);
  /** Where the boot went (BootPhases), sent with the first frame. */
  const boot = new BootClock();
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
  /** What the page was last told of a frame's answer. */
  const page = new PageRelay(post);
  /** The played frames' timings (the shell's `timings` call). */
  const records = new FrameRecords();
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

  /** The run's trace while one is recorded. */
  const trace = new EngineTrace();
  const relay = (answered: ReturnType<Engine["frame"]>) => {
    trace.take(engine);
    page.relay(answered);
  };
  /** The probe's requests: answered between frames, a pending capture on a later one. */
  const probes = new ProbeRequests({ engine: () => engine, rendering: () => rendering, crash: (message) => crash(message), schedule: () => schedule() }, post);

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
      probes.retry();
      probes.drain();
      if (playing && probes.waiting === 0) {
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
      if (playing && probes.waiting === 0 && drawn) {
        const began = performance.now();
        const answered = engine.frame(last ? now - last : 0);
        if (!answered.ok) {
          // The tick a system threw in, which a catch-up frame reaches past the last frame's.
          if (typeof answered.tick === "number") lastTick = answered.tick;
          crash(answered.reason ?? "the engine refused a frame");
          return;
        }
        if (last) {
          // As the records keep it (single precision), so both count the same frame.
          const spent = Math.fround(performance.now() - began);
          records.add(now - last, spent);
          frameCounter?.add(now - last, spent);
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
          post(boot.firstFrame());
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
    if (playing || probes.waiting > 0) schedule();
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
    while (probes.rings && !crashed) {
      await waitForFrames(probes.rings.requests, 1000);
      if (!engine) continue;
      try {
        pumpInput(performance.now());
        probes.drain();
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
  /** Textures still drawn from their stand-ins (s63), or font-pack faces the locale's chain still fetches. */
  const texturesWaiting = () => (glueRef?.og_web_textures_deferred?.() ?? 0) > 0 || (glueRef?.og_web_fonts_pending?.() ?? 0) > 0;
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
        boot.phase("present");
        post(boot.firstFrame());
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
    /** The files the command channel wrote since the last call (a script's `dump_*`, as `og run --commands` writes them). */
    outputs: (e) => e.takeOutputs(),
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
    /** What the lanes did so far (crates/og-runtime/src/author/lanes.rs `report`). */
    lanes: () => JSON.parse(glueRef?.og_web_lanes?.() ?? "null"),
    /** Test hook: the next item at lanes site `site` panics (og_web_lanes_fault); false when the module has no such hook or site. */
    fault: (_, [site]) => glueRef?.og_web_lanes_fault?.(String(site)) ?? false,
    frame: (e, [ms]) => {
      // Fork s60: a harness's frame is compared with the native one: nothing in it skipped for a pipeline compiling.
      // Its time is the harness's and the machine's, not the display's: automatic quality must not read it
      // (a loaded machine stepped the same script down a render-scale rung at a different tick each run).
      const answered = pipelines.drawnWhole(() => {
        glueRef?.og_web_scripted_frame?.(e.id, true);
        try {
          return e.frame(Number(ms));
        } finally {
          glueRef?.og_web_scripted_frame?.(e.id, false);
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
    // The frames played since the last reset (oldest first); `true` resets.
    /** Field telemetry: the frames played since the last take (frames.ts FrameStats), counting from now on; null when the page's build has telemetry off. */
    telemetry: () => frameCounter?.take() ?? null,
    timings: (_, [reset]) => records.take(Boolean(reset)),
    /**
     * The run's trace (src/trace.ts): `start` begins one (dropping any), `stop` ends it, `take` (the
     * default) answers the document so far, a Chrome trace as `og run --trace` writes it; null when
     * none is recorded or the module cannot trace.
     */
    trace: (_, [action]) => {
      const a = String(action ?? "take");
      if (a === "start") {
        trace.turn(engine, false);
        return trace.turn(engine, true);
      }
      if (a === "stop") return trace.turn(engine, false);
      return trace.document(engine, { tier: lastTier, lanes: lanesState });
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
        boot.restart();
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
        boot.phase("wasm");
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
        boot.phase("bundle");
        // A trace from the boot on records the boot scene's loads too.
        if (data.trace) boundHost()?.abi.og_engine_trace(TRACE_BEFORE_ENGINE, true);
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
        // A dev build serves its bundle's map beside it: the game's spawns name their lines with it (spawn-site.ts).
        const sourceMap = isPublishBuild() === true ? null : loadOwnSourceMap();
        if (data.trace) {
          trace.turn(engine, true);
          boundHost()?.abi.og_engine_trace(TRACE_BEFORE_ENGINE, false);
        }
        // og.platform: calls go to the page, which answers them (the Player, or the browser's storage).
        // The time is the browser's until the Player's answers carry its own (docs/platform-asks.md).
        engine.setClock(systemClock("browser"));
        engine.attachPlatform(toPage("none", [], platformCounter));
        if (data.viewport) glue.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
        ring = data.ring;
        probes.rings = data.rpc;
        boot.phase("create");
        let tier = "none";
        if (data.canvas) {
          canvas = data.canvas;
          // WebGPU only: a rejection is the reason the renderer did not start (no fallback).
          tier = await glue.og_web_attach_canvas(engine.id, data.canvas, data.lowPower === true).catch((e: unknown) => {
            throw new Error(`WebGPU did not start: ${e instanceof Error ? e.message : String(e)}`);
          });
          rendering = true;
        }
        boot.phase("attach");
        // Field telemetry: the device the renderer runs on, never a second adapter (gpuOf, shell/telemetry.ts).
        const gpu = data.telemetry && rendering ? deviceReport(glue, engine.id, canvas) : null;
        lastTier = tier;
        lanesState = lanes;
        // The textures go to the renderer's encodings, so after it is attached (the game starts later).
        const loaded = fetched ? await transcodeBundle(glue, rendering ? engine.id : null, fetched, data.basis, progress, data.expect ? received : undefined) : null;
        boot.phase("transcode");
        await sourceMap;
        progress(1, "ready");
        ready = true;
        self.postMessage({
          type: "ready",
          engine: engine.id,
          tier,
          transport: ring ? "ring" : "message",
          rpc: probes.rings ? "ring" : "message",
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
          boot.phase("prime");
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
          boot.phase("settle");
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
      if (data.type === "rpc") probes.refuse(data.frame, why, false);
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
    if (data.type === "rpc") probes.serve(data.frame, false);
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
