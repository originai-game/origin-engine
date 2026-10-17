// `og run` on the native host: load a game bundle under bun (or node) with
// the napi addon, open the project's cooked bundle, mount the probe, and
// drive the tick.
//
//   bun src/node/run.ts --entry <game.js|.ts> [--root <project>] [--ticks N] [--seed S]
//       [--setup action[:arg...]]... [--commands "t:action[:args];..."] [--out <dir>]
//       [--port P] [--size WxH] [--realtime] [--watch <bundle.js>] [--no-render]
//       [--audio null|device|<out.wav>] [--json]
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

import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, watch, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { loadNativeHost, native } from "./index.ts";
import { openAudio, type AudioSink } from "./audio.ts";
import { beginBundle, createEngine, type Engine, type GameToken } from "../runtime.ts";

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
if (port !== undefined && (!Number.isInteger(port) || port < 0 || port > 65535)) fail("--port takes 0 (a free port) to 65535");

loadNativeHost();
const extras = native();
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

/** project.ogt's `profile` ("2d" or "3d"), which chooses the physics profile. */
function projectProfile(dir: string | undefined): "2d" | "3d" | undefined {
  if (!dir || !existsSync(join(dir, "project.ogt"))) return undefined;
  const m = /^\s*profile\s+"(2d|3d)"/m.exec(readFileSync(join(dir, "project.ogt"), "utf8"));
  return m ? (m[1] as "2d" | "3d") : undefined;
}

const game = await importGame(entry);
const profile = projectProfile(root);
const created = createEngine(game, { ...(seed === undefined ? {} : { seed }), ...(root ? { root } : {}), ...(profile ? { profile } : {}) });
if (!created.ok) fail(`${entry}: ${created.reason}`);
const engine: Engine = created.value;
// One sink per engine, pumped after every step (crates/og-audio/README.md
// §Host contract): null keeps the stream drained, a .wav path captures it.
const audioMode = audioOut === "device" ? "device" : audioOut === "null" ? "null" : "capture";
const opened = openAudio(engine.id, audioMode);
if (!opened.ok) fail(`--audio ${audioOut}: ${opened.reason}`);
const sink: AudioSink = opened.value;
const outputs: string[] = [];
const warnings: string[] = [];

function decode(base64: string): Uint8Array {
  const bin = atob(base64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes;
}

function drain(): void {
  for (const o of engine.takeOutputs()) {
    const path = resolve(out, o.file);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, o.kind === "png" ? decode(o.data) : o.data);
    outputs.push(path);
    if (!asJson) process.stdout.write(`tick ${o.tick} ${o.action}: wrote ${path}\n`);
  }
  for (const l of engine.takeLog()) {
    const line = `${l.warning ? "WARN" : "INFO"} tick ${l.tick}: ${l.text}`;
    if (l.warning) warnings.push(line);
    process.stderr.write(line + "\n");
  }
}

const probing = commands !== undefined || port !== undefined || watchFile !== undefined;
let tier = "none";
if (probing && render) {
  const attached = JSON.parse(extras.og_node_attach_offscreen(engine.id, size.width, size.height)) as { ok: boolean; value?: string; reason?: string };
  if (attached.ok) tier = attached.value!;
  else process.stderr.write(`WARN no renderer (${attached.reason}); screenshot and capture refuse\n`);
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

let untilTick = -1;
if (commands !== undefined) {
  const report = engine.probe.commands(commands) as { scheduled: number; untilTick: number; warnings: string[] } | null;
  drain();
  if (!report) fail(`--commands: the script is malformed (the reason is above)`);
  for (const w of report!.warnings) process.stderr.write(`WARN ${w}\n`);
  untilTick = report!.untilTick;
}

const rate = engine.game.tick;
const tickMs = 1000 / rate;
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
    const answer = engine.frame(elapsed);
    if (!answer.ok) {
      frameFailed = answer.reason ?? "the frame failed";
      break;
    }
    sink.pump();
    if (listening !== undefined) extras.og_node_probe_serve(engine.id);
    drain();
    const tick = answer.tick ?? 0;
    if (answer.quit) {
      quit = true;
      break;
    }
    if (ticks !== undefined && tick >= ticks) break;
    if (ticks === undefined && commands !== undefined && listening === undefined && !watchFile && tick > untilTick) break;
    if (realtime || listening !== undefined || watchFile) await sleep(Math.max(0, tickMs - (performance.now() - last)));
  }
}
const ms = performance.now() - t0;
drain();
if (audioMode === "capture") {
  const wavPath = resolve(audioOut);
  mkdirSync(dirname(wavPath), { recursive: true });
  writeFileSync(wavPath, sink.wav());
  outputs.push(wavPath);
}
const audioStats = sink.stats();
sink.close();
if (frameFailed) process.stderr.write(`og run: ${frameFailed}\n`);
const dump = engine.probe.dump({ what: "world" }) as { hash: string; entities: number; tick: number };
const summary = {
  entry,
  ticks: ticks ?? dump.tick,
  tick: dump.tick,
  hash: dump.hash,
  entities: dump.entities,
  advanceMs: +ms.toFixed(2),
  runtime: typeof Bun !== "undefined" ? `bun ${Bun.version}` : `node ${process.version}`,
  tier,
  quit,
  audio: { mode: audioMode, commands: audioStats.commands, receivedHash: audioStats.receivedHash },
  outputs,
  warnings,
};
if (asJson) process.stdout.write(JSON.stringify(summary) + "\n");
else process.stdout.write(`tick ${dump.tick} hash ${dump.hash} entities ${dump.entities}\n`);
if (listening !== undefined) extras.og_node_probe_close(engine.id);
process.exit(frameFailed ? 1 : 0);
