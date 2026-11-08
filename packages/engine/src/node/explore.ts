// `og explore`: a search of a game's states for bugs its playtests do not
// script (crates/og-cli/src/explore.rs builds the game and runs this).
//
//   bun src/node/explore.ts --entry <game.native.js> [--root <project>] --out <dir>
//       [--ticks N] [--seconds S] [--memory MB] [--jobs N] [--seed S] [--horizon T]
//       [--first] [--no-shrink] [--web auto|on|off] [--json] [--project <og run's argument>]
//   bun src/node/explore.ts --entry <bundle> [--root <project>] --replay <repro.json> --host node|web
//
// The method is Go-Explore's (Ecoffet et al. 2019, "Go-Explore: a New
// Approach for Hard-Exploration Problems", and the 2021 Nature paper "First
// return, then explore"): an archive of cells, each a state abstraction with
// the shortest input trace known to reach it and the world's snapshot there;
// choose a promising cell, return to it by restoring its snapshot (the
// engine's whole-world snapshot replaces Go-Explore's emulator restore), then
// explore from it with random inputs, adding every new cell met. Its parts
// come from these, re-implemented here (no code is copied; the sources and
// their licences are in third_party/explore.md):
// - cells: Go-Explore's domain-knowledge cells, read through reflection
//   (probe.observe): each game field bucketed, integers by value up to 16
//   and by power of two beyond, numbers in -1..1 in eighths, others by
//   power of two (AFL's hit-count buckets applied to state, after IJON's
//   "annotating the state space", Aschermann et al., S&P 2020); NaN and the
//   infinities apart. GameDesc.explore names the fields instead. The time
//   is a feature too, in 64 slices of the horizon: without it the
//   shorter-trace rule keeps every cell at its earliest tick, and a state
//   that matters only late (a per-tick draw) is never returned to.
// - selection: Go-Explore's count-based weights, 1/sqrt(1 + times chosen)
//   plus 1/sqrt(1 + times seen), and a bonus for a cell that brought a new
//   feature (a field's bucket never seen: AFL's "new bits", libFuzzer's
//   features) or a new assertion firing (Antithesis: sometimes assertions as
//   the search's guidance), decaying as it is chosen. Half the returns
//   choose a state (a cell's features less the time) and then one of its
//   times, so the slices widen the search without thinning it; half choose
//   a time slice and then one of its cells, so every time is returned to.
// - a cell met again by a shorter trace takes it (Go-Explore's trajectory
//   replacement), so the archive's traces stay short.
// - inputs go through the recorded input path, the command channel: key and
//   gamepad presses of the game's actions (og.input.actions) and its intents
//   (the `intent` action), held 1 to 3 ticks, the next 0 to 3 ticks later
//   (so presses overlap and two inputs can fall a tick apart), and waits.
//   A cell is kept only with nothing held, and restored with its input
//   (RestoreDesc.session), so its trace replays to its exact world.
// - a finding is the first violation of a site (og.assert.always and
//   unreachable, the default invariants, playtest invariants), the first
//   error of a system (it threw), or the first firing of a sometimes,
//   reachable or each bucket, with the trace that got there. It is shrunk by delta debugging (Zeller and Hildebrandt 2002,
//   ddmin) over its presses, each candidate replayed from tick 0, then
//   replayed whole in a fresh `og run` process and, where this tree's web
//   module is built, on the wasm host: the same world_hash on every replay
//   is the finding's determinism, and a difference is reported as one.
//
// Workers (--jobs) are separate processes, each its own engine with its own
// search seed and a share of the tick and memory budgets; the coordinator
// merges their findings (the shortest trace per site) and their coverage.
// The report is facts: what was found, how, at what cost; never a verdict.

import { appendFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createEngine, declaredGameExplore, fixedClock, type Engine, type GameToken } from "../runtime.ts";
import { localPlatform } from "../platform-local.ts";
import { PLAYTEST_CLOCK } from "../playtest.ts";
import { og } from "../index.ts";

declare const Bun: {
  spawn(cmd: string[], o?: { env?: Record<string, string | undefined>; stdout?: string; stderr?: string; cwd?: string }): { exited: Promise<number>; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
};

// ── arguments ────────────────────────────────────────────────────────────

const argv = process.argv.slice(2);
const opt: Record<string, string> = {};
const flags = new Set<string>();
for (let i = 0; i < argv.length; i++) {
  const a = argv[i]!;
  if (["--json", "--no-shrink", "--first"].includes(a)) flags.add(a);
  else if (a.startsWith("--")) {
    const v = argv[++i];
    if (v === undefined) usage(`${a} needs a value`);
    opt[a] = v!;
  } else usage(`unexpected argument '${a}'`);
}
function usage(reason: string): never {
  process.stderr.write(`og explore: ${reason}\n`);
  process.exit(2);
}
function num(name: string, fallback: number, min = 0): number {
  const raw = opt[name];
  if (raw === undefined) return fallback;
  const v = Number(raw);
  if (!Number.isFinite(v) || v < min) usage(`${name} takes a number of at least ${min}`);
  return v;
}

const entry = opt["--entry"] ?? usage("no game entry: pass --entry <file>");
const root = opt["--root"] ? resolve(opt["--root"]) : undefined;
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

// ── the engine, on either host ───────────────────────────────────────────

/** A fresh engine of the game on the host this process loaded. */
async function boot(): Promise<{ engine: Engine; game: GameToken }> {
  const mod = (await import(pathToFileURL(resolve(entry)).href)) as { default?: GameToken };
  if (!mod.default) fail(`${entry} has no default export: export default og.game({ ... })`);
  const project = root && existsSync(join(root, "project.ogt")) ? readFileSync(join(root, "project.ogt"), "utf8") : undefined;
  const profile = project === undefined ? undefined : (/^\s*profile\s+"(2d|3d)"/m.exec(project)?.[1] as "2d" | "3d" | undefined);
  const created = createEngine(mod.default!, { ...(root ? { root } : {}), ...(project === undefined ? {} : { project }), ...(profile ? { profile } : {}) });
  if (!created.ok) fail(`${entry}: ${created.reason}`);
  const engine = created.value;
  // As og run and runPlaytest: the playtest clock, a local platform in memory.
  engine.setClock(fixedClock(engine, PLAYTEST_CLOCK));
  engine.attachPlatform(localPlatform());
  drain(engine);
  return { engine, game: mod.default! };
}

async function loadHost(host: string): Promise<void> {
  if (host === "node") (await import("./index.ts")).loadNativeHost();
  else if (host === "web") {
    const dir = join(pkg, "dist", "web");
    const { loadWebHost } = await import("../web/host.ts");
    await loadWebHost(await import(join(dir, "origin-engine.js")), readFileSync(join(dir, "origin-engine_bg.wasm")));
  } else usage(`--host takes node or web, not '${host}'`);
}

function fail(reason: string): never {
  process.stderr.write(`og explore: ${reason}\n`);
  process.exit(1);
}

/** The run's log and outputs, dropped (warnings to stderr once). */
const warned = new Set<string>();
function drain(engine: Engine): void {
  engine.takeOutputs();
  for (const l of engine.takeLog())
    if (l.warning && !warned.has(l.text)) {
      warned.add(l.text);
      if (warned.size <= 20) process.stderr.write(`WARN tick ${l.tick}: ${l.text}\n`);
    }
}

function tickOf(engine: Engine): number {
  return (engine.probe.tick() as number) ?? 0;
}

function hashOf(engine: Engine): string {
  return (engine.probe.dump({ what: "world" }) as { hash: string }).hash;
}

// ── traces ───────────────────────────────────────────────────────────────

/** One step of a trace: at a tick, a command-channel action; `unit` groups a press's down and up. */
interface Step {
  tick: number;
  action: string;
  args: string;
  unit: number;
}

/** The trace as a command script from tick 0: each step at its tick's time, then quit at `end`. */
function script(steps: readonly Step[], rate: number, end: number | null): string {
  const at = (t: number) => String(t / rate);
  const lines = [...steps].sort((a, b) => a.tick - b.tick).map((s) => `${at(s.tick)}:${s.action}${s.args ? `:${s.args}` : ""}`);
  if (end !== null) lines.push(`${at(end)}:quit`);
  return lines.join(";");
}

type Event = { outcome: "fired" | "violated" | "error"; tick: number; kind: string; origin: string; name: string; location: string; key?: string; details?: unknown; entity?: string | null; component?: string | null; field?: string | null; value?: unknown; message?: string };

const siteOf = (e: Event) => `${e.outcome}\u0000${e.kind}\u0000${e.origin}\u0000${e.name}\u0000${e.location}\u0000${e.key ?? ""}`;

/** A system that threw, as a finding's event: its site is the system and the error's first line. */
function errorEvent(reason: string, at: number): Event {
  const tick = Number(/^tick (\d+),/.exec(reason)?.[1] ?? at);
  const first = (reason.split("\n")[0] ?? reason).replace(/^tick \d+, /, "");
  const system = /^system '([^']+)'/.exec(first)?.[1] ?? "";
  return { outcome: "error", tick, kind: "error", origin: "game", name: first.replace(/^system '[^']+': /, ""), location: system, message: reason };
}

function events(engine: Engine): Event[] {
  return ((engine.probe.assertionEvents() as unknown as { events?: Event[] } | null)?.events ?? []).filter((e) => e.outcome === "fired" || e.outcome === "violated");
}

/**
 * Plays `steps` from the snapshot `save` (tick 0 of the game) until `end`, through the command
 * channel one tick at a time, on a report started afresh: what fired and was violated, and the
 * world's hash at the end (a fresh `og run` of the script, frame by frame, is checked against it).
 */
function replay(engine: Engine, save: string, steps: readonly Step[], end: number): { events: Event[]; hash: string; tick: number } {
  const restored = engine.world.restore(save, { session: true });
  if (!restored.ok) fail(`the start snapshot does not restore: ${restored.reason}`);
  engine.probe.assertions(true);
  engine.probe.assertionEvents();
  // Without the script's quit: a quit stays with the engine, and the next replay would end at once.
  const scheduled = steps.length === 0 || engine.probe.commands(script(steps, engine.rate, null));
  drain(engine);
  if (!scheduled) fail("a trace's script is malformed (the reason is above)");
  // One tick a step, as the search played it: due command steps run before each tick.
  const seen: Event[] = [];
  while (tickOf(engine) < end) {
    const answer = engine.advance(1);
    drain(engine);
    seen.push(...events(engine));
    if (!answer.ok) {
      seen.push(errorEvent(answer.reason, tickOf(engine)));
      break;
    }
  }
  return { events: seen, hash: hashOf(engine), tick: tickOf(engine) };
}

// ── cells ────────────────────────────────────────────────────────────────

type Observed = { tick: number; resources: Record<string, Record<string, unknown>>; components: Record<string, Record<string, Record<string, unknown>>> };

function bucket(v: unknown): string {
  if (typeof v === "boolean" || typeof v === "string") return String(v);
  if (typeof v !== "number") {
    if (Array.isArray(v)) return `[${v.map(bucket).join(",")}]`;
    if (v && typeof v === "object") return `{${Object.entries(v).map(([k, x]) => `${k}:${bucket(x)}`).join(",")}}`;
    return String(v);
  }
  if (Number.isNaN(v)) return "nan";
  if (!Number.isFinite(v)) return v > 0 ? "inf" : "-inf";
  if (Number.isInteger(v) && Math.abs(v) <= 16) return String(v);
  if (Math.abs(v) <= 1) return `e${Math.floor(v * 8)}`;
  return `${v < 0 ? "-" : ""}p${Math.ceil(Math.log2(Math.abs(v)))}`;
}

/** The cell's features: `Record.field=bucket`, for the fields the desc names (or every one), less the ignored. */
function features(o: Observed, cells: readonly string[], ignore: readonly string[], slice: number): string[] {
  const wants = (record: string, field: string) => {
    const path = `${record}.${field}`;
    if (ignore.includes(record) || ignore.includes(path)) return false;
    return cells.length === 0 || cells.includes(record) || cells.includes(path);
  };
  // The time, in 64 slices of the horizon: a state the game reaches only late (a timer, a per-tick
  // draw of og.Rng) keeps cells there, where the shorter-trace rule alone would keep each cell at
  // its earliest tick. The search chooses a state first and one of its times second, so the
  // slices widen the search without thinning it; ignore "@tick" for a game where time is no state.
  const out: string[] = ignore.includes("@tick") ? [] : [`@tick=${Math.floor(o.tick / slice)}`];
  for (const [r, fields] of Object.entries(o.resources)) for (const [f, v] of Object.entries(fields)) if (wants(r, f)) out.push(`${r}.${f}=${bucket(v)}`);
  for (const [c, rows] of Object.entries(o.components)) {
    // A named entity's fields by its name; unnamed ones as a count and each field's set of buckets.
    const unnamed: Record<string, Set<string>> = {};
    let count = 0;
    for (const [id, fields] of Object.entries(rows)) {
      const named = !id.startsWith("#");
      if (!named) count++;
      for (const [f, v] of Object.entries(fields))
        if (wants(c, f)) {
          if (named) out.push(`${id}.${c}.${f}=${bucket(v)}`);
          else (unnamed[f] ??= new Set()).add(bucket(v));
        }
    }
    if (count > 0) out.push(`${c}#=${bucket(count)}`);
    for (const [f, set] of Object.entries(unnamed)) out.push(`${c}.${f}={${[...set].sort().join(",")}}`);
  }
  return out.sort();
}

// ── inputs ───────────────────────────────────────────────────────────────

/** One input the search sends: a press (down, held, up) or an intent. */
type Input = { kind: "press"; down: [string, string]; up: [string, string]; label: string } | { kind: "intent"; event: string; payload: string; label: string };

type ExploreDesc = { cells?: string[]; ignore?: string[]; actions?: string[]; intents?: { event: string; values?: Record<string, unknown>[] }[] };
type Binding = { control?: string; negative?: string; positive?: string; up?: string; down?: string; left?: string; right?: string };
type FieldType = { name: string; type: string; min: number | null; max: number | null; variants: string[] };

/** A control as a press through the command channel, or null for one it cannot press. */
function pressOf(control: string | undefined): Input | null {
  if (!control) return null;
  if (control.startsWith("pad.")) {
    const button = control.slice(4);
    if (/X|Y|XY|Trigger|dpad/.test(button)) return null;
    return { kind: "press", down: ["gamepad", button], up: ["gamepad", "-"], label: control };
  }
  if (/^(mouse|wheel|touch)/.test(control)) return null;
  return { kind: "press", down: ["key_down", control], up: ["key_up", control], label: control };
}

/** The payloads of an intent from its reflection: each field's values, the others at defaults, one field varied at a time. */
function payloads(fields: FieldType[]): Record<string, unknown>[] {
  const out: Record<string, unknown>[] = [{}];
  for (const f of fields) {
    let values: unknown[] = [];
    if (f.variants.length > 0) values = f.variants;
    else if (f.type === "bool") values = [true, false];
    else if (/^[iu](32|64)$/.test(f.type) && f.min !== null && f.max !== null && f.max - f.min <= 16) for (let v = f.min; v <= f.max; v++) values.push(v);
    else if (f.min !== null && f.max !== null) values = [f.min, (f.min + f.max) / 2, f.max];
    for (const v of values) out.push({ [f.name]: v });
  }
  return out;
}

function vocabulary(engine: Engine, desc: ExploreDesc, types: Record<string, { kind?: string; fields: FieldType[] }>): Input[] {
  const inputs: Input[] = [];
  const actions = og.input.actions(engine.world as never) as unknown as { name: string; kind: string; bindings: Binding[] }[];
  for (const a of actions) {
    if (desc.actions?.length ? !desc.actions.includes(a.name) : a.kind !== "button") continue;
    // The first binding the channel can press; an axis by each of its directions.
    for (const b of a.bindings) {
      const parts = a.kind === "button" ? [b.control] : [b.control, b.negative, b.positive, b.up, b.down, b.left, b.right];
      const found = parts.map(pressOf).filter((p): p is Input => p !== null);
      if (found.length > 0) {
        inputs.push(...found.map((p) => ({ ...p, label: `${a.name} (${p.label})` })));
        break;
      }
    }
  }
  for (const i of desc.intents ?? []) {
    const t = types[i.event];
    if (!t) fail(`GameDesc.explore.intents names '${i.event}', which is no event the game declared`);
    for (const p of i.values?.length ? i.values : payloads(t.fields)) inputs.push({ kind: "intent", event: i.event, payload: JSON.stringify(p), label: `${i.event} ${JSON.stringify(p)}` });
  }
  return inputs;
}

// ── the search (one worker) ──────────────────────────────────────────────

/** The process's resident memory (bytes). */
const rss = (): number => (process as unknown as { memoryUsage(): { rss: number } }).memoryUsage().rss;

/** A seeded generator (mulberry32): the search's choices replay for a seed. */
function random(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

interface Cell {
  key: string;
  group: Group;
  save: string;
  steps: Step[];
  tick: number;
  chosen: number;
  seen: number;
  bonus: number;
  units: number;
}

/** The cells of one state at different times (their features less `@tick`): chosen first, then one of its cells. */
interface Group {
  cells: Set<Cell>;
  chosen: number;
  seen: number;
  bonus: number;
}

/** What a worker found: per site the first trace that reached it. */
interface Found {
  site: string;
  event: Event;
  steps: Step[];
  /** The worker's tick count when it found it. */
  atTicks: number;
  atMs: number;
}

interface WorkerResult {
  worker: number;
  found: Found[];
  ticks: number;
  iterations: number;
  restores: number;
  cells: number;
  features: number;
  archiveBytes: number;
  peakRssMb: number;
  ms: number;
  stopped: "ticks" | "seconds" | "found" | "done";
  evicted: number;
  inputs: string[];
  cellFields: number;
  fired: string[];
  sitesSeen: unknown[];
}

async function search(worker: number, seed: number, budget: { ticks: number; ms: number; bytes: number }, horizon: number, report: (f: Found) => void): Promise<WorkerResult> {
  const t0 = performance.now();
  const { engine, game } = await boot();
  const desc = declaredGameExplore(game) as ExploreDesc;
  const observed0 = engine.probe.observe(null, true) as unknown as Observed & { types: Record<string, { kind?: string; fields: FieldType[] }> };
  const inputs = vocabulary(engine, desc, observed0.types);
  if (inputs.length === 0) fail("the game defines no button action the command channel can press, and GameDesc.explore names no intent: nothing to explore with");
  const slice = Math.max(1, Math.ceil(horizon / 64));
  const rateOf = engine.rate;
  const cellsOf = (o: Observed) => features(o, desc.cells ?? [], desc.ignore ?? [], slice);
  const rng = random(seed * 2654435761 + worker * 97 + 1);
  const pick = <T>(xs: readonly T[]): T => xs[Math.floor(rng() * xs.length)]!;
  const archive = new Map<string, Cell>();
  const featureSet = new Set<string>();
  const firedSites = new Set<string>();
  let bytes = 0;
  let ticks = 0;
  let iterations = 0;
  let restores = 0;
  let evicted = 0;
  let unitSeq = 0;
  let peakRss = 0;
  let cellFields = 0;

  const groups = new Map<string, Group>();
  const add = (feats: string[], steps: Step[], tick: number, units: number, novelty: number): void => {
    const key = feats.join("|");
    const old = archive.get(key);
    if (old) {
      old.seen++;
      old.group.seen++;
      // A shorter way to the same cell takes its place.
      if (tick < old.tick || (tick === old.tick && units < old.units)) {
        const save = engine.world.snapshot();
        if (save.ok) {
          bytes += save.value.length * 2 - old.save.length * 2;
          Object.assign(old, { save: save.value, steps: [...steps], tick, units });
        }
      }
      return;
    }
    const save = engine.world.snapshot();
    if (!save.ok) fail(`the engine refused a snapshot at tick ${tick}: ${save.reason}`);
    const groupKey = feats.filter((f) => !f.startsWith("@tick=")).join("|");
    let group = groups.get(groupKey);
    if (!group) groups.set(groupKey, (group = { cells: new Set(), chosen: 0, seen: 0, bonus: novelty }));
    else group.bonus += novelty;
    group.seen++;
    const cell: Cell = { key, group, save: save.value, steps: [...steps], tick, chosen: 0, seen: 1, bonus: novelty, units };
    group.cells.add(cell);
    archive.set(key, cell);
    bytes += save.value.length * 2;
    // Over the memory budget: the most chosen, least promising cells go (never the root).
    while (bytes > budget.bytes && archive.size > 1) {
      let worst: Cell | null = null;
      for (const c of archive.values()) if (c.steps.length > 0 && (!worst || weight(c) < weight(worst))) worst = c;
      if (!worst) break;
      archive.delete(worst.key);
      worst.group.cells.delete(worst);
      if (worst.group.cells.size === 0) groups.delete(worst.key.split("|").filter((f) => !f.startsWith("@tick=")).join("|"));
      bytes -= worst.save.length * 2;
      evicted++;
    }
  };
  // Go-Explore's count-based weights, with a bonus for what the cell brought, decaying as it is chosen.
  const weight = (c: { chosen: number; seen: number; bonus: number }) => 1 / Math.sqrt(1 + c.chosen) + 1 / Math.sqrt(1 + c.seen) + c.bonus / (1 + c.chosen);
  const pickWeighted = <T extends { chosen: number; seen: number; bonus: number }>(xs: readonly T[]): T => {
    let total = 0;
    let last: T | undefined;
    for (const x of xs) total += weight(x);
    let r = rng() * total;
    for (const x of xs) {
      last = x;
      if ((r -= weight(x)) <= 0) return x;
    }
    return last!;
  };
  // Half the returns choose a state, then one of its times; half a time slice (uniformly among
  // those with cells), then one of its cells: the first spreads the search over what the game
  // holds, the second over when, which a state that matters only at one tick needs.
  const choose = (): Cell => {
    if (rng() < 0.5) {
      const bySlice = new Map<number, Cell[]>();
      for (const c of archive.values()) {
        const k = Math.floor(c.tick / slice);
        let l = bySlice.get(k);
        if (!l) bySlice.set(k, (l = []));
        l.push(c);
      }
      const keys = [...bySlice.keys()];
      const cells = bySlice.get(keys[Math.floor(rng() * keys.length)]!)!;
      const c = pickWeighted(cells);
      c.group.chosen++;
      return c;
    }
    const group = pickWeighted([...groups.values()]);
    group.chosen++;
    return pickWeighted([...group.cells]);
  };

  const f0 = cellsOf(observed0);
  cellFields = new Set(f0.map((f) => f.split("=")[0])).size;
  for (const f of f0) featureSet.add(f);
  add(f0, [], tickOf(engine), 0, 1);
  drain(engine);
  events(engine);
  let stopped: WorkerResult["stopped"] = "done";
  let stopFirst = false;
  for (;;) {
    // Another worker's first finding stops this one too (the coordinator writes the stop file).
    if (stopFirst || (stopFile && (iterations & 15) === 0 && existsSync(stopFile))) {
      stopped = "found";
      break;
    }
    if (ticks >= budget.ticks) {
      stopped = "ticks";
      break;
    }
    if (performance.now() - t0 >= budget.ms) {
      stopped = "seconds";
      break;
    }
    iterations++;
    const cell = choose();
    cell.chosen++;
    const restored = engine.world.restore(cell.save, { session: true });
    if (!restored.ok) fail(`a cell's snapshot does not restore: ${restored.reason}`);
    restores++;
    const steps = [...cell.steps];
    let units = cell.units;
    // Explore: a few inputs, each held a few ticks; the next starts 0 to 3 ticks later, so
    // presses overlap and an input may follow another in the same or the next tick.
    const pending: Step[] = [];
    let failed: Event | null = null;
    const step = (n: number): void => {
      for (let k = 0; k < n; k++) {
        const now = tickOf(engine);
        for (let j = pending.length - 1; j >= 0; j--)
          if (pending[j]!.tick <= now) {
            const up = pending.splice(j, 1)[0]!;
            steps.push({ ...up, tick: now });
            engine.probe.commands(`0:${up.action}:${up.args}`);
          }
        const r = engine.advance(1);
        ticks++;
        if (!r.ok) {
          failed = errorEvent(r.reason, now);
          return;
        }
      }
    };
    const look = (): void => {
      drain(engine);
      let novelty = 0;
      for (const e of [...events(engine), ...(failed ? [failed] : [])]) {
        const site = siteOf(e);
        if (firedSites.has(site)) continue;
        firedSites.add(site);
        novelty += 2;
        report({ site, event: e, steps: steps.filter((s) => s.tick <= e.tick), atTicks: ticks, atMs: performance.now() - t0 });
        if (stopAtFirst && e.outcome !== "fired") stopFirst = true;
      }
      // A cell is a state with nothing held (its trace ends with every press released), and
      // none after a system threw: the world is half a tick.
      if (pending.length > 0 || failed) return;
      const feats = cellsOf(engine.probe.observe() as unknown as Observed);
      for (const f of feats)
        if (!f.startsWith("@tick=") && !featureSet.has(f)) {
          featureSet.add(f);
          novelty++;
        }
      add(feats, steps, tickOf(engine), units, novelty);
    };
    const n = 1 + Math.floor(rng() * 16);
    for (let i = 0; i < n && ticks < budget.ticks; i++) {
      const at = tickOf(engine);
      if (at >= horizon) break;
      // One in (inputs + 1) is a wait of up to half a second instead: time passes cheaply.
      if (rng() * (inputs.length + 1) < 1) {
        step(1 + Math.floor(rng() * Math.max(1, Math.round(rateOf / 2))));
        look();
        if (failed) break;
        continue;
      }
      const input = pick(inputs);
      const unit = ++unitSeq;
      if (input.kind === "press") {
        steps.push({ tick: at, action: input.down[0], args: input.down[1], unit });
        engine.probe.commands(`0:${input.down[0]}:${input.down[1]}`);
        pending.push({ tick: at + 1 + Math.floor(rng() * 3), action: input.up[0], args: input.up[1], unit });
      } else {
        steps.push({ tick: at, action: "intent", args: `${input.event}:${input.payload}`, unit });
        engine.probe.commands(`0:intent:${input.event}:${input.payload}`);
      }
      units++;
      const gap = Math.floor(rng() * 4);
      step(gap);
      if (gap > 0) look();
      if (failed) break;
    }
    // Release what is held, and at least one tick for the last input.
    if (!failed) {
      step(1);
      while (pending.length > 0 && !failed) step(1);
    }
    look();
    if ((iterations & 63) === 0) peakRss = Math.max(peakRss, rss());
  }
  peakRss = Math.max(peakRss, rss());
  const final = engine.probe.assertions() as unknown as { sites: unknown[] };
  return {
    worker,
    found: [],
    ticks,
    iterations,
    restores,
    cells: archive.size,
    features: featureSet.size,
    archiveBytes: bytes,
    peakRssMb: Math.round(peakRss / 1048576),
    ms: Math.round(performance.now() - t0),
    stopped,
    evicted,
    inputs: inputs.map((i) => i.label),
    cellFields,
    fired: [...firedSites],
    sitesSeen: final.sites,
  };
}

// ── shrinking and verification ───────────────────────────────────────────

/**
 * The trace with its idle time cut: every gap between the ticks it acts on capped at 2 ticks,
 * the first at 2 from tick 0 (same-tick steps stay together, the order stays).
 */
function compact(steps: readonly Step[]): Step[] {
  const ticks = [...new Set(steps.map((s) => s.tick))].sort((a, b) => a - b);
  const to = new Map<number, number>();
  let at = 0;
  let prev = 0;
  for (const t of ticks) {
    at += Math.min(t - prev, 2);
    to.set(t, at);
    prev = t;
  }
  return steps.map((s) => ({ ...s, tick: to.get(s.tick)! }));
}

/**
 * The smallest trace found that still reaches `f.site` no later than it did: its idle time cut
 * (kept only when the finding still happens, so a finding that depends on when keeps its
 * ticks), then ddmin over its presses (a press's down and up go together), then the idle time
 * cut again, and the steps after the finding dropped. Each candidate is replayed from tick 0.
 */
function shrink(engine: Engine, save: string, f: Found, limit0: number): { steps: Step[]; tick: number; tests: number } {
  let tests = 0;
  let limit = limit0;
  const reaches = (steps: Step[]): Event | null => {
    tests++;
    const r = replay(engine, save, steps, limit + 1);
    return r.events.find((e) => siteOf(e) === f.site && e.tick <= limit) ?? null;
  };
  let steps = f.steps;
  const cut = (): void => {
    const c = compact(steps);
    if (c.every((s, i) => s.tick === steps[i]!.tick)) return;
    const e = reaches(c);
    if (e) {
      steps = c;
      limit = e.tick;
    }
  };
  const groups = (steps: Step[]) => [...new Set(steps.map((s) => s.unit))];
  cut();
  let units = groups(steps);
  let n = 2;
  while (units.length >= 2) {
    const size = Math.ceil(units.length / n);
    let reduced = false;
    for (let i = 0; i < units.length; i += size) {
      const drop = new Set(units.slice(i, i + size));
      const candidate = steps.filter((s) => !drop.has(s.unit));
      const e = reaches(candidate);
      if (e) {
        steps = candidate;
        limit = e.tick;
        units = groups(steps);
        n = Math.max(n - 1, 2);
        reduced = true;
        break;
      }
    }
    if (!reduced) {
      if (n >= units.length) break;
      n = Math.min(units.length, n * 2);
    }
  }
  cut();
  // Steps after the finding (its tick moved earlier as presses went) change nothing it shows,
  // and a script's quit at the tick after it must be its last step.
  steps = steps.filter((s) => s.tick <= limit);
  const last = reaches(steps);
  if (!last) fail("a shrunk trace stopped reaching its finding: the replay is not deterministic");
  return { steps, tick: last.tick, tests };
}

async function child(cmd: string[], env?: Record<string, string | undefined>): Promise<{ code: number; out: string; err: string }> {
  const p = Bun.spawn(cmd, { env: env ?? process.env, stdout: "pipe", stderr: "pipe" });
  const [code, out, err] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
  return { code, out, err };
}

/** Whether dist/web is this tree's module (every binding the ABI lists). */
async function webModule(): Promise<string | null> {
  const glue = join(pkg, "dist", "web", "origin-engine.js");
  if (!existsSync(glue)) return "no dist/web";
  const { SYMBOLS } = await import("../generated/abi.ts");
  const exports = (await import(glue)) as Record<string, unknown>;
  const missing = SYMBOLS.filter((s) => typeof exports[s] !== "function");
  return missing.length > 0 ? `dist/web lacks ${missing.length} of this tree's bindings (build it: scripts/check.sh --only wasm-bindgen)` : null;
}

function playtestFile(name: string, finding: { kind: string; script: string; tick: number; hash: string; site: { name: string; kind: string; location: string } }): string {
  const what = finding.kind === "error" ? `the system '${finding.site.location}' throws (${finding.site.name})` : `${finding.site.kind} '${finding.site.name}'`;
  const head = `// ${name}: written by og explore, a minimal input script that ${finding.kind === "reached" ? "reaches" : finding.kind === "error" ? "makes" : "violates"} ${what}\n// at tick ${finding.tick}; it replayed to world hash ${finding.hash}. Copy it into the game's tests/ to keep it.\nimport { expect, test } from "bun:test";\nimport { runPlaytest } from "@origingame/engine/playtest";\n\n`;
  const body =
    finding.kind === "reached"
      ? `test(${JSON.stringify(`the script reaches ${what}`)}, async () => {\n  const run = await runPlaytest({ commands: ${JSON.stringify(finding.script)}, require: [${JSON.stringify(finding.site.name)}] });\n  expect(run.quit).toBe(true);\n});\n`
      : `// It fails until the game is fixed: runPlaytest refuses a run with a ${finding.kind === "error" ? "system error" : "violated assertion"}.\ntest(${JSON.stringify(`the script plays without: ${what}`)}, async () => {\n  const run = await runPlaytest({ commands: ${JSON.stringify(finding.script)} });\n  expect(run.quit).toBe(true);\n});\n`;
  return head + body;
}

// ── modes ────────────────────────────────────────────────────────────────

if (opt["--replay"]) {
  // One repro on one host: its hash and what it fired, as a JSON line.
  await loadHost(opt["--host"] ?? "node");
  const repro = JSON.parse(readFileSync(opt["--replay"], "utf8")) as { steps: Step[]; end: number };
  const { engine } = await boot();
  const save = engine.world.snapshot();
  if (!save.ok) fail(save.reason);
  const r = replay(engine, save.value, repro.steps, repro.end);
  process.stdout.write(JSON.stringify({ host: opt["--host"] ?? "node", hash: r.hash, tick: r.tick, events: r.events }) + "\n");
  process.exit(0);
}

const ticksBudget = Math.floor(num("--ticks", 200000, 1));
const msBudget = num("--seconds", 60, 0.1) * 1000;
const bytesBudget = num("--memory", 256, 1) * 1048576;
const jobs = Math.floor(num("--jobs", 1, 1));
const seed = Math.floor(num("--seed", 1, 0));
const horizon = Math.floor(num("--horizon", 3600, 1));
const stopAtFirst = flags.has("--first");
const stopFile = opt["--stop-file"];

if (opt["--worker"] !== undefined) {
  // A worker of a parallel search: findings as JSON lines as they happen, then its result.
  await loadHost("node");
  const w = Number(opt["--worker"]);
  const result = await search(w, seed, { ticks: ticksBudget, ms: msBudget, bytes: bytesBudget }, horizon, (f) => process.stdout.write(JSON.stringify({ found: f }) + "\n"));
  process.stdout.write(JSON.stringify({ result }) + "\n");
  process.exit(0);
}

// The coordinator.
const out = resolve(opt["--out"] ?? "og-out/explore");
mkdirSync(out, { recursive: true });
const asJson = flags.has("--json");
const t0 = performance.now();
await loadHost("node");
const log = (line: string) => {
  if (!asJson) process.stdout.write(line + "\n");
};
const progress = join(out, "progress.jsonl");
writeFileSync(progress, "");

const results: WorkerResult[] = [];
const bySite = new Map<string, Found & { worker: number }>();
const stopPath = join(out, ".stop");
rmSync(stopPath, { force: true });
const keep = (f: Found, worker: number) => {
  if (stopAtFirst && f.event.outcome !== "fired") writeFileSync(stopPath, "");
  appendFileSync(progress, JSON.stringify({ worker, outcome: f.event.outcome, name: f.event.name, tick: f.event.tick, atTicks: f.atTicks, atMs: Math.round(f.atMs) }) + "\n");
  const old = bySite.get(f.site);
  if (!old || f.event.tick < old.event.tick) bySite.set(f.site, { ...f, worker });
  if (!old) log(`${{ error: "ERROR", violated: "VIOLATION", fired: "reached" }[f.event.outcome]} ${f.event.kind} '${f.event.name}' at tick ${f.event.tick} (worker ${worker}, ${f.atTicks} ticks, ${Math.round(f.atMs)} ms in)`);
};
const share = { ticks: Math.ceil(ticksBudget / jobs), ms: msBudget, bytes: Math.floor(bytesBudget / jobs) };
if (jobs === 1) results.push(await search(0, seed, share, horizon, (f) => keep(f, 0)));
else {
  const self = fileURLToPath(import.meta.url);
  const pass = argv.filter((_, i) => !["--out", "--jobs", "--ticks", "--memory"].includes(argv[i - 1] ?? "") && !["--out", "--jobs", "--ticks", "--memory", "--json"].includes(argv[i] ?? ""));
  await Promise.all(
    Array.from({ length: jobs }, async (_, w) => {
      const p = Bun.spawn(["bun", self, ...pass, "--worker", String(w), "--ticks", String(share.ticks), "--memory", String(share.bytes / 1048576), "--stop-file", stopPath], { stdout: "pipe", stderr: "pipe" });
      let rest = "";
      const decoder = new TextDecoder();
      const errText = new Response(p.stderr).text();
      for await (const chunk of p.stdout as unknown as AsyncIterable<Uint8Array>) {
        rest += decoder.decode(chunk, { stream: true });
        let nl: number;
        while ((nl = rest.indexOf("\n")) >= 0) {
          const line = rest.slice(0, nl);
          rest = rest.slice(nl + 1);
          const msg = JSON.parse(line) as { found?: Found; result?: WorkerResult };
          if (msg.found) keep(msg.found, w);
          if (msg.result) results.push(msg.result);
        }
      }
      const code = await p.exited;
      if (code !== 0) fail(`worker ${w} failed (exit ${code}): ${(await errText).trim()}`);
    }),
  );
}
const searchMs = performance.now() - t0;
rmSync(stopPath, { force: true });

// Shrink and verify each finding: replays from tick 0, then a fresh og run, then the web host.
const { engine } = await boot();
const rate = engine.rate;
const start = engine.world.snapshot();
if (!start.ok) fail(start.reason);
const noWeb = opt["--web"] === "off" ? "--web off" : await webModule();
if (opt["--web"] === "on" && noWeb) fail(`--web on: ${noWeb}`);
const ogBin = process.env.OG_EXPLORE_OG ?? "";
const findings: unknown[] = [];
const rank = (e: Event) => ({ error: 0, violated: 1, fired: 2 })[e.outcome];
const kindOf = (e: Event) => ({ error: "error", violated: "violation", fired: "reached" })[e.outcome];
let index = 0;
for (const f of [...bySite.values()].sort((a, b) => (a.event.outcome === b.event.outcome ? a.event.tick - b.event.tick : rank(a.event) - rank(b.event)))) {
  index++;
  const id = `${kindOf(f.event)}-${index}`;
  const before = new Set(f.steps.map((s) => s.unit)).size;
  const s0 = performance.now();
  const small = flags.has("--no-shrink") ? { steps: f.steps, tick: f.event.tick, tests: 0 } : shrink(engine, start.value, f, f.event.tick);
  const end = small.tick + 1;
  const first = replay(engine, start.value, small.steps, end);
  const second = replay(engine, start.value, small.steps, end);
  const event = first.events.find((e) => siteOf(e) === f.site) ?? f.event;
  const commands = script(small.steps, rate, end);
  const reproFile = join(out, `${id}.repro.json`);
  writeFileSync(reproFile, JSON.stringify({ steps: small.steps, end, commands }) + "\n");
  const hashes: Record<string, string | null> = { explore: first.hash, again: second.hash };
  const notes: string[] = [];
  // og run in a fresh process: the script as a playtest plays it.
  if (ogBin) {
    const r = await child([ogBin, "run", ...(opt["--project"] ? [opt["--project"]] : []), "--headless", "--no-check", "--commands", commands, "--json", "--out", join(out, `${id}.og-run`)], { ...process.env, OG_RENDERER: "none" });
    const summary = (() => {
      try {
        return JSON.parse(r.out.trim().split("\n").pop() ?? "") as { hash?: string };
      } catch {
        return null;
      }
    })();
    hashes.ogRun = summary?.hash ?? null;
    if (!summary?.hash) notes.push(`og run gave no summary (exit ${r.code}): ${r.err.trim().slice(0, 400)}`);
  }
  if (!noWeb) {
    const r = await child(["bun", fileURLToPath(import.meta.url), "--entry", entry, ...(root ? ["--root", root] : []), "--replay", reproFile, "--host", "web"]);
    try {
      const w = JSON.parse(r.out.trim().split("\n").pop() ?? "") as { hash: string; events: Event[] };
      hashes.web = w.hash;
      if (!w.events.some((e) => siteOf(e) === f.site)) notes.push("the web replay did not reach the finding");
    } catch {
      hashes.web = null;
      notes.push(`the web replay failed (exit ${r.code}): ${r.err.trim().slice(0, 400)}`);
    }
  } else notes.push(`not replayed on the web host: ${noWeb}`);
  const values = Object.values(hashes).filter((h): h is string => h !== null);
  const deterministic = values.length > 0 && values.every((h) => h === values[0]);
  const finding = {
    id,
    kind: kindOf(f.event),
    site: { kind: event.kind, origin: event.origin, name: event.name, location: event.location, ...(event.key !== undefined ? { key: event.key } : {}) },
    tick: event.tick,
    ...(f.event.outcome === "error" ? { message: event.message ?? "" } : {}),
    ...(f.event.outcome === "violated" ? { entity: event.entity ?? null, component: event.component ?? null, field: event.field ?? null, value: event.value ?? null, message: event.message ?? "" } : {}),
    details: event.details ?? null,
    found: { worker: f.worker, afterTicks: f.atTicks, afterMs: Math.round(f.atMs), tick: f.event.tick },
    shrunk: { presses: { from: before, to: new Set(small.steps.map((s) => s.unit)).size }, steps: small.steps.length, replays: small.tests, ms: Math.round(performance.now() - s0) },
    repro: { commands, ticks: end, hash: first.hash, file: relative(process.cwd(), reproFile) },
    replays: hashes,
    deterministic,
    notes,
    playtest: relative(process.cwd(), join(out, `${id}.og.ts`)),
  };
  writeFileSync(join(out, `${id}.og.ts`), playtestFile(`${id}.og.ts`, { kind: finding.kind, script: commands, tick: finding.tick, hash: first.hash, site: finding.site }));
  findings.push(finding);
  log(`${id}: ${finding.site.kind} '${finding.site.name}' at tick ${finding.tick}, ${finding.shrunk.presses.from} → ${finding.shrunk.presses.to} presses; hashes ${deterministic ? "equal" : "DIFFER"} (${Object.entries(hashes).map(([k, v]) => `${k} ${v}`).join(", ")})`);
}

// Coverage: what every worker fired, and the sites none did.
const fired = new Set(results.flatMap((r) => r.fired));
type Site = { kind: string; origin: string; name: string; location: string; ran: boolean; fired: boolean; buckets?: { key: string; fired: boolean }[] };
const sites = new Map<string, Site>();
for (const r of results)
  for (const s of r.sitesSeen as Site[]) {
    const k = `${s.kind}\u0000${s.name}\u0000${s.location}`;
    const o = sites.get(k);
    if (!o) sites.set(k, { ...s, ...(s.buckets ? { buckets: s.buckets.map((b) => ({ ...b })) } : {}) });
    else {
      o.ran ||= s.ran;
      o.fired ||= s.fired;
      for (const b of s.buckets ?? []) {
        const ob = o.buckets?.find((x) => x.key === b.key);
        if (ob) ob.fired ||= b.fired;
        else (o.buckets ??= []).push({ ...b });
      }
    }
  }
const covers = (k: string) => ["sometimes", "reachable", "each"].includes(k);
const uncovered = [...sites.values()].filter((s) => covers(s.kind) && !s.fired).map((s) => ({ kind: s.kind, name: s.name, location: s.location, ran: s.ran }));
const sum = (k: keyof WorkerResult) => results.reduce((a, r) => a + (r[k] as number), 0);
const report = {
  kind: "og-explore",
  schema: 1,
  engine: (JSON.parse(readFileSync(join(pkg, "package.json"), "utf8")) as { version: string }).version,
  entry: opt["--project-entry"] ?? entry,
  seed,
  budget: { ticks: ticksBudget, seconds: msBudget / 1000, memoryMb: bytesBudget / 1048576, jobs, horizon },
  used: {
    ticks: sum("ticks"),
    searchMs: Math.round(searchMs),
    totalMs: Math.round(performance.now() - t0),
    iterations: sum("iterations"),
    restores: sum("restores"),
    peakRssMb: Math.max(0, ...results.map((r) => r.peakRssMb)),
    archiveMb: +(sum("archiveBytes") / 1048576).toFixed(2),
    stopped: [...new Set(results.map((r) => r.stopped))],
  },
  inputs: results[0]?.inputs ?? [],
  cells: { count: sum("cells"), fields: results[0]?.cellFields ?? 0, features: Math.max(0, ...results.map((r) => r.features)), evicted: sum("evicted") },
  coverage: {
    sites: sites.size,
    fired: [...sites.values()].filter((s) => s.fired || (s.kind === "always" && s.ran)).length,
    firings: fired.size,
    uncovered,
  },
  findings,
  workers: results.map((r) => ({ worker: r.worker, ticks: r.ticks, iterations: r.iterations, cells: r.cells, ms: r.ms, stopped: r.stopped })),
};
writeFileSync(join(out, "report.json"), JSON.stringify(report, null, 2) + "\n");
if (asJson) process.stdout.write(JSON.stringify(report) + "\n");
else {
  log(`explored ${report.used.ticks} ticks in ${report.used.searchMs} ms (${report.used.iterations} returns to ${report.cells.count} cells over ${report.cells.fields} fields; inputs: ${report.inputs.join(", ")})`);
  const count = (k: string) => findings.filter((f) => (f as { kind: string }).kind === k).length;
  log(`${count("error")} error(s), ${count("violation")} violation(s), ${count("reached")} site(s) reached; never fired: ${uncovered.map((u) => `${u.kind} '${u.name}'`).join(", ") || "none"}`);
  log(`report: ${join(out, "report.json")}`);
}
process.exit(0);
