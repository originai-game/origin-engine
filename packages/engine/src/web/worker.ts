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
// Messages to the shell: { type: "progress", value: 0..1, stage }, then
// { type: "ready", engine, tier, transport, rpc, textureFormat, files,
// textures } or { type: "error", message }; { type: "result", id, value |
// error } for `call`; { type: "rpc", frame } for an answer that did not cross
// the ring.

import { createEngine, type Engine, type GameToken } from "../runtime.ts";
import { loadWebHost, type WebGlue } from "./host.ts";
import { drainRing, type InjectedInput } from "./input.ts";
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
  /** The play surface in CSS pixels, its DPR and safe area: what the UI lays out in. */
  viewport?: { width: number; height: number; dpr: number; safeArea?: Record<string, number>; coarsePointer?: boolean };
}

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
  | { type: "viewport"; viewport: NonNullable<Start["viewport"]> }
  | AudioMessage;

interface WorkerScope {
  onmessage: ((e: { data: Message }) => void) | null;
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
}

/** Where the runtime asks for a bundle's files: `<root>/cooked/<file>` (crates/og-api/src/runtime/assets.rs). */
export function cookedDir(root: string): string {
  return `${root.replace(/\/+$/, "")}/cooked`;
}

async function bytesOf(url: string): Promise<Uint8Array> {
  const r = await fetch(url);
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return new Uint8Array(await r.arrayBuffer());
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
        const bytes = await bytesOf(path);
        glue.og_web_put_file(path, bytes);
        blobs.set(o.id, bytes);
      } catch (e) {
        failed++;
        glue.og_web_fail_file(path, `cannot fetch ${path} (${e instanceof Error ? e.message : String(e)})`);
      }
      progress(0.1 + 0.7 * (++fetched / Math.max(1, listed.length)), "fetch");
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
    progress(0.8 + 0.2 * ((i + 1) / textures.length), "transcode");
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

/** Run `game` in this Worker, driven by the shell's messages. */
export function startWorker(game: GameToken): void {
  let engine: Engine | null = null;
  let ring: SharedArrayBuffer | undefined;
  let rpc: RpcRings | undefined;
  let rendering = false;
  let playing = false;
  let scheduled = false;
  let last = 0;
  let glueRef: (WebGlue & WebExtras & WebAudioGlue) | null = null;
  let audio: WorkerAudio | null = null;
  /** Captures waiting for their readback (requested on tick N, answered on N + 1). */
  const retries: Array<{ frame: string; left: number; viaRing: boolean }> = [];

  const pumpInput = () => {
    if (engine && ring) for (const event of drainRing(ring)) engine.probe.input(event);
  };

  const answer = (frame: string, viaRing: boolean) => {
    if (!viaRing || !rpc || !writeFrame(rpc.answers, frame)) self.postMessage({ type: "rpc", frame });
  };

  /** One request frame: answered now, or kept for a retry when it is a capture whose readback has not landed. */
  const serve = (frame: string, viaRing: boolean) => {
    if (!engine) return;
    const out = engine.call(frame);
    if (rendering && isPendingCapture(frame, out)) retries.push({ frame, left: 60, viaRing });
    else answer(out, viaRing);
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
    if (!engine) return;
    pumpInput();
    // Retries first: a capture asked this frame is requested now and answered on a later frame.
    retry();
    drainRpc();
    // The clock holds while a capture waits for its readback.
    if (playing && retries.length === 0) {
      const answered = engine.frame(last ? now - last : 0);
      if (!answered.ok) {
        playing = false;
        self.postMessage({ type: "error", message: answered.reason });
        return;
      }
      audio?.pump();
    }
    last = now;
    if (playing || retries.length > 0) schedule();
  };
  const schedule = () => {
    if (scheduled) return;
    scheduled = true;
    if (self.requestAnimationFrame) self.requestAnimationFrame(frame);
    else setTimeout(() => frame(performance.now()), 16);
  };

  /** While the clock is not playing, requests are still answered between frames. */
  const idle = async () => {
    while (rpc) {
      await waitForFrames(rpc.requests, 100);
      if (!playing && engine) {
        drainRpc();
        if (retries.length > 0) schedule();
      }
    }
  };

  const calls: Record<string, (e: Engine, args: unknown[]) => unknown> = {
    advance: (e, [n]) => e.advance(Number(n)),
    command: (e, [action, args]) => e.command(String(action), (args as string[]) ?? []),
    hash: (e) => e.hash(),
    frame: (e, [ms]) => e.frame(Number(ms)),
    probe: (e, [member, ...rest]) => (e.probe as unknown as Record<string, (...a: unknown[]) => unknown>)[String(member)]!(...rest),
  };

  self.onmessage = async ({ data }) => {
    if (data.type === "start") {
      try {
        const progress = (value: number, stage: string) => self.postMessage({ type: "progress", value, stage });
        progress(0, "wasm");
        const glue = (await import(data.glue)) as WebGlue & WebExtras & WebAudioGlue;
        glueRef = glue;
        await loadWebHost(glue, data.wasm);
        progress(0.1, "bundle");
        let loaded: Preloaded | null = null;
        if (data.root !== undefined) {
          const compressions = await deviceCompressions(!!data.webgl);
          loaded = await preloadBundle(glue, data.root, data.basis, compressions, progress);
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
          deferStart: true,
        });
        if (!created.ok) throw new Error(created.reason);
        engine = created.value;
        if (data.viewport) glue.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
        ring = data.ring;
        rpc = data.rpc;
        let tier = "none";
        if (data.canvas) {
          tier = await glue.og_web_attach_canvas(engine.id, data.canvas, !!data.webgl);
          rendering = true;
        }
        progress(1, "ready");
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
        void idle();
        if (data.autoplay) {
          playing = true;
          schedule();
        }
      } catch (e) {
        self.postMessage({ type: "error", message: e instanceof Error ? e.message : String(e) });
      }
      return;
    }
    if (!engine) return;
    if (data.type === "input") engine.probe.input(data.event);
    if (data.type === "viewport" && glueRef) glueRef.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
    if (data.type === "audio" && glueRef && !audio) {
      const attached = WorkerAudio.attach(glueRef, engine.id, data);
      if (attached.ok) audio = attached.value;
      else self.postMessage({ type: "error", message: `audio: ${attached.reason}` });
    }
    if (data.type === "rpc") {
      serve(data.frame, false);
      if (retries.length > 0) schedule();
    }
    if (data.type === "play") {
      playing = data.on;
      last = 0;
      if (playing) schedule();
    }
    if (data.type === "call") {
      try {
        pumpInput();
        const call = calls[data.method];
        if (!call) throw new Error(`unknown call '${data.method}'`);
        self.postMessage({ type: "result", id: data.id, value: call(engine, data.args) });
      } catch (e) {
        self.postMessage({ type: "result", id: data.id, error: e instanceof Error ? e.message : String(e) });
      }
    }
  };
}
