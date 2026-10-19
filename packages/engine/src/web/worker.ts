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
// then every blob in parallel), transcodes its Basis textures with the
// vendored transcoder (basis.ts) and hands everything to the wasm, so every
// load of the bundle completes at once; later files are fetched on request.
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
// frame; { type: "crash", message } when the engine fails after ready (a
// trap, a failing system, an uncaught error); { type: "log", lines } with
// the engine's log and the Worker's console (the crash card shows the last
// 200); { type: "result", id, value | error } for `call`; { type: "rpc",
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
import { WorkerAudio, type AudioMessage, type WebAudioGlue } from "./audio.ts";
import { readFrames, waitForFrames, writeFrame, type RpcRings } from "./rpc-ring.ts";
import {
  compressionsFromWebGl2,
  compressionsFromWebGpu,
  loadBasisTranscoder,
  pickFormat,
  transcodeKtx2,
  type BasisTranscoder,
  type Compression,
  type TextureFormat,
} from "./basis.ts";

interface Start {
  type: "start";
  glue: string;
  wasm: string;
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

type Viewport = { width: number; height: number; dpr: number; safeArea?: Record<string, number>; coarsePointer?: boolean };

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
  | AudioMessage;

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
  og_web_set_fetcher(fetch: (path: string) => void): void;
  og_web_bundle_files(manifest: string): string;
  og_web_texture_ktx2(manifest: string, id: string, bytes: Uint8Array): Uint8Array;
  og_web_attach_canvas(engine: number, canvas: OffscreenCanvas, webgl: boolean): Promise<string>;
  og_web_set_viewport(engine: number, viewport: string): string;
  og_web_resize?(engine: number, width: number, height: number): void;
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

async function bytesOf(url: string, received?: Received, onChunk?: () => void): Promise<Uint8Array> {
  const r = await fetch(url);
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

/** The block compressions this device's GPU takes (for the format a texture would be transcoded to). */
async function deviceCompressions(webgl: boolean): Promise<Set<Compression>> {
  try {
    if (!webgl) {
      const gpu = (globalThis as { navigator?: { gpu?: { requestAdapter(): Promise<{ features: { has(f: string): boolean } } | null> } } }).navigator?.gpu;
      const adapter = gpu ? await gpu.requestAdapter() : null;
      if (adapter) return compressionsFromWebGpu(adapter.features);
    }
    const Canvas = (globalThis as { OffscreenCanvas?: new (w: number, h: number) => { getContext(k: string): { getSupportedExtensions(): string[] | null } | null } }).OffscreenCanvas;
    const gl = Canvas ? new Canvas(1, 1).getContext("webgl2") : null;
    return compressionsFromWebGl2(gl?.getSupportedExtensions() ?? []);
  } catch {
    return new Set();
  }
}

export interface Preloaded {
  readonly files: number;
  readonly failed: number;
  /** Basis textures transcoded and handed to the wasm. */
  readonly textures: number;
  /** The GPU format this device would take; RGBA8 is what is uploaded until og-render accepts compressed formats. */
  readonly textureFormat: TextureFormat;
}

/**
 * Fetch the bundle under `root` into the wasm: the manifest, every blob, and
 * every Basis texture transcoded to RGBA8 (level 0).
 */
export async function preloadBundle(
  glue: WebExtras,
  root: string,
  basis: { js: string; wasm: string } | undefined,
  compressions: ReadonlySet<Compression>,
  progress: (value: number, stage: string) => void,
  received?: Received,
): Promise<Preloaded> {
  const dir = cookedDir(root);
  const manifestPath = `${dir}/.cook.ogt`;
  let manifest: string;
  try {
    manifest = new TextDecoder().decode(await bytesOf(manifestPath));
  } catch (e) {
    glue.og_web_fail_file(manifestPath, `cannot fetch ${manifestPath} (${e instanceof Error ? e.message : String(e)}); run og cook and serve cooked/ next to the page`);
    return { files: 0, failed: 1, textures: 0, textureFormat: pickFormat(compressions) };
  }
  glue.og_web_put_file(manifestPath, new TextEncoder().encode(manifest));
  const listed = JSON.parse(glue.og_web_bundle_files(manifest)) as Array<{ id: string; file: string; type: string }> | { error: string };
  if (!Array.isArray(listed)) throw new Error(`${manifestPath}: ${listed.error}`);
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
  const textures = listed.filter((o) => o.type === "og.Texture" && blobs.has(o.id));
  let transcoder: BasisTranscoder | null = null;
  let textureFormat: TextureFormat = pickFormat(compressions);
  let transcoded = 0;
  for (const [i, t] of textures.entries()) {
    const ktx2 = glue.og_web_texture_ktx2(manifest, t.id, blobs.get(t.id)!);
    if (ktx2.length > 0) {
      transcoder ??= basis ? await loadBasisTranscoder(basis.js, basis.wasm) : await loadBasisTranscoder();
      // og-render uploads RGBA8 today: transcode to it; report what the device would take.
      const out = transcodeKtx2(transcoder, ktx2, "rgba8");
      textureFormat = pickFormat(compressions, out.kind);
      glue.og_web_put_texture(t.id, out.width, out.height, out.levels[0]!, out.srgb);
      transcoded++;
    }
    if (!received) progress(0.8 + 0.2 * ((i + 1) / textures.length), "transcode");
  }
  return { files: listed.length - failed, failed, textures: transcoded, textureFormat };
}

/** A `capture` answered null: on the web the readback finishes on a later call. */
function isPendingCapture(frame: string, answer: string): boolean {
  try {
    const request = JSON.parse(frame) as { method?: string };
    if (request.method !== "capture") return false;
    const response = JSON.parse(answer) as { result?: unknown; error?: unknown };
    return response.error === undefined && response.result === null;
  } catch {
    return false;
  }
}

/** The most log lines the Worker keeps for a crash card. */
export const LOG_LINES = 200;

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
    try {
      flushLog();
    } catch {
      /* the engine may be the thing that broke */
    }
    self.postMessage(ready ? { type: "crash", message } : { type: "error", message });
  };
  self.addEventListener?.("error", (e) => crash(e.message ?? String(e.error)));
  self.addEventListener?.("unhandledrejection", (e) => crash(e.reason instanceof Error ? e.reason.message : String(e.reason)));

  /** Real time not yet simulated after the last frame (its answer's `behind`), ms. */
  let behind = 0;
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

  const answer = (frame: string, viaRing: boolean) => {
    if (!viaRing || !rpc || !writeFrame(rpc.answers, frame)) self.postMessage({ type: "rpc", frame });
  };

  /** One request frame: answered now, or kept for a retry when it is a capture whose readback has not landed. */
  const serve = (frame: string, viaRing: boolean) => {
    if (!engine) return;
    let out: string;
    try {
      out = engine.call(frame);
    } catch (e) {
      const id = (() => {
        try {
          return (JSON.parse(frame) as { id?: unknown }).id ?? null;
        } catch {
          return null;
        }
      })();
      answer(JSON.stringify({ jsonrpc: "2.0", id, error: { code: -32603, message: e instanceof Error ? e.message : String(e) } }), viaRing);
      crash(e instanceof Error ? e.message : String(e));
      return;
    }
    if (rendering && isPendingCapture(frame, out)) {
      retries.push({ frame, left: 60, viaRing });
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
        const answered = engine.frame(last ? now - last : 0);
        if (!answered.ok) {
          crash(answered.reason ?? "the engine refused a frame");
          return;
        }
        behind = answered.behind ?? 0;
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
    frame: (e, [ms]) => e.frame(Number(ms)),
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
        const glue = (await import(data.glue)) as WebGlue & WebExtras & WebAudioGlue;
        glueRef = glue;
        // The wasm streams through a counting body, so its bytes move the bar while it compiles.
        let wasm: unknown = data.wasm;
        if (data.expect?.wasm && typeof Response === "function" && typeof ReadableStream === "function") {
          const r = await fetch(data.wasm);
          if (!r.ok || !r.body) throw new Error(`cannot fetch ${data.wasm} (HTTP ${r.status})`);
          const reader = r.body.getReader();
          const counted = new ReadableStream<Uint8Array>({
            async pull(controller) {
              const { done, value } = await reader.read();
              if (done) return controller.close();
              received.loaded += value.length;
              progress(received.value, "wasm");
              controller.enqueue(value);
            },
          });
          wasm = new Response(counted, { headers: { "content-type": "application/wasm" } });
        }
        await loadWebHost(glue, wasm);
        progress(data.expect ? received.value : 0.1, "bundle");
        let loaded: Preloaded | null = null;
        if (data.root !== undefined) {
          const compressions = await deviceCompressions(!!data.webgl);
          loaded = await preloadBundle(glue, data.root, data.basis, compressions, progress, data.expect ? received : undefined);
          glue.og_web_set_fetcher((path: string) => {
            bytesOf(path).then(
              (bytes) => glue.og_web_put_file(path, bytes),
              (e) => glue.og_web_fail_file(path, `cannot fetch ${path} (${e instanceof Error ? e.message : String(e)})`),
            );
          });
        }
        const created = createEngine(game, {
          ...(data.seed === undefined ? {} : { seed: data.seed }),
          ...(data.root === undefined ? {} : { root: data.root }),
          ...(data.profile === undefined ? {} : { profile: data.profile }),
          ...(data.locale ? { locale: data.locale } : {}),
          deferStart: true,
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
    if (!engine || crashed) return;
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
