// `og run` on the native host: load a game bundle under bun (or node) with
// the napi addon, open the project's cooked bundle, mount the probe, and
// drive the tick.
//
//   bun src/node/run.ts --entry <game.js|.ts> [--root <project>] [--ticks N] [--seed S]
//       [--setup action[:arg...]]... [--commands "t:action[:args];..."] [--out <dir>]
//       [--port P] [--size WxH] [--realtime] [--watch <bundle.js>] [--no-render]
//       [--audio null|device|<out.wav>] [--platform-store <file.json>] [--clock <UTC ms>] [--locale <tag>]
//       [--trace <out.json> [--trace-append]] [--room ws://127.0.0.1:P/<room>] [--json]
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
// - Without a renderer (no usable adapter, as in the platform's engine and
//   probe sandboxes, or OG_RENDERER=none) the run is the same run: the play
//   surface keeps --size, a `screenshot` writes no PNG and is listed in
//   --json's `unchecked` ("no renderer, frame unchecked"), the tier is
//   `none`, and the exit status does not change.
// - With --trace: the run in the Chrome trace format (crates/og-cli/src/trace.rs
//   has the lanes): each tick with its systems, each frame with its render
//   passes, each load; --trace-append adds them to the file og wrote first
//   (its metadata, its origin and the cook's spans).

import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, watch, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { loadNativeHost, native } from "./index.ts";
import { openAudio, type AudioSink } from "./audio.ts";
import { nativePlatform } from "./platform.ts";
import { beginBundle, boundHost, createEngine, fixedClock, type Engine, type GameToken } from "../runtime.ts";

const argv = process.argv.slice(2);
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
let locale: string | undefined;
let clock: number | undefined;
let tracePath: string | undefined;
let traceAppend = false;
let room: string | undefined;
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
  else if (a === "--locale") locale = next();
  else if (a === "--clock") clock = Number(next());
  else if (a === "--trace") tracePath = resolve(next());
  else if (a === "--trace-append") traceAppend = true;
  else if (a === "--room") room = next();
  else if (a === "--headless") {
    // Native runs are always headless; accepted for `og run --headless`.
  } else fail(`unknown argument '${a}'`);
}

function fail(reason: string): never {
  process.stderr.write(`og run: ${reason}\n`);
  process.exit(2);
}

if (!entry) fail("no game entry: pass --entry <file>");
if (ticks !== undefined && (!Number.isInteger(ticks) || ticks < 0)) fail("--ticks takes a whole number of fixed steps");
if (seed !== undefined && (!Number.isInteger(seed) || seed < 0)) fail("--seed takes a whole number");
if (clock !== undefined && !(Number.isFinite(clock) && clock > 0)) fail("--clock takes UTC milliseconds since 1970, such as 1767225600000");
if (port !== undefined && (!Number.isInteger(port) || port < 0 || port > 65535)) fail("--port takes 0 (a free port) to 65535");

loadNativeHost();
const extras = native();

// --trace: the Chrome trace's events, on Unix milliseconds less the origin.
type TraceSpan = { kind: "tick" | "render" | "load"; name: string; tick?: number; start: number; ms: number; bytes?: number; children?: { name: string; ms: number }[] };
const traceEvents: Record<string, unknown>[] = [];
let traceOrigin = performance.timeOrigin + performance.now();
const LANES: Record<TraceSpan["kind"], number> = { tick: 1, render: 2, load: 3 };
if (tracePath && traceAppend && existsSync(tracePath)) {
  const prior = JSON.parse(readFileSync(tracePath, "utf8")) as { traceEvents?: Record<string, unknown>[]; otherData?: { originUnixMs?: number } };
  traceEvents.push(...(prior.traceEvents ?? []));
  if (typeof prior.otherData?.originUnixMs === "number") traceOrigin = prior.otherData.originUnixMs;
} else if (tracePath) {
  traceEvents.push({ name: "process_name", ph: "M", pid: 1, tid: 0, args: { name: "og run" } });
  for (const [tid, name] of [[1, "engine"], [2, "render"], [3, "load"], [4, "cook"]] as const) {
    traceEvents.push({ name: "thread_name", ph: "M", pid: 1, tid, args: { name } });
    traceEvents.push({ name: "thread_sort_index", ph: "M", pid: 1, tid, args: { sort_index: tid } });
  }
}
const us = (ms: number) => Math.round(ms * 1000);
/** Takes the engine's spans and places them: a span's children (systems, passes) laid end to end from its start, in run order. */
function takeTrace(engineId: number): void {
  if (!tracePath) return;
  const taken = JSON.parse(extras.og_node_trace_take(engineId)) as { now: number; events: TraceSpan[] };
  const offset = performance.timeOrigin + performance.now() - taken.now - traceOrigin;
  for (const s of taken.events) {
    const tid = LANES[s.kind];
    const ts = s.start + offset;
    const args: Record<string, unknown> = {};
    if (s.tick !== undefined) args.tick = s.tick;
    if (s.bytes !== undefined) args.bytes = s.bytes;
    traceEvents.push({ name: s.name, cat: s.kind, ph: "X", pid: 1, tid, ts: us(ts), dur: us(s.ms), args });
    // Whole microseconds end to end, so rounding never overlaps neighbours.
    let at = us(ts);
    for (const c of s.children ?? []) {
      const dur = us(c.ms);
      traceEvents.push({ name: c.name, cat: s.kind === "tick" ? "system" : "pass", ph: "X", pid: 1, tid, ts: at, dur, args: s.tick === undefined ? {} : { tick: s.tick } });
      at += dur;
    }
  }
}
let version = 0;
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

const game = await importGame(entry);
// Loads made while the engine is created (its boot scene's) are traced too.
const BEFORE_ENGINE = 0xffffffff;
if (tracePath) extras.og_node_trace(BEFORE_ENGINE, true);
const project = projectText(root);
const profile = projectProfile(project);
const probing = commands !== undefined || port !== undefined || watchFile !== undefined || room !== undefined;
// With a renderer, GameDesc.start runs once it is attached (as the web Worker
// does), so og.render.capabilities reports the play surface's size there.
const deferStart = probing && render;
const created = createEngine(game, { ...(seed === undefined ? {} : { seed }), ...(root ? { root } : {}), ...(project === undefined ? {} : { project }), ...(profile ? { profile } : {}), ...(locale ? { locale } : {}), ...(deferStart ? { deferStart } : {}) });
if (!created.ok) fail(`${entry}: ${created.reason}`);
const engine: Engine = created.value;
if (tracePath) {
  extras.og_node_trace(engine.id, true);
  extras.og_node_trace(BEFORE_ENGINE, false);
}
// og.platform: the local store, in memory unless --platform-store names a file;
// its clock the system's, or with --clock one that starts there and moves with the ticks.
if (clock !== undefined) engine.setClock(fixedClock(engine, clock));
engine.attachPlatform(nativePlatform(platformStore));
// One sink per engine, pumped after every step (crates/og-audio/README.md
// §Host contract): null keeps the stream drained, a .wav path captures it.
const audioMode = audioOut === "device" ? "device" : audioOut === "null" ? "null" : "capture";
const opened = openAudio(engine.id, audioMode);
if (!opened.ok) fail(`--audio ${audioOut}: ${opened.reason}`);
const sink: AudioSink = opened.value;
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
if (deferStart) {
  const attached = JSON.parse(extras.og_node_attach_offscreen(engine.id, size.width, size.height)) as { ok: boolean; value?: string; reason?: string };
  if (attached.ok) tier = attached.value!;
  else process.stderr.write(`INFO no renderer (${attached.reason}): the run is the same, a screenshot writes no frame and reports it unchecked, capture answers null\n`);
  // The first tick boundary: GameDesc.start, with the surface in place.
  const started = engine.advance(0);
  if (!started.ok) fail(started.reason);
}
drain();

for (const step of setup) {
  const [action, ...args] = step.split(":");
  const r = engine.command(action!, args);
  if (!r.ok) fail(`--setup ${step}: ${r.reason}`);
}

let listening: number | undefined;
if (port !== undefined) {
  const r = JSON.parse(extras.og_node_probe_listen(engine.id, port)) as { ok: boolean; value?: number; reason?: string };
  if (!r.ok) fail(r.reason!);
  listening = r.value!;
  process.stdout.write(`probe: ws://127.0.0.1:${listening}/probe\n`);
}

if (room !== undefined) {
  const linked = JSON.parse(extras.og_node_room_connect(engine.id, room)) as { ok: boolean; reason?: string };
  if (!linked.ok) fail(`--room ${room}: ${linked.reason}`);
  // The relay joins the room the URL names: wait for its answer before the first tick.
  for (let i = 0; i < 2000 && (JSON.parse(abiRoomInfo()) as { state: string }).state !== "in"; i++) {
    extras.og_node_room_pump(engine.id);
    await new Promise((r) => setTimeout(r, 1));
  }
}

function abiRoomInfo(): string {
  return boundHost()!.abi.og_platform_room_info(engine.id);
}

type NetStatus = { running: boolean; startTick: number | null; waits: number; desync: number | null };
const netStatus = (): NetStatus => JSON.parse(boundHost()!.abi.og_net_status(engine.id)) as NetStatus;

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
// first five left out, as og polish does), and each system's time over the
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
  let pendingReload: { mtimeMs: number } | null = null;
  // Imports in flight: a reload swaps only once the last one has declared
  // everything (a second write of the bundle starts another import, whose
  // beginBundle clears the declarations the first one made).
  let importing = 0;
  if (watchFile) {
    let busy = false;
    watch(watchFile, () => {
      if (busy) return;
      busy = true;
      setTimeout(async () => {
        busy = false;
        const mtimeMs = statSync(watchFile!).mtimeMs;
        importing++;
        try {
          beginBundle();
          await importGame(watchFile!);
          pendingReload = { mtimeMs };
        } catch (e) {
          process.stderr.write(`WARN reload: the bundle failed to import (${e instanceof Error ? e.message : String(e)}); the old one keeps running\n`);
        } finally {
          importing--;
        }
      }, 5);
    });
  }
  const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
  let last = performance.now();
  for (;;) {
    if (pendingReload && importing === 0) {
      const { mtimeMs } = pendingReload as { mtimeMs: number };
      pendingReload = null;
      const report = engine.probe.reload() as { ms: number; tick: number } | null;
      const first = engine.frame(0);
      const captured = render && engine.probe.capture(0, 0) !== null;
      const latencyMs = Date.now() - mtimeMs;
      process.stdout.write(JSON.stringify({ event: "reload", ok: report !== null, report, tick: first.tick, captured, latencyMs: +latencyMs.toFixed(1) }) + "\n");
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
    if (ticks !== undefined && tick >= ticks) break;
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
drain();
if (tracePath) {
  takeTrace(engine.id);
  mkdirSync(dirname(tracePath), { recursive: true });
  writeFileSync(tracePath, JSON.stringify({ traceEvents, displayTimeUnit: "ms", otherData: { originUnixMs: traceOrigin, producer: "og run --trace" } }));
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
sink.close();
if (frameFailed) process.stderr.write(`og run: ${frameFailed}\n`);
const dump = engine.probe.dump({ what: "world" }) as unknown as { hash: string; entities: number; tick: number; systems?: SystemTimes };
// The script's expectations (`expect:` steps): a failure fails the run.
const expectations = engine.probe.expectations();
// og polish's order statistics (src/node/polish.ts), so the two agree.
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
  // With --room: lockstep's state at the end (the tick it started, the waits, the first divergent tick).
  ...(room !== undefined ? { lockstep: netStatus() } : {}),
  entities: dump.entities,
  advanceMs: +ms.toFixed(2),
  frames,
  systems,
  runtime: typeof Bun !== "undefined" ? `bun ${Bun.version}` : `node ${process.version}`,
  tier,
  quit,
  unchecked,
  audio: { mode: audioMode, commands: audioStats.commands, receivedHash: audioStats.receivedHash, wavSha256 },
  outputs,
  warnings,
  expectations,
};
if (asJson) process.stdout.write(JSON.stringify(summary) + "\n");
else {
  process.stdout.write(`tick ${dump.tick} hash ${dump.hash} entities ${dump.entities}\n`);
  if (expectations.results.length > 0) process.stdout.write(`expectations: ${expectations.passed} passed, ${expectations.failed} failed, ${expectations.unchecked} unchecked\n`);
}
if (listening !== undefined) extras.og_node_probe_close(engine.id);
if (room !== undefined) extras.og_node_room_close(engine.id);
process.exit(frameFailed || expectations.failed > 0 ? 1 : 0);
