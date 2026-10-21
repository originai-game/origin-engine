// Runs one playtest (`tests/*.og.ts`) for `og test`: the script's commands
// through `og run --headless --commands` in a fresh process (its `runs` and
// `then` steps in more), then its checks over the files the runs wrote. A
// playtest that made no check fails.
//
// Without a renderer (tier `none`: the platform's engine and probe
// sandboxes, OG_RENDERER=none) every check runs as before except those
// inside `t.frame`, and each screenshot the runs took is reported
// "no renderer, frame unchecked": one line per frame and a count, never a
// failure. `--result <file>` also writes the result line there (og test
// sums the unchecked frames from it).
//
//   bun src/node/playtest-run.ts --file <tests/x.og.ts> --entry <game bundle> [--root <project>] [--out <dir>] [--json] [--result <file>]

import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bundleGame } from "./build.ts";
import { FrameUnchecked, PLAYTEST_CLOCK, type PlaytestChecks, type PlaytestDesc, type PlaytestRun } from "../playtest.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
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
// Checks made inside t.frame (with a renderer): what a run without one leaves out.
let frameChecks = 0;
// Every screenshot taken without a renderer, by the file it would have written, in order.
const unchecked: string[] = [];
// Notes on unchecked frames beyond the per-file lines (a check stopped by one).
const notes: string[] = [];
let result: Record<string, unknown> = {};

function finish(): never {
  // A playtest whose every check reads a frame has nothing to check without a renderer: it passes, and says so.
  const ok = failures.length === 0 && (checks > 0 || unchecked.length > 0);
  if (checks === 0 && failures.length === 0) {
    if (unchecked.length > 0) notes.push("every check of this playtest reads a frame: nothing was checked without a renderer");
    else failures.push("the playtest made no check: call t.ok, t.equal or t.near at least once");
  }
  const line = { playtest: name, file, ok, checks, frameChecks, unchecked, failures, notes, out, ms: +(performance.now() - t0).toFixed(0), ...result };
  if (resultFile) {
    mkdirSync(dirname(resolve(resultFile)), { recursive: true });
    writeFileSync(resultFile, JSON.stringify(line) + "\n");
  }
  if (asJson) process.stdout.write(JSON.stringify(line) + "\n");
  else {
    const frames = unchecked.length > 0 ? `, ${unchecked.length} frame${unchecked.length === 1 ? "" : "s"} unchecked` : "";
    process.stdout.write(`${ok ? "PASS" : "FAIL"} ${name} (${checks} checks${frames}, ${line.ms} ms)${(line as { title?: string }).title ? ": " + (line as { title?: string }).title : ""}\n`);
    for (const f of failures) process.stdout.write(`  - ${f}\n`);
    for (const u of unchecked) process.stdout.write(`  ~ ${u}: no renderer, frame unchecked\n`);
    for (const n of notes) process.stdout.write(`  ~ ${n}\n`);
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
if (!desc || typeof desc.check !== "function") {
  failures.push(`${file} has no default export: export default playtest({ name, commands, check })`);
  finish();
}
result.title = desc!.name;

// Every process of the playtest: the script, its `runs` and its `then` steps.
// The host's clock is pinned (PlaytestDesc.clock), so og.platform.now is
// reproducible; `store: "keep"` shares one local store file between them,
// kept under a dot name, which no step's output may take (an output named
// `platform.json` once replaced it: hollowbell FINDINGS #18).
const store = desc!.store === "keep" ? join(out, ".store", "platform.json") : undefined;
if (store) mkdirSync(dirname(store), { recursive: true });
const command = (commands: string, dir: string, clock: number) => {
  const c = ["bun", join(here, "run.ts"), "--entry", entry!, "--commands", commands, "--out", dir, "--json", "--clock", String(clock)];
  if (root) c.push("--root", root);
  if (desc!.seed !== undefined) c.push("--seed", String(desc!.seed));
  if (desc!.size) c.push("--size", desc!.size);
  if (store) c.push("--platform-store", store);
  return c;
};
type Summary = { tick: number; hash: string; entities: number; quit: boolean; tier: string; outputs: string[]; warnings: string[]; unchecked?: string[] };
const spawn = (c: string[], what: string): Summary | null => {
  const r = Bun.spawnSync(c, { stdout: "pipe", stderr: "pipe" });
  const stdout = new TextDecoder().decode(r.stdout);
  const stderr = new TextDecoder().decode(r.stderr);
  let summary: Summary | null = null;
  try {
    summary = JSON.parse(stdout.trim().split("\n").pop() ?? "");
  } catch {
    summary = null;
  }
  if (r.exitCode !== 0 || !summary) {
    failures.push(`${what} failed (exit ${r.exitCode}): ${stderr.trim() || stdout.trim()}`);
    return null;
  }
  return summary;
};
const clock = desc!.clock ?? PLAYTEST_CLOCK;
const summary = spawn(command(desc!.commands, out, clock), "og run");
if (!summary) finish();
const rendered = summary!.tier !== "none";
unchecked.push(...(summary!.unchecked ?? []));
result = { ...result, tick: summary!.tick, hash: summary!.hash, tier: summary!.tier, warnings: summary!.warnings };
// More runs of the same script (desc.runs), each in a fresh process into its
// own directory: only their end hashes are kept.
const hashes = [summary!.hash];
for (let i = 1; i < Math.max(1, Math.floor(desc!.runs ?? 1)); i++) {
  const again = spawn(command(desc!.commands, join(out, `.run-${i + 1}`), clock), `run ${i + 1} of ${desc!.runs}`);
  if (again) hashes.push(again.hash);
}
result = { ...result, hashes };

const runOf = (s: Summary, dir: string, later: readonly PlaytestRun[]): PlaytestRun => {
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
    json: <T>(f: string) => JSON.parse(readFileSync(path(f), "utf8")) as T,
    text: (f: string) => readFileSync(path(f), "utf8"),
    png: (f: string) => {
      if (missed.has(f)) throw new FrameUnchecked(f);
      const bytes = readFileSync(path(f));
      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      if (bytes.length < 24 || view.getUint32(12) !== 0x49484452) throw new Error(`${f} is not a PNG`);
      return { width: view.getUint32(16), height: view.getUint32(20), bytes: bytes.length };
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
// The later processes (desc.then), in order, after the script and its runs.
const later: PlaytestRun[] = [];
for (const [i, step] of (desc!.then ?? []).entries()) {
  const dir = join(out, `then-${i + 1}`);
  mkdirSync(dir, { recursive: true });
  const s = spawn(command(step.commands, dir, step.clock ?? clock), `then[${i}]`);
  if (!s) finish();
  for (const f of s!.unchecked ?? []) unchecked.push(`then[${i}] ${f}`);
  later.push(runOf(s!, dir, []));
}
const run = runOf(summary!, out, later);
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
  if (e instanceof FrameUnchecked) return e.file;
  const err = e as { code?: string; path?: string } | null;
  if (err && err.code === "ENOENT" && typeof err.path === "string" && missedPaths.has(resolve(err.path))) return err.path.slice(out.length + 1);
  return null;
}
try {
  await desc!.check(run, t);
} catch (e) {
  const frame = missedFrame(e);
  if (frame !== null) notes.push(`the check read ${frame}, a frame taken without a renderer, and stopped there: the checks after it did not run (put frame checks in t.frame so every other check runs)`);
  else failures.push(`check threw: ${e instanceof Error ? e.message : String(e)}`);
}
finish();
