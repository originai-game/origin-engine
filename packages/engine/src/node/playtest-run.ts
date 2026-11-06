// Runs one playtest (`tests/*.og.ts`) for `og test`: the script's commands
// through `og run --headless --commands` in a fresh process (its `runs` and
// `then` steps in more), then its checks over the files the runs wrote. A
// playtest that made no check fails.
//
// Simulation-only playtests can run without a renderer. Requested frames
// without a GPU are reported unchecked, never counted as passed.
//
//   bun src/node/playtest-run.ts --file <tests/x.og.ts> --entry <game bundle> [--root <project>] [--out <dir>] [--json] [--result <file>]

import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bundleGame } from "./build.ts";
import { FrameUnchecked, PLAYTEST_CLOCK, type PlaytestAssertions, type PlaytestChecks, type PlaytestDesc, type PlaytestLockstep, type PlaytestRun } from "../playtest.ts";
import { loadNativeHost, native } from "./index.ts";
import { decodePng } from "./png.ts";
import { runInSession } from "./session-client.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(cmd: string[], o?: { stdout?: string; stderr?: string }): { exited: Promise<number>; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
};

const argv = process.argv.slice(2);
const opt = (name: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1] : undefined;
};
const file = opt("--file");
const entry = opt("--entry");
const root = opt("--root");
const resultFile = opt("--result");
const asJson = argv.includes("--json");
if (!file || !entry) {
  process.stderr.write("usage: bun src/node/playtest-run.ts --file <tests/x.og.ts> --entry <game bundle> [--root <project>] [--out <dir>] [--json]\n");
  process.exit(2);
}
const here = dirname(fileURLToPath(import.meta.url));
const name = basename(file!).replace(/\.og\.ts$/, "");
const out = resolve(opt("--out") ?? join(root ?? dirname(file!), "og-out", "test", name));
rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

const t0 = performance.now();
const failures: string[] = [];
let checks = 0;
// Checks made inside t.frame and region or frame expectations (with a renderer): what a run without one leaves out.
let frameChecks = 0;
// Every screenshot taken without a renderer, by the file it would have written, in order.
const unchecked: string[] = [];
const adapterErrors = new Set<string>();
// Notes on unchecked frames beyond the per-file lines (a check stopped by one).
const notes: string[] = [];
let result: Record<string, unknown> = {};

function finish(): never {
  if (checks === 0 && unchecked.length === 0 && failures.length === 0) {
    failures.push("the playtest made no check: call t.ok, t.equal or t.near at least once");
  }
  const ok = failures.length === 0;
  const status = !ok ? "failed" : unchecked.length > 0 ? "unchecked" : "passed";
  const line = { playtest: name, file, ok, status, checks, frameChecks, unchecked, uncheckedCount: unchecked.length, adapterErrors: [...adapterErrors], failures, notes, out, ms: +(performance.now() - t0).toFixed(0), assertions: assertionLog, ...result };
  if (resultFile) {
    mkdirSync(dirname(resolve(resultFile)), { recursive: true });
    writeFileSync(resultFile, JSON.stringify(line) + "\n");
  }
  if (asJson) process.stdout.write(JSON.stringify(line) + "\n");
  else {
    const frames = `, ${unchecked.length} GPU frame/expectation checks unchecked`;
    process.stdout.write(`${!ok ? "FAIL" : unchecked.length > 0 ? "UNCHECKED" : "PASS"} ${name} (${checks} checks${frames}, ${line.ms} ms)${(line as { title?: string }).title ? ": " + (line as { title?: string }).title : ""}\n`);
    for (const f of failures) process.stdout.write(`  - ${f}\n`);
    for (const u of unchecked) process.stdout.write(`  ~ ${u}: no renderer, frame unchecked\n`);
    for (const n of notes) process.stdout.write(`  ~ ${n}\n`);
    for (const u of assertionLog.uncovered as Array<{ run: string; kind: string; name: string; key?: string }>) process.stdout.write(`  ? ${u.run}: ${u.kind} '${u.name}'${u.key !== undefined ? ` [${u.key}]` : ""} never fired\n`);
  }
  process.exit(ok ? 0 : 1);
}

// The playtest module, bundled like the game so `@origingame/engine/playtest` resolves.
const bundled = join(out, ".playtest.js");
const b = await bundleGame(file!, bundled, "native");
if (!b.ok) {
  failures.push(`the playtest does not bundle: ${b.reason}`);
  finish();
}
const desc = ((await import(pathToFileURL(bundled).href)) as { default?: PlaytestDesc }).default;
if (!desc || !desc.commands) {
  failures.push(`${file} has no default export: export default playtest({ name, commands, check })`);
  finish();
}
result.title = desc!.name;

// Every process of the playtest: the script, its `runs` and its `then` steps.
// The host's clock is pinned (PlaytestDesc.clock), so og.platform.now is
// reproducible; `store: "keep"` shares one local store file between them,
// kept under a dot name, which no step's output may take (an output named
// `platform.json` once replaced it: hollowbell FINDINGS #18). In a room each
// place has its own (`platform-<place>.json`): its processes run at once,
// and a `then` step relaunches every place on its own store.
const storeOf = (place: number): string | undefined => {
  if (desc!.store !== "keep") return undefined;
  const path = join(out, ".store", desc!.room && place > 0 ? `platform-${place}.json` : "platform.json");
  mkdirSync(dirname(path), { recursive: true });
  return path;
};
// PlaytestDesc.platform: the services' fake, one file every process reads (a dot name, as the store's).
const fakeFile = desc!.platform
  ? (() => {
      const path = join(out, ".store", "fake.json");
      mkdirSync(dirname(path), { recursive: true });
      writeFileSync(path, JSON.stringify(desc!.platform));
      return path;
    })()
  : undefined;
// PlaytestDesc.invariants: `invariant:` steps at tick 0 of every process's script.
const invariantSteps = (desc!.invariants ?? []).map((i) => `0:invariant:${i}`);
const command = (commands: string, dir: string, clock: number, room?: { url: string; ticks?: number | undefined; place: number }) => {
  const script = [...invariantSteps, commands].join(";");
  const c = ["bun", join(here, "run.ts"), "--entry", entry!, "--commands", script, "--out", dir, "--json", "--clock", String(clock)];
  if (room) c.push("--room", room.url);
  if (room?.ticks !== undefined) c.push("--ticks", String(room.ticks));
  if (root) c.push("--root", root);
  if (desc!.seed !== undefined) c.push("--seed", String(desc!.seed));
  if (desc!.size) c.push("--size", desc!.size);
  for (const step of (desc!.setup ?? "").split(";").map((s) => s.trim()).filter((s) => s !== "")) c.push("--setup", step);
  const store = storeOf(room?.place ?? 0);
  if (store) c.push("--platform-store", store);
  if (fakeFile) c.push("--platform-fake", fakeFile);
  return c;
};
type ExpectResult = { tick: number; expect: string; kind: string; status: "pass" | "fail" | "unchecked"; expected: unknown; observed: unknown; reason: string };
type Summary = {
  tick: number;
  hash: string;
  netHash?: string;
  lockstep?: PlaytestLockstep;
  entities: number;
  quit: boolean;
  tier: string;
  renderer: string;
  adapterError: string | null;
  outputs: string[];
  warnings: string[];
  unchecked?: string[];
  expectations?: { failed: number; results: ExpectResult[] };
  assertions?: PlaytestAssertions;
};
const EMPTY_ASSERTIONS: PlaytestAssertions = { violations: [], uncovered: [], sites: [], defaults: { fields: 0, ticks: 0, values: 0, totalMs: 0, meanUs: 0, maxUs: 0 } };
// Every process's violations and uncovered sites, for the result line (the session carries it per run).
const assertionLog: { violations: unknown[]; uncovered: unknown[]; required: Array<{ name: string; fired: boolean }>; defaults: unknown[] } = { violations: [], uncovered: [], required: [], defaults: [] };
const spawn = async (c: string[], what: string): Promise<Summary | null> => {
  try {
    const resident = await runInSession(c.slice(2), opt("--session"), opt("--session-build"));
    if (resident) return summarise(resident.exitCode, resident.stdout, resident.stderr, what);
  } catch (error) {
    failures.push(`${what}: ${error instanceof Error ? error.message : String(error)}`);
    return null;
  }
  const r = Bun.spawnSync(c, { stdout: "pipe", stderr: "pipe" });
  return summarise(r.exitCode, new TextDecoder().decode(r.stdout), new TextDecoder().decode(r.stderr), what);
};
function summarise(exitCode: number, stdout: string, stderr: string, what: string): Summary | null {
  let summary: Summary | null = null;
  try {
    summary = JSON.parse(stdout.trim().split("\n").pop() ?? "");
  } catch {
    summary = null;
  }
  // A run whose only failures are its expectations or assertions exits 1 with its summary: each is a check.
  const expectedOnly = exitCode === 1 && summary !== null && ((summary.expectations?.failed ?? 0) > 0 || (summary.assertions?.violations.length ?? 0) > 0);
  if ((exitCode !== 0 && !expectedOnly) || !summary) {
    failures.push(`${what} failed (exit ${exitCode}): ${stderr.trim() || stdout.trim()}`);
    return null;
  }
  if (summary.adapterError && !adapterErrors.has(summary.adapterError)) {
    adapterErrors.add(summary.adapterError);
    const expected = process.env.OG_RENDERER?.trim().toLowerCase() === "none";
    process.stderr.write(`WARNING ${expected ? "" : "unexpected "}missing GPU: ${summary.adapterError}; frame checks are unchecked, not passed\n`);
  }
  for (const e of summary.expectations?.results ?? []) {
    if (e.status === "unchecked") unchecked.push(`${what}: expect:${e.expect}`);
    else {
      checks++;
      // A region or frame expectation read the frame: without a renderer it is unchecked.
      if (e.kind === "region" || e.kind === "frame") frameChecks++;
      if (e.status === "fail") failures.push(`${what}: expect:${e.expect} failed at tick ${e.tick}: expected ${JSON.stringify(e.expected)}, observed ${JSON.stringify(e.observed)}; ${e.reason}`);
    }
  }
  const a = summary.assertions ?? EMPTY_ASSERTIONS;
  // Every invariant site (the playtest's, a failed default, the game's always) is one check per process.
  checks += a.sites.filter((s) => s.kind === "always" || s.kind === "unreachable").length;
  for (const v of a.violations) {
    const where = [v.entity && `on ${v.entity}`, v.component && v.field && `${v.component}.${v.field}`].filter(Boolean).join(" ");
    failures.push(`${what}: ${v.origin} ${v.kind} '${v.name}'${v.location && v.origin === "game" ? ` (${v.location})` : ""} failed at tick ${v.tick}${where ? ` ${where}` : ""}: ${v.message}; observed ${JSON.stringify(v.value)}${v.expected !== null && v.expected !== undefined ? `, expected ${JSON.stringify(v.expected)}` : ""}${v.details !== null && v.details !== undefined ? `, details ${JSON.stringify(v.details)}` : ""}${v.failures > 1 ? ` (${v.failures} failing calls)` : ""}`);
  }
  assertionLog.violations.push(...a.violations.map((v) => ({ run: what, ...v })));
  assertionLog.uncovered.push(...a.uncovered.map((u) => ({ run: what, ...u })));
  assertionLog.defaults.push({ run: what, ...a.defaults });
  return summary;
}

/** PlaytestDesc.require over a run's report: each named site (or `name[key]` bucket) fired. */
function requireFired(a: PlaytestAssertions, what: string): void {
  for (const want of desc!.require ?? []) {
    checks++;
    const m = /^(.*)\[(.*)\]$/.exec(want);
    const name = m ? m[1]! : want;
    const sites = a.sites.filter((s) => s.name === name && s.kind !== "always" && s.kind !== "unreachable");
    let fired: boolean;
    let why = "";
    if (sites.length === 0) {
      fired = false;
      why = `no og.assert.sometimes, reachable or each named '${name}' ran or was declared: check the name`;
    } else if (m) {
      fired = sites.some((s) => s.buckets?.some((b) => b.key === m[2] && b.fired));
      if (!fired) why = `bucket [${m[2]}] never held${sites.some((s) => s.buckets?.some((b) => b.key === m[2])) ? "" : " (never seen)"}`;
    } else {
      fired = sites.some((s) => s.fired && (s.kind !== "each" || (s.buckets ?? []).every((b) => b.fired)));
      if (!fired) {
        const s = sites[0]!;
        const open = (s.buckets ?? []).filter((b) => !b.fired).map((b) => b.key);
        why = s.kind === "each" && open.length > 0 ? `buckets never held: ${open.join(", ")}` : s.ran ? "it ran but never held" : "the run never got there";
      }
    }
    assertionLog.required.push({ name: want, fired });
    if (!fired) failures.push(`${what}: required ${sites[0]?.kind ?? "assertion"} '${want}' never fired: ${why}`);
  }
}
const clock = desc!.clock ?? PLAYTEST_CLOCK;

// PlaytestDesc.room: a relay in this process, and every peer's process at
// once, each joined to the room (the first is the run, the others its peers).
// A `then` step relaunches every place at once into a room of its own
// (`<name>.then-<n>`), each running the step's script: lockstep keeps the
// steps on the room's shared ticks, so a `quit` ends every place on one tick.
const room = desc!.room;
const peerSummaries: Summary[] = [];
let summary: Summary | null;
let playRoom: ((name: string, scripts: (place: number) => string, dirOf: (place: number) => string, clock: number, what: string) => Promise<Summary[]>) | null = null;
if (room) {
  const players = Math.floor(room.players);
  if (!(players >= 2 && players <= 16)) failures.push(`room.players is ${room.players}: a room playtest takes 2 to 16 processes`);
  if ((desc!.runs ?? 1) > 1) failures.push("a room playtest takes no `runs`: its processes play at once (a relaunch is a `then` step)");
  const name = room.name ?? "playtest";
  if (room.ticks !== undefined && !(Number.isInteger(room.ticks) && room.ticks > 0)) failures.push(`room.ticks is ${room.ticks}: the tick every process stops at, a whole number above 0 (or leave it out and end the script with quit)`);
  if (!/^[A-Za-z0-9._-]{1,54}$/.test(name)) failures.push(`room.name '${name}' is not a room id: 1 to 54 of letters, digits, '.', '_' and '-' (a then step's room adds .then-<n>)`);
  if (failures.length > 0) finish();
  loadNativeHost();
  const relay = JSON.parse(native().og_node_relay_start(0)) as { ok: boolean; value?: number; reason?: string };
  if (!relay.ok) {
    failures.push(`the playtest's relay did not start: ${relay.reason}`);
    finish();
  }
  playRoom = async (roomName, scripts, dirOf, at, what) => {
    const url = `ws://127.0.0.1:${relay.value}/${roomName}`;
    const results = await Promise.all(
      Array.from({ length: players }, async (_, i) => {
        const dir = dirOf(i);
        mkdirSync(dir, { recursive: true });
        const p = Bun.spawn(command(scripts(i), dir, at, { url, ticks: room.ticks, place: i }), { stdout: "pipe", stderr: "pipe" });
        const [code, stdout, stderr] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);
        return summarise(code, stdout, stderr, `${what} (peer ${i})`);
      }),
    );
    if (results.some((r) => r === null)) finish();
    return results as Summary[];
  };
  const results = await playRoom(name, (i) => room.commands?.[i] ?? desc!.commands, (i) => (i === 0 ? out : join(out, `peer-${i}`)), clock, "og run");
  summary = results[0]!;
  peerSummaries.push(...results.slice(1));
  for (const [i, s] of peerSummaries.entries()) for (const f of s.unchecked ?? []) unchecked.push(`peer ${i + 1} ${f}`);
} else summary = await spawn(command(desc!.commands, out, clock), "og run");
if (!summary) finish();
requireFired(summary!.assertions ?? EMPTY_ASSERTIONS, "og run");
const rendered = summary!.tier !== "none";
unchecked.push(...(summary!.unchecked ?? []));
result = { ...result, tick: summary!.tick, hash: summary!.hash, tier: summary!.tier, renderer: summary!.renderer, adapterError: summary!.adapterError, warnings: summary!.warnings };
// A room's shared hashes, the run's first: equal ones are peers that stayed in step.
if (room) result = { ...result, netHashes: [summary!, ...peerSummaries].map((s) => s.netHash ?? ""), lockstep: [summary!, ...peerSummaries].map((s) => s.lockstep ?? null) };
// More runs of the same script (desc.runs), each in a fresh process into its
// own directory: only their end hashes are kept.
const hashes = [summary!.hash];
for (let i = 1; i < Math.max(1, Math.floor(desc!.runs ?? 1)); i++) {
  const again = await spawn(command(desc!.commands, join(out, `.run-${i + 1}`), clock), `run ${i + 1} of ${desc!.runs}`);
  if (again) hashes.push(again.hash);
}
result = { ...result, hashes };

const runOf = (s: Summary, dir: string, later: readonly PlaytestRun[], peers: readonly PlaytestRun[] = []): PlaytestRun => {
  const path = (f: string) => {
    const p = resolve(dir, f);
    if (!existsSync(p)) throw new Error(`the run wrote no ${f} (it wrote ${s.outputs.map((o) => o.slice(dir.length + 1)).join(", ") || "nothing"})`);
    return p;
  };
  const missed = new Set(s.unchecked ?? []);
  return {
    tick: s.tick,
    hash: s.hash,
    hashes: dir === out ? hashes : [s.hash],
    entities: s.entities,
    quit: s.quit,
    tier: s.tier,
    rendered: s.tier !== "none",
    unchecked: [...missed],
    files: s.outputs.map((o) => o.slice(dir.length + 1)),
    warnings: s.warnings,
    then: later,
    assertions: s.assertions ?? EMPTY_ASSERTIONS,
    netHash: s.netHash ?? "",
    lockstep: s.lockstep ?? null,
    peers,
    json: <T>(f: string) => JSON.parse(readFileSync(path(f), "utf8")) as T,
    text: (f: string) => readFileSync(path(f), "utf8"),
    png: (f: string) => {
      if (missed.has(f)) throw new FrameUnchecked(f);
      const bytes = readFileSync(path(f));
      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      if (bytes.length < 24 || view.getUint32(12) !== 0x49484452) throw new Error(`${f} is not a PNG`);
      return { width: view.getUint32(16), height: view.getUint32(20), bytes: bytes.length };
    },
    pixels: (f: string) => {
      if (missed.has(f)) throw new FrameUnchecked(f);
      return decodePng(new Uint8Array(readFileSync(path(f))));
    },
    wav: (f: string) => {
      const bytes = readFileSync(path(f));
      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      if (bytes.length < 44 || view.getUint32(0) !== 0x52494646 || view.getUint32(8) !== 0x57415645) throw new Error(`${f} is not a WAV`);
      const rate = view.getUint32(24, true);
      const channels = view.getUint16(22, true);
      const samples = (bytes.length - 44) >> 1;
      let peak = 0;
      let sum = 0;
      for (let i = 0; i < samples; i++) {
        const x = view.getInt16(44 + i * 2, true) / 32767;
        peak = Math.max(peak, Math.abs(x));
        sum += x * x;
      }
      return { rate, frames: samples / Math.max(1, channels), peak, rms: samples > 0 ? Math.sqrt(sum / samples) : 0 };
    },
  };
};
// The later processes (desc.then), in order, after the script and its runs;
// in a room every place again, at once (their runs are each step's `peers`).
const later: PlaytestRun[] = [];
for (const [i, step] of (desc!.then ?? []).entries()) {
  const dir = join(out, `then-${i + 1}`);
  mkdirSync(dir, { recursive: true });
  if (playRoom) {
    const peerDir = (p: number) => join(dir, `peer-${p}`);
    const all = await playRoom(`${room!.name ?? "playtest"}.then-${i + 1}`, () => step.commands, (p) => (p === 0 ? dir : peerDir(p)), step.clock ?? clock, `then[${i}]`);
    for (const [p, s] of all.entries()) for (const f of s.unchecked ?? []) unchecked.push(`then[${i}]${p > 0 ? ` peer ${p}` : ""} ${f}`);
    later.push(runOf(all[0]!, dir, [], all.slice(1).map((s, p) => runOf(s, peerDir(p + 1), []))));
    continue;
  }
  const s = await spawn(command(step.commands, dir, step.clock ?? clock), `then[${i}]`);
  if (!s) finish();
  for (const f of s!.unchecked ?? []) unchecked.push(`then[${i}] ${f}`);
  later.push(runOf(s!, dir, []));
}
const run = runOf(
  summary!,
  out,
  later,
  peerSummaries.map((s, i) => runOf(s, join(out, `peer-${i + 1}`), [])),
);
// Every file a run took a screenshot for without a renderer, by name and by path.
const missedPaths = new Set<string>([...(summary!.unchecked ?? []).map((f) => resolve(out, f)), ...later.flatMap((r, i) => r.unchecked.map((f) => resolve(join(out, `then-${i + 1}`), f)))]);
const missedNames = new Set<string>([...(summary!.unchecked ?? []), ...later.flatMap((r) => r.unchecked)]);
let inFrame = false;
const t: PlaytestChecks = {
  ok(condition, what) {
    checks++;
    if (!condition) failures.push(what);
  },
  equal(actual, expected, what) {
    checks++;
    if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  },
  near(actual, expected, tolerance, what) {
    checks++;
    if (!(Math.abs(actual - expected) <= tolerance)) failures.push(`${what}: expected ${expected} ± ${tolerance}, got ${actual}`);
  },
  frame(files, check) {
    const names = typeof files === "string" ? [files] : [...files];
    if (!rendered) {
      // The frames are listed once each, from the runs' own notes; a name no
      // screenshot took is listed too, so nothing is skipped silently.
      for (const f of names) if (!missedNames.has(f) && !unchecked.includes(f)) unchecked.push(f);
      return;
    }
    if (inFrame) return check();
    const before = checks;
    inFrame = true;
    try {
      check();
    } finally {
      inFrame = false;
      frameChecks += checks - before;
    }
  },
};
/** The file a frame read without a renderer was after, if `e` is one: a FrameUnchecked, or a missing screenshot read from disk. */
function missedFrame(e: unknown): string | null {
  if (rendered) return null;
  // By name too: a playtest's bundle carries its own copy of the class.
  if (e instanceof FrameUnchecked || (e instanceof Error && e.name === "FrameUnchecked")) return (e as FrameUnchecked).file;
  const err = e as { code?: string; path?: string } | null;
  if (err && err.code === "ENOENT" && typeof err.path === "string" && missedPaths.has(resolve(err.path))) return err.path.slice(out.length + 1);
  return null;
}
try {
  if (desc!.check) await desc!.check(run, t);
} catch (e) {
  const frame = missedFrame(e);
  if (frame !== null) notes.push(`the check read ${frame}, a frame taken without a renderer, and stopped there: the checks after it did not run (put frame checks in t.frame so every other check runs)`);
  else failures.push(`check threw: ${e instanceof Error ? e.message : String(e)}`);
}
finish();
