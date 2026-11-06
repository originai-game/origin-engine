// `og run` on the native host: load a game bundle under bun (or node) with
// the napi addon, open the project's cooked bundle, mount the probe, and
// drive the tick.
//
//   bun src/node/run.ts --entry <game.js|.ts> [--root <project>] [--ticks N] [--seed S]
//       [--setup action[:arg...]]... [--commands "t:action[:args];..."] [--out <dir>]
//       [--port P] [--size WxH] [--realtime] [--watch <bundle.js>] [--no-render]
//       [--audio null|device|<out.wav>] [--platform-store <file.json>] [--platform-fake <file.json>] [--clock <UTC ms>] [--locale <tag>]
//       [--trace <out.json> [--trace-append]] [--room ws://127.0.0.1:P/<room>] [--json]
//       [--restore <world.save>] [--handoff <world.save>]
//
// - Without --commands, --port or --watch: advance --ticks N at once and print
//   the world hash (M2's hello world).
// - With --commands: the script is scheduled through og-probe at tick 0 and
//   the run takes one tick per frame until `quit`, --ticks, or the script's
//   last step has run. Outputs (`screenshot`, `dump_*`, `help`) are written
//   under --out at the probe's `Output::file_name()`.
// - With --port: the probe answers JSON-RPC 2.0 at ws://127.0.0.1:<port>/probe
//   between frames (`og probe`); the run keeps going until `quit` or a signal.
// - With --watch: a change to the bundle re-imports it and hot-reloads it
//   through the probe's `reload` (snapshot, swap, restore), printing one JSON
//   line per reload with the latency from the file write to the first frame
//   of the new bundle.
// - With --room: the engine joins that room through `og relay` (og.platform's
//   rooms, og.net's lockstep), the link pumped around every frame, and the run
//   takes frames (never one big advance), so a tick lockstep holds waits for
//   the other players' inputs; it ends at `quit`, --ticks or the script's end.
// - Without a renderer (OG_RENDERER=none, as the platform's engine and probe
//   sandboxes set) the run is the same run: the play surface keeps --size, a
//   `screenshot` writes no PNG and is listed in --json's `unchecked` ("no
//   renderer, frame unchecked"), the tier is `none`, and the exit status does
//   not change. An unavailable adapter also takes this path, with a warning
//   naming wgpu's error. Other renderer errors still fail the run.
// - With --trace: the run in the Chrome trace format (src/trace.ts and
//   crates/og-runtime/src/trace.rs have the threads): each tick with its
//   systems, each frame with its render passes' CPU encoding and their GPU
//   time, each load, og-lanes' jobs on their pool threads, every span where
//   it ran; --trace-append adds them to the file og wrote first (its
//   metadata, its origin and the cook's spans).
// - With --handoff: SIGUSR2 makes the run write the world's snapshot (the
//   save text of og.world.snapshot) to that file between two frames, print
//   `{"event":"handoff","tick":T}` and exit 0. With --restore the run starts
//   from such a file, migrating by name what the schema changed, once
//   GameDesc.start and --setup have run. Together they are `og dev --engine`'s
//   hot restart: the engine rebuilt, a new process carries the world on. A
//   refused restore warns and the run goes on from the start.

import { copyFileSync, existsSync, mkdirSync, readFileSync, renameSync, rmSync, statSync, watch, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { attachRenderer, loadNativeHost, native } from "./index.ts";
import { openAudio, type AudioSink } from "./audio.ts";
import { nativePlatform, readFake } from "./platform.ts";
import { Trace, unixMs, type TraceDocument } from "../trace.ts";
import { beginBundle, boundHost, createEngine, fixedClock, type Engine, type GameToken, type Result } from "../runtime.ts";

const hostProcess = process;
let version = 0;
class RunExit extends Error {
  readonly code: number;
  constructor(code: number) {
    super(`run exited ${code}`);
    this.code = code;
  }
}

export interface RunOutput {
  stdout: { write(text: string): void };
  stderr: { write(text: string): void };
}

/** Runs sequentially in one host, destroying every run-owned resource on exit. */
export async function run(argv: string[], output: RunOutput = hostProcess): Promise<number> {
  const process = { stdout: output.stdout, stderr: output.stderr, version: hostProcess.version };
  const cleanup: Array<() => void> = [];
  try {
    const hostEntered = unixMs();
    let entry = "";
    let root: string | undefined;
    let ticks: number | undefined;
    let seed: number | undefined;
    let commands: string | undefined;
    let out = "og-out";
    let port: number | undefined;
    let size = { width: 1280, height: 720 };
    let realtime = false;
    let watchFile: string | undefined;
    let render = true;
    let asJson = false;
    let audioOut = "null";
    let platformStore: string | undefined;
    let platformFake: string | undefined;
    let locale: string | undefined;
    let clock: number | undefined;
    let tracePath: string | undefined;
    let traceAppend = false;
    let room: string | undefined;
    let restoreFile: string | undefined;
    let handoffFile: string | undefined;
    const setup: string[] = [];
    for (let i = 0; i < argv.length; i++) {
      const a = argv[i]!;
      const next = () => {
        const v = argv[++i];
        if (v === undefined) fail(`${a} needs a value`);
        return v!;
      };
      if (a === "--entry") entry = next();
      else if (a === "--root") root = resolve(next());
      else if (a === "--ticks") ticks = Number(next());
      else if (a === "--seed") seed = Number(next());
      else if (a === "--setup") setup.push(next());
      else if (a === "--commands") commands = next();
      else if (a === "--out") out = next();
      else if (a === "--port") port = Number(next());
      else if (a === "--size") {
        const m = /^(\d+)x(\d+)$/.exec(next());
        if (!m) fail("--size takes WIDTHxHEIGHT, such as 1280x720");
        size = { width: Number(m![1]), height: Number(m![2]) };
      } else if (a === "--realtime") realtime = true;
      else if (a === "--watch") watchFile = resolve(next());
      else if (a === "--no-render") render = false;
      else if (a === "--json") asJson = true;
      else if (a === "--audio") audioOut = next();
      else if (a === "--platform-store") platformStore = resolve(next());
      else if (a === "--platform-fake") platformFake = resolve(next());
      else if (a === "--locale") locale = next();
      else if (a === "--clock") clock = Number(next());
      else if (a === "--trace") tracePath = resolve(next());
      else if (a === "--trace-append") traceAppend = true;
      else if (a === "--room") room = next();
      else if (a === "--restore") restoreFile = resolve(next());
      else if (a === "--handoff") handoffFile = resolve(next());
      else if (a === "--headless") {
        // Native runs are always headless; accepted for `og run --headless`.
      } else fail(`unknown argument '${a}'`);
    }

    function fail(reason: string): never {
      process.stderr.write(`og run: ${reason}\n`);
      throw new RunExit(2);
    }

    if (!entry) fail("no game entry: pass --entry <file>");
    if (ticks !== undefined && (!Number.isInteger(ticks) || ticks < 0)) fail("--ticks takes a whole number of fixed steps");
    if (seed !== undefined && (!Number.isInteger(seed) || seed < 0)) fail("--seed takes a whole number");
    if (clock !== undefined && !(Number.isFinite(clock) && clock > 0)) fail("--clock takes UTC milliseconds since 1970, such as 1767225600000");
    if (port !== undefined && (!Number.isInteger(port) || port < 0 || port > 65535)) fail("--port takes 0 (a free port) to 65535");

    // --trace: the Chrome trace (src/trace.ts), continuing og's prelude (the cook) when appending.
    const trace = tracePath
      ? new Trace({
          producer: "og run --trace",
          process: "og run",
          ...(traceAppend && existsSync(tracePath) ? { prior: JSON.parse(readFileSync(tracePath, "utf8")) as Partial<TraceDocument> } : {}),
        })
      : undefined;
    function hostPhase(name: string, start: number, end = unixMs()): void {
      if (!trace) return;
      const ts = Math.round((start - trace.origin) * 1000);
      trace.events.push({ name, cat: "host", ph: "X", pid: 1, tid: 7, ts, dur: Math.max(0, Math.round((end - trace.origin) * 1000) - ts) });
    }
    if (trace) {
      trace.events.push({ name: "thread_name", ph: "M", pid: 1, tid: 7, args: { name: "host" } });
      const bundle = trace.events.find((e) => e.name === "bundle" && e.cat === "prepare");
      if (bundle) hostPhase("process-start-and-imports", trace.origin + ((bundle.ts ?? 0) + (bundle.dur ?? 0)) / 1000, hostEntered);
    }
    const addonStart = unixMs();
    loadNativeHost();
    const extras = native();
    hostPhase("addon-load", addonStart);
    /** Takes the engine's spans (each where it ran) onto the trace; `last` waits for the GPU's frames in flight. */
    function takeTrace(engineId: number, last = false): void {
      if (!trace) return;
      const taken = extras.og_node_trace_take(engineId, last);
      trace.append(taken, unixMs());
    }
    /** Keep maps until the run finishes, so errors retain their source locations. */
    const reloadMaps: string[] = [];
    cleanup.push(() => {
      for (const m of reloadMaps) rmSync(m, { force: true });
    });
    /**
     * Import a bundle as a fresh module. A re-import goes through a copy under
     * its own name: bun keys file:// modules by path and ignores a query, so
     * `game.js?v=2` would answer the module already evaluated, declaring nothing.
     */
    async function importGame(file: string): Promise<GameToken> {
      let path = resolve(file);
      if (version > 0) {
        const copy = path.replace(/(\.[cm]?[jt]s)?$/, (ext) => `.reload-${version}${ext || ".js"}`);
        copyFileSync(path, copy);
        cleanup.push(() => rmSync(copy, { force: true }));
        // bun maps a stack through `<module>.map`, read when the stack is formed: the copy's map
        // stays until the process exits, so a reloaded bundle's errors still name its .ts lines.
        if (existsSync(`${path}.map`)) {
          copyFileSync(`${path}.map`, `${copy}.map`);
          reloadMaps.push(`${copy}.map`);
        }
        path = copy;
      }
      const url = pathToFileURL(path).href;
      version++;
      const mod = (await import(url)) as { default?: GameToken };
      if (path !== resolve(file)) rmSync(path, { force: true });
      if (!mod.default) fail(`${file} has no default export: export default og.game({ ... })`);
      return mod.default!;
    }

    /** project.ogt's text, which the engine reads tick, seed, scenes and scene from (CreateOptions.project). */
    function projectText(dir: string | undefined): string | undefined {
      if (!dir || !existsSync(join(dir, "project.ogt"))) return undefined;
      return readFileSync(join(dir, "project.ogt"), "utf8");
    }

    /** project.ogt's `profile` ("2d" or "3d"), which chooses the physics profile. */
    function projectProfile(text: string | undefined): "2d" | "3d" | undefined {
      const m = text === undefined ? null : /^\s*profile\s+"(2d|3d)"/m.exec(text);
      return m ? (m[1] as "2d" | "3d") : undefined;
    }

    const importStart = unixMs();
    beginBundle();
    const game = await importGame(entry);
    hostPhase("game-import", importStart);
    // Loads made while the engine is created (its boot scene's) are traced too.
    const BEFORE_ENGINE = 0xffffffff;
    if (tracePath) {
      extras.og_node_trace(BEFORE_ENGINE, true);
      cleanup.push(() => extras.og_node_trace(BEFORE_ENGINE, false));
    }
    const project = projectText(root);
    const profile = projectProfile(project);
    const probing = commands !== undefined || port !== undefined || watchFile !== undefined || room !== undefined || handoffFile !== undefined;
    // With a renderer, GameDesc.start runs once it is attached (as the web Worker
    // does), so og.render.capabilities reports the play surface's size there.
    const deferStart = probing && render;
    const createStart = unixMs();
    const created = createEngine(game, {
      ...(seed === undefined ? {} : { seed }),
      ...(root ? { root } : {}),
      ...(project === undefined ? {} : { project }),
      ...(profile ? { profile } : {}),
      ...(locale ? { locale } : {}),
      ...(deferStart ? { deferStart } : {}),
    });
    hostPhase("engine-create", createStart);
    if (!created.ok) fail(`${entry}: ${created.reason}`);
    const engine: Engine = created.value;
    cleanup.push(() => engine.destroy());
    // Without --realtime every frame is the script's (elapsed is one tick): its time is the machine's load, not a
    // display's, so automatic quality must not read it, or the same script draws at another render scale per run.
    if (!realtime) extras.og_node_scripted_frames(engine.id, true);
    if (tracePath) {
      extras.og_node_trace(engine.id, true);
      extras.og_node_trace(BEFORE_ENGINE, false);
    }
    // og.platform: the local store, in memory unless --platform-store names a file;
    // its clock the system's, or with --clock one that starts there and moves with the ticks.
    if (clock !== undefined) engine.setClock(fixedClock(engine, clock));
    // The platform services are faked: every call succeeds unless --platform-fake scripts it.
    const fake = platformFake === undefined ? { ok: true as const, value: {} } : readFake(platformFake);
    if (!fake.ok) fail(`--platform-fake: ${fake.reason}`);
    engine.attachPlatform(nativePlatform(platformStore, fake.ok ? fake.value : {}));
    // One sink per engine, pumped after every step (crates/og-audio/README.md
    // §Host contract): null keeps the stream drained, a .wav path captures it.
    const audioMode = audioOut === "device" ? "device" : audioOut === "null" ? "null" : "capture";
    const opened = openAudio(engine.id, audioMode);
    if (!opened.ok) fail(`--audio ${audioOut}: ${opened.reason}`);
    const sink: AudioSink = opened.value;
    cleanup.push(() => sink.close());
    const outputs: string[] = [];
    const warnings: string[] = [];
    // The screenshots taken without a renderer: the probe's FRAME_UNCHECKED notes.
    const unchecked: string[] = [];
    const UNCHECKED = /^[\d.]+s screenshot: no renderer, frame unchecked \((.+)\)$/;

    function decode(base64: string): Uint8Array {
      const bin = atob(base64);
      const bytes = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      return bytes;
    }

    function drain(): void {
      for (const o of engine.takeOutputs()) {
        // A script names its output relative to --out; one that would land
        // outside it (an absolute path or `..`) is refused, never written.
        const path = resolve(out, o.file);
        const inside = relative(resolve(out), path);
        if (inside === "" || inside === ".." || inside.startsWith(".." + sep) || isAbsolute(inside)) {
          const line = `WARN tick ${o.tick} ${o.action}: output path '${o.file}' leaves --out ${out}; not written`;
          warnings.push(line);
          process.stderr.write(line + "\n");
          continue;
        }
        mkdirSync(dirname(path), { recursive: true });
        writeFileSync(path, o.kind === "png" || o.kind === "wav" ? decode(o.data) : o.data);
        outputs.push(path);
        // `dump_ui:<name>.uicheck.json`: og ui check's reading of the same frame
        // beside it, `<name>.uiframe.json` (checkUi in a playtest reads it).
        if (o.action === "dump_ui" && path.endsWith(".uicheck.json")) writeFileSync(path.replace(/\.uicheck\.json$/, ".uiframe.json"), extras.og_node_ui_frame(engine.id));
        if (!asJson) process.stdout.write(`tick ${o.tick} ${o.action}: wrote ${path}\n`);
      }
      for (const l of engine.takeLog()) {
        const line = `${l.warning ? "WARN" : "INFO"} tick ${l.tick}: ${l.text}`;
        if (l.warning) warnings.push(line);
        else {
          const frame = UNCHECKED.exec(l.text);
          if (frame) unchecked.push(frame[1]!);
        }
        process.stderr.write(line + "\n");
      }
    }

    let tier = "none";
    let adapterError: string | null = null;
    if (deferStart) {
      const attachStart = unixMs();
      const attached = attachRenderer(engine.id, size.width, size.height);
      hostPhase("renderer-attach", attachStart);
      if (!attached.ok) fail(attached.reason);
      tier = attached.value;
      adapterError = attached.adapterError;
      // The first tick boundary: GameDesc.start, with the surface in place.
      const startTime = unixMs();
      const started = engine.advance(0);
      hostPhase("game-start", startTime);
      if (!started.ok) fail(started.reason);
    }
    drain();

    // Before --setup, so a setup step may ask the relay for a room itself (og.platform.quickPlay).
    if (room !== undefined) {
      const linked = JSON.parse(extras.og_node_room_connect(engine.id, room)) as { ok: boolean; reason?: string };
      if (!linked.ok) fail(`--room ${room}: ${linked.reason}`);
      cleanup.push(() => {
        extras.og_node_room_close(engine.id);
      });
      // The relay joins the room the URL names: wait for its answer before the first tick. A URL
      // without a room (ws://127.0.0.1:P/) only connects: the game joins, creates or quick-plays.
      const named = !/^ws:\/\/[^/]+\/?$/.test(room);
      for (let i = 0; named && i < 2000 && (JSON.parse(abiRoomInfo()) as { state: string }).state !== "in"; i++) {
        extras.og_node_room_pump(engine.id);
        await new Promise((r) => setTimeout(r, 1));
      }
    }

    for (const step of setup) {
      const [action, ...args] = step.split(":");
      const r = engine.command(action!, args);
      if (!r.ok) fail(`--setup ${step}: ${r.reason}`);
    }

    /** The world's tick, from its dump. */
    const worldTick = (): number => (engine.probe.dump({ what: "world" }) as unknown as { tick: number }).tick;

    if (restoreFile) {
      const restored = engine.world.restore(readFileSync(restoreFile, "utf8"), { migrate: true });
      if (restored.ok) process.stdout.write(JSON.stringify({ event: "restored", ok: true, tick: worldTick() }) + "\n");
      else {
        process.stderr.write(`WARN --restore ${restoreFile}: ${restored.reason}; the run starts fresh\n`);
        process.stdout.write(JSON.stringify({ event: "restored", ok: false, tick: worldTick(), reason: restored.reason }) + "\n");
      }
      drain();
    }

    let handoffAsked = false;
    if (handoffFile) {
      const ask = () => {
        handoffAsked = true;
      };
      hostProcess.on("SIGUSR2", ask);
      cleanup.push(() => {
        hostProcess.off("SIGUSR2", ask);
      });
    }

    let listening: number | undefined;
    if (port !== undefined) {
      const r = JSON.parse(extras.og_node_probe_listen(engine.id, port)) as { ok: boolean; value?: number; reason?: string };
      if (!r.ok) fail(r.reason!);
      listening = r.value!;
      cleanup.push(() => {
        extras.og_node_probe_close(engine.id);
      });
      process.stdout.write(`probe: ws://127.0.0.1:${listening}/probe\n`);
    }

    function abiRoomInfo(): string {
      return boundHost()!.abi.og_platform_room_info(engine.id);
    }

    type NetStatus = { running: boolean; startTick: number | null; waits: number; desync: number | null; rollback: boolean; acknowledged: number | null };
    const netStatus = (): NetStatus => JSON.parse(boundHost()!.abi.og_net_status(engine.id)) as NetStatus;
    const finalHash = (t: number): string | null => JSON.parse(boundHost()!.abi.og_net_final_hash(engine.id, t)) as string | null;
    const rollbackDone = (t: number): boolean => {
      const s = netStatus();
      return !s.rollback || (finalHash(t) !== null && (s.acknowledged ?? t) >= t - 1);
    };

    let untilTick = -1;
    if (commands !== undefined) {
      const report = engine.probe.commands(commands) as { scheduled: number; untilTick: number; warnings: string[] } | null;
      drain();
      if (!report) fail(`--commands: the script is malformed (the reason is above)`);
      for (const w of report!.warnings) process.stderr.write(`WARN ${w}\n`);
      untilTick = report!.untilTick;
    }

    const rate = engine.rate;
    const tickMs = 1000 / rate;
    // --json's frame times: each frame's tick and render milliseconds (the
    // first five left out: the first frames compile pipelines), and each system's time over the
    // run from the world's dump (World::set_clock).
    type SystemTimes = { name: string; totalMs?: number; runs?: number }[];
    const systemTimes = (): SystemTimes => (engine.probe.dump({ what: "world" }) as { systems?: SystemTimes }).systems ?? [];
    const systemsBefore = asJson ? systemTimes() : [];
    const stepTimes: number[] = [];
    const renderTimes: number[] = [];
    let framesSeen = 0;
    const t0 = performance.now();
    let frameFailed: string | null = null;
    let quit = false;

    if (!probing) {
      const advanced = engine.advance(ticks ?? 0);
      if (!advanced.ok) fail(advanced.reason);
      sink.pump();
    } else {
      let pendingReload: { mtimeMs: number; importMs: number } | null = null;
      // Imports in flight: a reload swaps only once the last one has declared
      // everything (a second write of the bundle starts another import, whose
      // beginBundle clears the declarations the first one made).
      let importing = 0;
      if (watchFile) {
        let busy = false;
        const watcher = watch(watchFile, () => {
          if (busy) return;
          busy = true;
          setTimeout(async () => {
            busy = false;
            const mtimeMs = statSync(watchFile!).mtimeMs;
            importing++;
            try {
              const started = performance.now();
              beginBundle();
              await importGame(watchFile!);
              pendingReload = { mtimeMs, importMs: performance.now() - started };
            } catch (e) {
              process.stderr.write(`WARN reload: the bundle failed to import (${e instanceof Error ? e.message : String(e)}); the old one keeps running\n`);
            } finally {
              importing--;
            }
          }, 5);
        });
        cleanup.push(() => watcher.close());
      }
      const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
      let last = performance.now();
      for (;;) {
        if (handoffAsked) {
          const save = engine.world.snapshot();
          if (!save.ok) fail(`--handoff: the engine refused a snapshot (${save.reason})`);
          // Written whole or not at all: the next process reads it once this one is gone.
          writeFileSync(`${handoffFile}.part`, save.value);
          renameSync(`${handoffFile}.part`, handoffFile!);
          const tick = worldTick();
          process.stdout.write(JSON.stringify({ event: "handoff", tick }) + "\n");
          return 0;
        }
        if (pendingReload && importing === 0) {
          const { mtimeMs, importMs } = pendingReload as { mtimeMs: number; importMs: number };
          pendingReload = null;
          const restoring = performance.now();
          const report = engine.probe.reload() as { ms: number; tick: number } | null;
          const restored = performance.now();
          const first = engine.frame(0);
          const submitted = performance.now();
          let rendered = false;
          if (first.rendered) {
            // Measure the completed first frame, without rendering and encoding
            // an unused second frame through probe.capture.
            const waited: Result<void> = JSON.parse(extras.og_node_wait_gpu(engine.id));
            if (!waited.ok) fail(`reload's first frame: ${waited.reason}`);
            rendered = true;
          }
          const timings = {
            importMs,
            reloadMs: restored - restoring,
            frameMs: submitted - restored,
            gpuWaitMs: performance.now() - submitted,
          };
          const latencyMs = Date.now() - mtimeMs;
          process.stdout.write(
            JSON.stringify({ event: "reload", ok: report !== null && first.ok, report, tick: first.tick, rendered, timings, latencyMs: +latencyMs.toFixed(1) }) + "\n",
          );
          drain();
        }
        const elapsed = realtime ? performance.now() - last : tickMs;
        last = performance.now();
        if (room !== undefined) extras.og_node_room_pump(engine.id);
        const answer = engine.frame(elapsed);
        if (room !== undefined) extras.og_node_room_pump(engine.id);
        if (!answer.ok) {
          frameFailed = answer.reason ?? "the frame failed";
          break;
        }
        if (asJson && ++framesSeen > 5) {
          if ((answer.ticks ?? 0) > 0) stepTimes.push(answer.stepMs ?? 0);
          if (answer.rendered) renderTimes.push(answer.renderMs ?? 0);
        }
        sink.pump();
        if (listening !== undefined) extras.og_node_probe_serve(engine.id);
        takeTrace(engine.id);
        drain();
        const tick = answer.tick ?? 0;
        if (answer.quit) {
          quit = true;
          break;
        }
        // In rollback the world reaches --ticks on predicted inputs: the run
        // goes on (predicting at most maxPrediction ahead) until that tick's
        // world is final and every peer acknowledged this one's inputs for it
        // (they may have been lost), so `confirmedHash` is the room's (finalHash).
        if (ticks !== undefined && tick >= ticks && (room === undefined || rollbackDone(ticks) || tick > ticks + 600)) break;
        // The script ends once its last step has run: a restore moving og.Time
        // forward or back moves the steps with it (they keep their delays), so
        // the tick its last step was scheduled for is no end (brasswick #13).
        const left = answer.pending ?? (tick > untilTick ? 0 : 1);
        if (ticks === undefined && commands !== undefined && listening === undefined && !watchFile && left === 0) break;
        if (realtime || listening !== undefined || watchFile) await sleep(Math.max(0, tickMs - (performance.now() - last)));
        // In a room a frame that took no tick waits for the other players' inputs,
        // and outside lockstep (its lobby, or a game that leaves it off) the run
        // keeps the tick's pace with the other peers: an unpaced lobby would
        // spend the run's --ticks before they arrive.
        else if (room !== undefined && (answer.ticks ?? 0) === 0) await sleep(1);
        else if (room !== undefined && !netStatus().running) await sleep(Math.max(0, tickMs - (performance.now() - last)));
      }
    }
    const ms = performance.now() - t0;
    hostPhase("run-loop", performance.timeOrigin + t0);
    drain();
    if (tracePath) {
      takeTrace(engine.id, true);
      mkdirSync(dirname(tracePath), { recursive: true });
      writeFileSync(tracePath, JSON.stringify(trace!.document()));
      outputs.push(tracePath);
    }
    let wavSha256: string | undefined;
    if (audioMode === "capture") {
      const wavPath = resolve(audioOut);
      mkdirSync(dirname(wavPath), { recursive: true });
      const wav = sink.wav();
      writeFileSync(wavPath, wav);
      wavSha256 = createHash("sha256").update(wav).digest("hex");
      outputs.push(wavPath);
    }
    const audioStats = sink.stats();
    if (frameFailed) process.stderr.write(`og run: ${frameFailed}\n`);
    const dump = engine.probe.dump({ what: "world" }) as unknown as { hash: string; entities: number; tick: number; systems?: SystemTimes };
    // The script's expectations (`expect:` steps): a failure fails the run.
    const expectations = engine.probe.expectations();
    // The run's assertions (og.assert, the default invariants, `invariant:` steps): a violation fails the run.
    const assertions = engine.probe.assertions() as unknown as { violations: unknown[]; uncovered: unknown[] };
    // Order statistics (nearest rank).
    const percentile = (xs: number[], q: number): number => {
      if (xs.length === 0) return 0;
      const s = [...xs].sort((a, b) => a - b);
      return s[Math.min(s.length - 1, Math.floor(q * s.length))]!;
    };
    const r3 = (x: number) => +x.toFixed(3);
    const frames = {
      stepMedianMs: r3(percentile(stepTimes, 0.5)),
      stepP95Ms: r3(percentile(stepTimes, 0.95)),
      stepMaxMs: r3(Math.max(0, ...stepTimes)),
      renderMedianMs: r3(percentile(renderTimes, 0.5)),
      renderP95Ms: r3(percentile(renderTimes, 0.95)),
      renderMaxMs: r3(Math.max(0, ...renderTimes)),
      stepped: stepTimes.length,
      rendered: renderTimes.length,
    };
    // Each system's mean milliseconds a run over this run, slowest first.
    const systems = (dump.systems ?? [])
      .map((s) => {
        const before = systemsBefore.find((b) => b.name === s.name);
        const runs = (s.runs ?? 0) - (before?.runs ?? 0);
        const ms = (s.totalMs ?? 0) - (before?.totalMs ?? 0);
        return { name: s.name, meanMs: runs > 0 ? r3(ms / runs) : 0, totalMs: r3(ms), runs };
      })
      .filter((s) => s.runs > 0)
      .sort((a, b) => b.totalMs - a.totalMs);
    const summary = {
      entry,
      ticks: ticks ?? dump.tick,
      tick: dump.tick,
      hash: dump.hash,
      // og.net's shared hash: the world without this peer's session, what lockstep peers compare.
      netHash: JSON.parse(boundHost()!.abi.og_net_hash(engine.id)) as string,
      // With --room: lockstep's state at the end (the tick it started, the waits, the first divergent tick),
      // and the shared hash of the final world at --ticks (in rollback the world may run past it).
      ...(room !== undefined ? { room: JSON.parse(abiRoomInfo()) as unknown, lockstep: netStatus(), confirmedHash: finalHash(ticks ?? dump.tick) } : {}),
      entities: dump.entities,
      // The game's facts (og.game's `facts`), as the probe prints them.
      facts: (engine.probe.state() as { facts?: unknown }).facts ?? null,
      advanceMs: +ms.toFixed(2),
      frames,
      systems,
      runtime: typeof Bun !== "undefined" ? `bun ${Bun.version}` : `node ${process.version}`,
      tier,
      renderer: tier,
      adapterError,
      // Whether the device timed each render pass on the GPU (RenderCapabilities.gpuTimestamps): the trace's gpu thread and ProbeTimings.gpuPasses.
      gpuTimestamps: (JSON.parse(boundHost()!.abi.og_render_capabilities(engine.id)) as { gpuTimestamps?: boolean }).gpuTimestamps ?? false,
      quit,
      unchecked,
      uncheckedCount: unchecked.length + expectations.unchecked,
      audio: { mode: audioMode, commands: audioStats.commands, receivedHash: audioStats.receivedHash, wavSha256 },
      outputs,
      warnings,
      expectations,
      assertions,
    };
    if (asJson) process.stdout.write(JSON.stringify(summary) + "\n");
    else {
      process.stdout.write(
        `tick ${dump.tick} hash ${dump.hash} entities ${dump.entities}; ${summary.uncheckedCount} GPU frame/expectation checks unchecked${adapterError ? `: ${adapterError}` : ""}\n`,
      );
      if (expectations.results.length > 0)
        process.stdout.write(`expectations: ${expectations.passed} passed, ${expectations.failed} failed, ${expectations.unchecked} unchecked\n`);
      for (const v of assertions.violations as Array<{ name: string; tick: number; entity?: string | null; value: unknown; message: string }>)
        process.stdout.write(`assertion failed at tick ${v.tick}: ${v.name}${v.entity ? ` on ${v.entity}` : ""}: ${v.message} (${JSON.stringify(v.value)})\n`);
      for (const u of assertions.uncovered as Array<{ kind: string; name: string; key?: string }>)
        process.stdout.write(`never fired: ${u.kind} ${u.name}${u.key !== undefined ? ` [${u.key}]` : ""}\n`);
    }
    return frameFailed || expectations.failed > 0 || assertions.violations.length > 0 ? 1 : 0;
  } catch (error) {
    if (error instanceof RunExit) return error.code;
    throw error;
  } finally {
    const errors: unknown[] = [];
    for (const dispose of cleanup.reverse()) {
      try {
        dispose();
      } catch (error) {
        errors.push(error);
      }
    }
    if (errors.length > 0) throw new AggregateError(errors, "run cleanup failed");
  }
}

if (import.meta.main) hostProcess.exit(await run(hostProcess.argv.slice(2)));
