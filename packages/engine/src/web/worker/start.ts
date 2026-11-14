// The engine Worker's boot (worker.ts), on the shell's `start`: the wasm
// (the lanes variant when the shell chose it, else the single-threaded
// build), the game's cooked bundle, the engine, the renderer on the canvas
// and the bundle's textures; then `ready`, and either play or the first
// frame behind the gate, the platform's first answers applied.

import { boundHost, createEngine, isPublishBuild, systemClock, type GameToken } from "../../runtime.ts";
import { SourceMap } from "../../sourcemap.ts";
import { useSourceMap } from "../../spawn-site.ts";
import { type Fetched, Received, TRACE_BEFORE_ENGINE, UPLOAD_BUDGET, deviceReport, installFetcher, installImageDecoder, installInflater, installVideo, onBar, preloadBundle, transcodeBundle } from "../boot.ts";
import { FrameCounter } from "../frames.ts";
import { loadWebHost } from "../host.ts";
import * as pipelines from "../pipelines.ts";
import { settle, SETTLE_FIRST_MS, SETTLE_MS } from "../settle.ts";
import type { Gate } from "./gate.ts";
import { onDeviceAdapter } from "./gpu.ts";
import type { FrameLoop } from "./loop.ts";
import type { Start, WorkerScope } from "./messages.ts";
import { toPage } from "./platform.ts";
import type { Glue, Run } from "./run.ts";

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

/** Boots `game` as `data` asks; a failure is the run's crash (an `error` before ready). */
export async function start(game: GameToken, data: Start, run: Run, gate: Gate, loop: FrameLoop): Promise<void> {
  try {
    if (data.telemetry) run.frameCounter = new FrameCounter();
    if (data.stages) gate.pageStages();
    const received = new Received((data.expect?.wasm ?? 0) + (data.expect?.cooked ?? 0));
    // Fork s61: the bar runs on past the download (BOOT_SPANS): the bytes, then the transcode by
    // texture, the first tick, and the first frame's pipelines (present below).
    const progress = (value: number, stage: string) =>
      self.postMessage({ type: "progress", value: onBar(stage, value), fraction: Math.max(0, Math.min(1, value)), stage, loaded: received.loaded, total: received.total });
    progress(0, "wasm");
    run.boot.restart();
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
    let glue: Glue | null = null;
    if (data.lanes) {
      // The bar's total counts the lanes wasm in place of the
      // single-threaded one; after a failure it counts what the lanes
      // wasm downloaded plus the single-threaded wasm, once on every path.
      const total = received.total;
      const loaded = received.loaded;
      const lanesBytes = data.lanes.bytes ?? data.expect?.wasm;
      try {
        const g = (await import(data.lanes.glue)) as Glue;
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
      glue = (await import(data.glue)) as Glue;
      await loadWebHost(glue, await counted(data.wasm, data.expect?.wasm));
    }
    run.glue = glue;
    // Fork s63: at most UPLOAD_BUDGET texture bytes a frame; the rest in the frames after.
    glue.og_web_upload_budget?.(UPLOAD_BUDGET);
    // Fork s60: the frame pipelines compile off the browser's GPU-process main task (pipelines.ts)
    // as the build asks, or for auto (absent) on a mobile GPU, picked at the adapter: a phone's
    // browser compiles synchronous ones on its GPU process's main thread, while on Chrome/Metal a
    // cold shader cache compiles them asynchronously several times slower (docs/web-export.md §Pipelines).
    if (glue.og_web_frame_pipelines) {
      const asked = data.pipelines;
      glue.og_web_frame_pipelines(pipelines.frameSpan);
      onDeviceAdapter((adapter) => {
        const choice = pipelines.choose(asked, adapter.info, (navigator as { userAgentData?: { mobile?: boolean } }).userAgentData?.mobile);
        pipelines.use(choice);
        console.info(`[og] pipelines ${choice.mode}: ${choice.reason}`);
      });
    }
    // Fork s70: frames prepare without drawing while the page's loading screen covers the canvas.
    if (gate.covered) glue.og_web_covered?.(true);
    run.boot.phase("wasm");
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
      installFetcher(glue, fetched.manifest ?? "", data.basis, () => (run.rendering && run.engine ? run.engine.id : null));
    }
    run.boot.phase("bundle");
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
    const engine = created.value;
    run.engine = engine;
    // A dev build serves its bundle's map beside it: the game's spawns name their lines with it (spawn-site.ts).
    const sourceMap = isPublishBuild() === true ? null : loadOwnSourceMap();
    if (data.trace) {
      run.trace.turn(engine, true);
      boundHost()?.abi.og_engine_trace(TRACE_BEFORE_ENGINE, false);
    }
    // og.platform: calls go to the page, which answers them (the Player, or the browser's storage).
    // The time is the browser's until the Player's answers carry its own (docs/platform-asks.md).
    engine.setClock(systemClock("browser"));
    engine.attachPlatform(toPage("none", [], run.platformCounter));
    if (data.viewport) glue.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
    run.ring = data.ring;
    run.probes.rings = data.rpc;
    run.boot.phase("create");
    let tier = "none";
    if (data.canvas) {
      run.canvas = data.canvas;
      // WebGPU only: a rejection is the reason the renderer did not start (no fallback).
      tier = await glue.og_web_attach_canvas(engine.id, data.canvas, data.lowPower === true).catch((e: unknown) => {
        throw new Error(`WebGPU did not start: ${e instanceof Error ? e.message : String(e)}`);
      });
      run.rendering = true;
    }
    run.boot.phase("attach");
    // Field telemetry: the device the renderer runs on, never a second adapter (gpuOf, shell/telemetry.ts).
    const gpu = data.telemetry && run.rendering ? deviceReport(glue, engine.id, run.canvas) : null;
    run.lastTier = tier;
    run.lanes = lanes;
    // The textures go to the renderer's encodings, so after it is attached (the game starts later).
    const loaded = fetched ? await transcodeBundle(glue, run.rendering ? engine.id : null, fetched, data.basis, progress, data.expect ? received : undefined) : null;
    run.boot.phase("transcode");
    await sourceMap;
    progress(1, "ready");
    run.ready = true;
    self.postMessage({
      type: "ready",
      engine: engine.id,
      tier,
      transport: run.ring ? "ring" : "message",
      rpc: run.probes.rings ? "ring" : "message",
      textureFormat: loaded?.textureFormat ?? null,
      files: loaded?.files ?? 0,
      textures: loaded?.textures ?? 0,
      lanes,
      ...(gpu ? { gpu } : {}),
    });
    void loop.requests();
    if (data.autoplay) {
      run.playing = true;
      loop.schedule();
    } else if (run.rendering) {
      let primed = false;
      if (data.prime && engine.probe.tick() === 0) {
        const first = engine.advance(1);
        if (!first.ok) throw new Error(first.reason);
        primed = true;
      }
      run.boot.phase("prime");
      progress(1, "prime");
      // The platform's answers to the first tick's reads (the saved language, a slot's
      // meta) applied behind the gate: before the first frame while they come quickly,
      // then each settling tick drawn as it lands (settle.ts).
      const at = performance.now();
      const behindGate = {
        stop: () => run.playing || run.crashed,
        pending: () => run.engine?.platformPending() ?? 0,
        advance: () => (run.engine ? run.engine.advance(1) : { ok: false, reason: "no engine" }),
      };
      if (primed) {
        gate.settling = true;
        try {
          await settle(run.platformCounter, behindGate.advance, { until: at + SETTLE_FIRST_MS, stop: behindGate.stop, pending: behindGate.pending });
        } finally {
          gate.settling = false;
        }
      }
      run.boot.phase("settle");
      gate.present();
      if (primed && (run.platformCounter.waiting > 0 || run.platformCounter.fresh || behindGate.pending() > 0)) {
        void settle(run.platformCounter, behindGate.advance, { until: at + SETTLE_MS, stop: behindGate.stop, pending: behindGate.pending, drawn: () => gate.present() })
          .then((n) => n > 0 && console.info(`[og] behind the gate: ${n} settling tick(s) applied answers that landed after the first frame`))
          .catch((e: unknown) => run.crash(e instanceof Error ? e.message : String(e)));
      }
    }
  } catch (e) {
    run.crash(e instanceof Error ? e.message : String(e));
  }
}
