// `og render` on the native host (docs/design.md §2.9): the game run
// headless at its own tick, one frame read back every `rate / fps` ticks,
// streamed to `og` (crates/og-cli/src/render.rs), which writes the frames,
// encodes the video and draws the contact sheet. One process of two kinds:
//
//   bun src/node/render.ts --entry <game.js> [--root <project>] [--seed S] [--size WxH]
//       [--commands "t:action[:args];..."] --fps F [--from <seconds>] [--warmup W]
//       [--clock <UTC ms>] [--locale <tag>] [--out <dir>]
//       a range:    --first I --count N [--restore <snapshot>] [--audio <out.wav>]
//       the leader: --leader --end N [--snapshot <frame>:<file>]... [--audio <out.wav>]
//
// Frame i of the render shows the world at tick T0 + i·k, where T0 is
// round(from · rate) and k = rate / fps (the fps must divide the tick rate).
// - A range sends frames I to I + N - 1, after W warm-up frames on the same
//   grid before it (drawn, not sent: the renderer's history, TAA and the
//   adapted exposure, is then that of a run that drew them). Ticks between
//   frames are simulated without drawing. With --restore it starts from the
//   leader's render snapshot, taken at the first warm-up frame's tick,
//   instead of simulating from tick 0. With --audio the capture mixer's
//   WAV from tick 0 to the range's end is written there.
// - The leader (`og render --jobs N`) simulates to frame N's tick without
//   drawing, writing a render snapshot at the first warm-up tick of each
//   --snapshot frame, and the WAV.
//
// Stdout carries records only: a little-endian u32 length, that many bytes
// of JSON, then the JSON's `bytes` of payload. A frame is
// `{kind:"frame", index, tick, time, width, height, sha256, bytes}` with its
// RGBA8 rows; the last record is `{kind:"end", ...}`. Everything else
// (the engine's log, warnings) goes to stderr. Outputs of the script's own
// steps (screenshot, dump_*) are written under --out.

import { mkdirSync, readFileSync, writeFileSync, writeSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";
import { loadNativeHost, native } from "./index.ts";
import { openAudio } from "./audio.ts";
import { nativePlatform } from "./platform.ts";
import { createEngine, fixedClock, type Engine, type GameToken } from "../runtime.ts";

declare const Bun: { sleepSync(ms: number): void };

const argv = process.argv.slice(2);
let entry = "";
let root: string | undefined;
let seed: number | undefined;
let commands: string | undefined;
let size = { width: 1280, height: 720 };
let fps = 0;
let fromSeconds = 0;
let first: number | undefined;
let count: number | undefined;
let warmup = 0;
let restore: string | undefined;
let audioOut: string | undefined;
let leader = false;
let end: number | undefined;
let clock: number | undefined;
let locale: string | undefined;
let out: string | undefined;
const snapshots: { frame: number; file: string }[] = [];
for (let i = 0; i < argv.length; i++) {
  const a = argv[i]!;
  const next = () => {
    const v = argv[++i];
    if (v === undefined) fail(`${a} needs a value`);
    return v!;
  };
  const whole = (v: string) => {
    const n = Number(v);
    if (!Number.isInteger(n) || n < 0) fail(`${a} takes a whole number, not '${v}'`);
    return n;
  };
  if (a === "--entry") entry = next();
  else if (a === "--root") root = resolve(next());
  else if (a === "--seed") seed = whole(next());
  else if (a === "--commands") commands = next();
  else if (a === "--size") {
    const m = /^(\d+)x(\d+)$/.exec(next());
    if (!m) fail("--size takes WIDTHxHEIGHT, such as 1280x720");
    size = { width: Number(m![1]), height: Number(m![2]) };
  } else if (a === "--fps") fps = whole(next());
  else if (a === "--from") {
    fromSeconds = Number(next());
    if (!(Number.isFinite(fromSeconds) && fromSeconds >= 0)) fail("--from takes seconds, 0 or more");
  } else if (a === "--first") first = whole(next());
  else if (a === "--count") count = whole(next());
  else if (a === "--warmup") warmup = whole(next());
  else if (a === "--restore") restore = resolve(next());
  else if (a === "--audio") audioOut = resolve(next());
  else if (a === "--leader") leader = true;
  else if (a === "--end") end = whole(next());
  else if (a === "--snapshot") {
    const v = next();
    const at = v.indexOf(":");
    if (at <= 0) fail("--snapshot takes <frame>:<file>");
    snapshots.push({ frame: whole(v.slice(0, at)), file: resolve(v.slice(at + 1)) });
  } else if (a === "--clock") clock = Number(next());
  else if (a === "--locale") locale = next();
  else if (a === "--out") out = resolve(next());
  else fail(`unknown argument '${a}'`);
}

function fail(reason: string): never {
  process.stderr.write(`og render: ${reason}\n`);
  process.exit(2);
}

if (!entry) fail("no game entry: pass --entry <file>");
if (fps <= 0) fail("--fps takes frames per second, such as 30");
if (leader ? end === undefined : first === undefined || count === undefined) fail("a range takes --first and --count, the leader --end");

/** One record on stdout: length, JSON, payload. */
function record(header: Record<string, unknown>, payload?: Uint8Array): void {
  const json = new TextEncoder().encode(JSON.stringify({ ...header, bytes: payload?.length ?? 0 }));
  const len = new Uint8Array(4);
  new DataView(len.buffer).setUint32(0, json.length, true);
  writeAll(len);
  writeAll(json);
  if (payload) writeAll(payload);
}

function writeAll(bytes: Uint8Array): void {
  let at = 0;
  while (at < bytes.length) {
    try {
      at += writeSync(1, bytes, at, bytes.length - at);
    } catch (e) {
      // A pipe that is full answers EAGAIN on a non-blocking descriptor: wait.
      if ((e as { code?: string }).code !== "EAGAIN") throw e;
      Bun.sleepSync(1);
    }
  }
}

loadNativeHost();
const extras = native();
const mod = (await import(pathToFileURL(resolve(entry)).href)) as { default?: GameToken };
if (!mod.default) fail(`${entry} has no default export: export default og.game({ ... })`);
const projectText = root ? (() => {
  try {
    return readFileSync(join(root, "project.ogt"), "utf8");
  } catch {
    return undefined;
  }
})() : undefined;
const profile = projectText ? (/^\s*profile\s+"(2d|3d)"/m.exec(projectText)?.[1] as "2d" | "3d" | undefined) : undefined;
const created = createEngine(mod.default!, {
  ...(seed === undefined ? {} : { seed }),
  ...(root ? { root } : {}),
  ...(projectText === undefined ? {} : { project: projectText }),
  ...(profile ? { profile } : {}),
  ...(locale ? { locale } : {}),
  deferStart: true,
});
if (!created.ok) fail(`${entry}: ${created.reason}`);
const engine: Engine = created.value;
if (clock !== undefined) engine.setClock(fixedClock(engine, clock));
engine.attachPlatform(nativePlatform(undefined));

const rate = engine.rate;
if (rate % fps !== 0) {
  const divisors = [];
  for (let d = 1; d <= rate; d++) if (rate % d === 0) divisors.push(d);
  const below = divisors.filter((d) => d < fps).pop();
  const above = divisors.find((d) => d > fps);
  fail(`--fps ${fps} does not divide the game's tick rate ${rate}: a frame would fall between ticks; the nearest that do are ${[below, above].filter((d) => d !== undefined).join(" and ")}`);
}
const step = rate / fps;
const tickMs = 1000 / rate;
const t0 = Math.round(fromSeconds * rate);
/** The tick frame `i` shows. */
const tickOf = (i: number): number => t0 + i * step;
/** The tick a range starting at frame `i` starts drawing at (its first warm-up frame). */
const warmTick = (i: number): number => Math.max(0, tickOf(i) - warmup * step);

// The renderer: the leader attaches one too, so GameDesc.start sees the same
// play surface (og.render.capabilities) in every process of the render.
const attached = JSON.parse(extras.og_node_attach_offscreen(engine.id, size.width, size.height)) as { ok: boolean; value?: string; reason?: string };
if (!attached.ok && !leader) fail(`no renderer (${attached.reason}): og render needs a headless renderer (og doctor)`);
const tier = attached.ok ? attached.value! : "none";
const started = engine.advance(0);
if (!started.ok) fail(started.reason);

const sink = (() => {
  const opened = openAudio(engine.id, audioOut ? "capture" : "null");
  if (!opened.ok) fail(`audio: ${opened.reason}`);
  return opened.value;
})();

const warnings: string[] = [];
function drain(): void {
  for (const o of engine.takeOutputs()) {
    if (!out) continue;
    const path = resolve(out, o.file);
    const inside = relative(out, path);
    if (inside === "" || inside.startsWith(".." + sep) || inside === ".." || isAbsolute(inside)) continue;
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, o.kind === "png" || o.kind === "wav" ? Uint8Array.from(atob(o.data), (c) => c.charCodeAt(0)) : o.data);
    process.stderr.write(`tick ${o.tick} ${o.action}: wrote ${path}\n`);
  }
  for (const l of engine.takeLog()) {
    const line = `${l.warning ? "WARN" : "INFO"} tick ${l.tick}: ${l.text}`;
    if (l.warning) warnings.push(line);
    process.stderr.write(line + "\n");
  }
}
drain();

if (commands !== undefined) {
  const report = engine.probe.commands(commands) as { warnings: string[] } | null;
  drain();
  if (!report) fail("--commands: the script is malformed (the reason is above)");
  for (const w of report!.warnings) {
    warnings.push(`WARN ${w}`);
    process.stderr.write(`WARN ${w}\n`);
  }
}
if (restore) {
  const r = JSON.parse(extras.og_node_render_restore(engine.id, readFileSync(restore))) as { ok: boolean; value?: number; reason?: string };
  if (!r.ok) fail(`--restore ${restore}: ${r.reason}`);
}

let quit = false;
const tick = (): number => engine.probe.tick();

/** Simulate to tick `t` without drawing. */
function simulate(t: number): void {
  const n = t - tick();
  if (n <= 0) return;
  const r = engine.advance(n);
  if (!r.ok) fail(r.reason);
  sink.pump();
  drain();
  // The probe takes no tick after a `quit` step.
  if (tick() < t) quit = true;
}

/** Bring the world to tick `t` and draw it there. */
function drawAt(t: number): void {
  if (t > tick()) simulate(t - 1);
  if (quit) return;
  const answer = engine.frame(t > tick() ? tickMs : 0);
  if (!answer.ok) fail(answer.reason ?? "the frame failed");
  sink.pump();
  drain();
  if (answer.quit) quit = true;
  if (!answer.rendered) fail("the frame was not drawn");
}

if (leader) {
  for (const s of [...snapshots].sort((a, b) => a.frame - b.frame)) {
    const at = warmTick(s.frame);
    simulate(at);
    if (tick() !== at) fail(`the world stopped at tick ${tick()} before snapshot tick ${at} (a quit step?)`);
    const blob = extras.og_node_render_snapshot(engine.id);
    if (blob.length === 0) fail(`the engine refused a snapshot at tick ${at}`);
    writeFileSync(s.file, blob);
  }
  simulate(tickOf(end!));
} else {
  const from = tickOf(first!);
  // Warm-up frames on the frame grid before the range: drawn, not sent.
  for (let t = Math.max(tick(), warmTick(first!)); t < from && !quit; t += step) drawAt(t);
  let sent = 0;
  for (let i = first!; i < first! + count! && !quit; i++) {
    const t = tickOf(i);
    drawAt(t);
    if (quit) break;
    const rgba = extras.og_node_render_frame(engine.id);
    if (rgba.length !== size.width * size.height * 4) fail(`frame ${i} read back ${rgba.length} bytes, not ${size.width}x${size.height} RGBA`);
    const sha256 = createHash("sha256").update(rgba).digest("hex");
    record({ kind: "frame", index: i, tick: t, time: t / rate, width: size.width, height: size.height, sha256 }, rgba);
    sent++;
  }
  // The audio runs to the range's end, the last frame's span included.
  if (audioOut && !quit) simulate(tickOf(first! + count!));
  if (sent < count!) warnings.push(`WARN the script quit at tick ${tick()}: ${sent} of ${count} frames`);
}

let wavSha256: string | undefined;
if (audioOut) {
  const wav = sink.wav();
  mkdirSync(dirname(audioOut), { recursive: true });
  writeFileSync(audioOut, wav);
  wavSha256 = createHash("sha256").update(wav).digest("hex");
}
sink.close();
const dump = engine.probe.dump({ what: "world" }) as unknown as { hash: string };
record({ kind: "end", tick: tick(), hash: dump.hash, rate, fromTick: t0, step, tier, quit, warnings, wavSha256 });
process.exit(0);
