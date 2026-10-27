// `og replay` on the native host: a game's simulation recorded tick by tick
// with this engine, or a recorded one replayed and compared, as the evidence
// that an engine upgrade does not change it (crates/og-cli/src/replay.rs).
//
//   bun src/node/replay.ts --entry <game.native.js> [--root <project>] [--project-entry <rel>]
//       --record <trace.json> --commands "t:action[:args];..." [--ticks N] [--seed S]
//       [--clock <UTC ms>] [--locale <tag>] [--keyframe-every K] [--json]
//   bun src/node/replay.ts --entry <game.native.js> [--root <project>] --compare <trace.json> [--json]
//
// Both run the game as `og run --no-render --commands` does (headless, no
// renderer: rendering never feeds back into the simulation), one tick per
// frame. After each tick the world's portable hash (og-net's
// `Snapshot::portable_hash`: by type, field and entity name, independent of
// layout hashes and storage order) is taken; every K ticks and at the last
// one the world's save is kept as a keyframe.
//
// - --record writes the trace: { schema: 1, kind: "og-replay", engine,
//   project: { id, entry, bundleSha256 }, seed, clock, locale, commands,
//   ticks, startTick, table, hashes, keyframes: [{ tick, save }] }. `table`
//   is every type with its fields the run registered: the hash's domain.
//   `bundleSha256` is the bundle's with the machine's paths left out
//   (bundleHash): a change of it is a change of the game, not the engine.
// - --compare replays the trace's script, seed, clock and locale and hashes
//   each tick within the trace's table, so what a newer engine added is left
//   out and what it removed differs. At the first keyframe at or after the
//   first divergent tick the recorded save is diffed against the replayed
//   world (og-net's `state::diff`, local the recorded, remote the replayed).
//   Prints { same, recorded: { engine, ticks }, replayed: { engine, ticks },
//   sourceChanged, ticksCompared, firstDivergentTick, keyframe: { tick, diff }
//   | null, divergentKeyframes }. Exit 0 the same, 3 diverged.
//
// Exit 1 on a failure (the game failed, a trace that cannot be read), 2 on
// a usage error.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve, sep } from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";
import { loadNativeHost, native } from "./index.ts";
import { openAudio } from "./audio.ts";
import { nativePlatform } from "./platform.ts";
import { createEngine, fixedClock, type Engine, type GameToken } from "../runtime.ts";

/** Fields of a type the hash reads, by type name. */
type Table = Record<string, string[]>;

export interface ReplayTrace {
  schema: 1;
  kind: "og-replay";
  engine: string;
  project: { id: string | null; entry: string; bundleSha256: string };
  seed: number | null;
  clock: number | null;
  locale: string | null;
  commands: string;
  ticks: number;
  startTick: number;
  table: Table;
  hashes: string[];
  keyframes: { tick: number; save: string }[];
}

const argv = process.argv.slice(2);
let entry = "";
let root: string | undefined;
let projectEntry: string | undefined;
let recordPath: string | undefined;
let comparePath: string | undefined;
let commands: string | undefined;
let ticks: number | undefined;
let seed: number | undefined;
let clock: number | undefined;
let locale: string | undefined;
let keyframeEvery = 30;
let asJson = false;
for (let i = 0; i < argv.length; i++) {
  const a = argv[i]!;
  const next = () => {
    const v = argv[++i];
    if (v === undefined) usage(`${a} needs a value`);
    return v!;
  };
  if (a === "--entry") entry = next();
  else if (a === "--root") root = resolve(next());
  else if (a === "--project-entry") projectEntry = next();
  else if (a === "--record") recordPath = resolve(next());
  else if (a === "--compare") comparePath = resolve(next());
  else if (a === "--commands") commands = next();
  else if (a === "--ticks") ticks = Number(next());
  else if (a === "--seed") seed = Number(next());
  else if (a === "--clock") clock = Number(next());
  else if (a === "--locale") locale = next();
  else if (a === "--keyframe-every") keyframeEvery = Number(next());
  else if (a === "--json") asJson = true;
  else usage(`unknown argument '${a}'`);
}

function usage(reason: string): never {
  process.stderr.write(`og replay: ${reason}\n`);
  process.exit(2);
}

function fail(reason: string): never {
  process.stderr.write(`og replay: ${reason}\n`);
  process.exit(1);
}

if (!entry) usage("no game entry: pass --entry <file>");
if ((recordPath === undefined) === (comparePath === undefined)) usage("pass one of --record <trace.json> or --compare <trace.json>");

let trace: ReplayTrace | undefined;
if (comparePath) {
  try {
    trace = JSON.parse(readFileSync(comparePath, "utf8")) as ReplayTrace;
  } catch (e) {
    fail(`${comparePath}: ${e instanceof Error ? e.message : String(e)}`);
  }
  if (trace!.kind !== "og-replay" || trace!.schema !== 1) fail(`${comparePath} is not an og replay trace (schema 1)`);
  commands = trace!.commands;
  ticks = trace!.ticks;
  seed = trace!.seed ?? undefined;
  clock = trace!.clock ?? undefined;
  locale = trace!.locale ?? undefined;
} else {
  if (commands === undefined) usage("og replay record needs --commands or --commands-file");
  if (ticks !== undefined && (!Number.isInteger(ticks) || ticks <= 0)) usage("--ticks takes a whole number of fixed steps");
  if (!Number.isInteger(keyframeEvery) || keyframeEvery <= 0) usage("--keyframe-every takes a whole number of ticks");
}
if (seed !== undefined && (!Number.isInteger(seed) || seed < 0)) usage("--seed takes a whole number");
if (clock !== undefined && !(Number.isFinite(clock) && clock > 0)) usage("--clock takes UTC milliseconds since 1970, such as 1767225600000");

/** This package's version: the engine the run is on. */
function engineVersion(): string {
  const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "package.json");
  return (JSON.parse(readFileSync(pkg, "utf8")) as { version: string }).version;
}

loadNativeHost();
const extras = native();
/**
 * The game's code as its bundle holds it: the sha256 of the bundle with what
 * names the machine left out (this package's absolute path in its engine
 * imports, the module-path comments the bundler writes relative to where og
 * ran), so the same game hashes the same in any checkout or install.
 */
function bundleHash(file: string): string {
  const pkg = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
  const text = readFileSync(file, "utf8")
    .split(pkg + sep)
    .join("@origingame/engine/")
    .split("\n")
    .filter((line) => !line.startsWith("// "))
    .join("\n");
  return createHash("sha256").update(text).digest("hex");
}
const bundleSha256 = bundleHash(entry);
const mod = (await import(pathToFileURL(resolve(entry)).href)) as { default?: GameToken };
if (!mod.default) fail(`${entry} has no default export: export default og.game({ ... })`);

const projectText = root && existsSync(join(root, "project.ogt")) ? readFileSync(join(root, "project.ogt"), "utf8") : undefined;
const profile = projectText === undefined ? undefined : (/^\s*profile\s+"(2d|3d)"/m.exec(projectText)?.[1] as "2d" | "3d" | undefined);
const projectId = projectText === undefined ? null : (/^\s*id\s+"([^"]*)"/m.exec(projectText)?.[1] ?? null);

const created = createEngine(mod.default!, {
  ...(seed === undefined ? {} : { seed }),
  ...(root ? { root } : {}),
  ...(projectText === undefined ? {} : { project: projectText }),
  ...(profile ? { profile } : {}),
  ...(locale ? { locale } : {}),
});
if (!created.ok) fail(`${entry}: ${created.reason}`);
const engine: Engine = created.value;
if (clock !== undefined) engine.setClock(fixedClock(engine, clock));
engine.attachPlatform(nativePlatform());
const opened = openAudio(engine.id, "null");
if (!opened.ok) fail(`audio: ${opened.reason}`);
const sink = opened.value;

/** The run's log on stderr; the outputs of `screenshot`/`dump_*` steps are not kept. */
function drain(): void {
  engine.takeOutputs();
  for (const l of engine.takeLog()) if (l.warning) process.stderr.write(`WARN tick ${l.tick}: ${l.text}\n`);
}
drain();

const scheduled = engine.probe.commands(commands!) as { scheduled: number; untilTick: number; warnings: string[] } | null;
drain();
if (!scheduled) fail("--commands: the script is malformed (the reason is above)");
for (const w of scheduled!.warnings) process.stderr.write(`WARN ${w}\n`);

type Hashed = { hash?: string; table?: Table; error?: string };
function hashNow(table: string): Hashed {
  const h = JSON.parse(extras.og_node_replay_hash(engine.id, table)) as Hashed;
  if (h.error) fail(`the world's hash: ${h.error}`);
  return h;
}

function snapshot(): string {
  const save = engine.world.snapshot();
  if (!save.ok) fail(`the engine refused a snapshot (${save.reason})`);
  return save.value;
}

const tickMs = 1000 / engine.rate;
const table: Table = {};
const tableJson = trace ? JSON.stringify(trace.table) : "";
const hashes: string[] = [];
const keyframes: { tick: number; save: string }[] = [];
const recordedKeyframes = new Map((trace?.keyframes ?? []).map((k) => [k.tick, k.save]));
let startTick: number | null = null;
let lastTick = 0;
let firstDivergentTick: number | null = null;
let ticksCompared = 0;
let keyframe: { tick: number; diff: unknown } | null = null;
const divergentKeyframes: number[] = [];

/** One tick's hash taken: recorded, or compared with the trace's. */
function take(tick: number): void {
  if (startTick === null) startTick = tick;
  if (!trace) {
    const h = hashNow("");
    for (const [name, fields] of Object.entries(h.table!)) table[name] ??= fields;
    hashes.push(h.hash!);
    return;
  }
  const want = trace.hashes[tick - trace.startTick];
  if (want === undefined) return;
  const got = hashNow(tableJson).hash!;
  ticksCompared++;
  const differs = got !== want;
  if (differs && firstDivergentTick === null) firstDivergentTick = tick;
  const save = recordedKeyframes.get(tick);
  if (save !== undefined) {
    if (differs) divergentKeyframes.push(tick);
    if (firstDivergentTick !== null && keyframe === null) {
      keyframe = { tick, diff: JSON.parse(extras.og_node_replay_diff(engine.id, save)) as unknown };
    }
  }
}

for (;;) {
  const answer = engine.frame(tickMs);
  if (!answer.ok) fail(`the game failed at tick ${lastTick + 1}: ${answer.reason ?? "the frame failed"}`);
  sink.pump();
  drain();
  const tick = answer.tick ?? 0;
  const stepped = answer.ticks ?? 0;
  if (stepped > 1) fail(`a frame took ${stepped} ticks: the replay hashes one tick per frame`);
  if (stepped === 1) {
    take(tick);
    lastTick = tick;
    // The last tick's keyframe is taken once the run ends, below.
    if (!trace && tick % keyframeEvery === 0) keyframes.push({ tick, save: snapshot() });
  }
  if (answer.quit) break;
  if (ticks !== undefined && tick >= ticks) break;
  if (ticks === undefined && (answer.pending ?? (tick > scheduled!.untilTick ? 0 : 1)) === 0) break;
}
sink.close();

if (!trace) {
  if (keyframes.at(-1)?.tick !== lastTick) keyframes.push({ tick: lastTick, save: snapshot() });
  const out: ReplayTrace = {
    schema: 1,
    kind: "og-replay",
    engine: engineVersion(),
    project: { id: projectId, entry: projectEntry ?? entry, bundleSha256 },
    seed: seed ?? null,
    clock: clock ?? null,
    locale: locale ?? null,
    commands: commands!,
    ticks: lastTick,
    startTick: startTick ?? lastTick,
    table,
    hashes,
    keyframes,
  };
  mkdirSync(dirname(recordPath!), { recursive: true });
  writeFileSync(recordPath!, JSON.stringify(out) + "\n");
  const summary = { trace: recordPath!, ticks: lastTick, hashes: hashes.length, keyframes: keyframes.length, engine: out.engine };
  process.stdout.write(asJson ? JSON.stringify(summary) + "\n" : `recorded ${hashes.length} tick hashes (ticks ${out.startTick}..${lastTick}) and ${keyframes.length} keyframes on engine ${out.engine} into ${recordPath}\n`);
  process.exit(0);
}

const t = trace;
// A replay that stopped before the recording's last tick diverges there.
if (firstDivergentTick === null && ticksCompared < t.hashes.length) firstDivergentTick = t.startTick + ticksCompared;
const same = firstDivergentTick === null;
const report = {
  same,
  recorded: { engine: t.engine, ticks: t.ticks },
  replayed: { engine: engineVersion(), ticks: lastTick },
  sourceChanged: t.project.bundleSha256 !== bundleSha256,
  ticksCompared,
  firstDivergentTick,
  keyframe,
  divergentKeyframes,
};
if (asJson) process.stdout.write(JSON.stringify(report) + "\n");
else {
  const lines = [
    same
      ? `same: ${ticksCompared} ticks replayed on engine ${report.replayed.engine} hash as recorded on ${t.engine}`
      : `diverged at tick ${firstDivergentTick} (${ticksCompared} ticks compared; recorded on ${t.engine}, replayed on ${report.replayed.engine})`,
  ];
  if (report.sourceChanged) lines.push("the game's bundle changed since the recording: re-record it");
  if (divergentKeyframes.length > 0) lines.push(`keyframes that differ: ${divergentKeyframes.join(", ")}`);
  if (keyframe) lines.push(...describeDiff(keyframe));
  process.stdout.write(lines.join("\n") + "\n");
}
process.exit(same ? 0 : 3);

/** A few lines naming what the keyframe's diff found. */
function describeDiff(k: { tick: number; diff: unknown }): string[] {
  const d = k.diff as {
    error?: string;
    components?: { entity: string; component: string; fields?: { field: string; local: unknown; remote: unknown }[]; only?: string }[];
    entities?: { onlyLocal: string[]; onlyRemote: string[] };
    resources?: { resource: string; fields?: { field: string; local: unknown; remote: unknown }[] }[];
    events?: { event: string; local: number; remote: number }[];
    blocks?: { block: string }[];
    allocator?: boolean;
  };
  if (d.error) return [`keyframe ${k.tick}: ${d.error}`];
  const out = [`keyframe ${k.tick} (recorded → replayed):`];
  for (const c of (d.components ?? []).slice(0, 8)) {
    if (c.only) out.push(`  ${c.entity} ${c.component}: only ${c.only === "local" ? "recorded" : "replayed"}`);
    else for (const f of c.fields ?? []) out.push(`  ${c.entity} ${c.component}.${f.field}: ${JSON.stringify(f.local)} → ${JSON.stringify(f.remote)}`);
  }
  for (const r of (d.resources ?? []).slice(0, 4)) for (const f of r.fields ?? []) out.push(`  resource ${r.resource}.${f.field}: ${JSON.stringify(f.local)} → ${JSON.stringify(f.remote)}`);
  if (d.entities && d.entities.onlyLocal.length + d.entities.onlyRemote.length > 0) out.push(`  entities only recorded: ${d.entities.onlyLocal.join(", ") || "none"}; only replayed: ${d.entities.onlyRemote.join(", ") || "none"}`);
  for (const e of (d.events ?? []).slice(0, 4)) out.push(`  event ${e.event}: ${e.local} queued → ${e.remote}`);
  for (const b of (d.blocks ?? []).slice(0, 4)) out.push(`  state block ${b.block} differs`);
  if (d.allocator) out.push("  the entity allocator differs");
  return out;
}
