// Runs one playtest (`tests/*.og.ts`) for `og test`: the script's commands
// through `og run --headless --commands` in a fresh process, then its checks
// over the files the run wrote. A playtest that made no check fails.
//
//   bun src/node/playtest-run.ts --file <tests/x.og.ts> --entry <game bundle> [--root <project>] [--out <dir>] [--json]

import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bundleGame } from "./build.ts";
import type { PlaytestChecks, PlaytestDesc, PlaytestRun } from "../playtest.ts";

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
let result: Record<string, unknown> = {};

function finish(): never {
  const ok = failures.length === 0 && checks > 0;
  if (checks === 0 && failures.length === 0) failures.push("the playtest made no check: call t.ok, t.equal or t.near at least once");
  const line = { playtest: name, file, ok, checks, failures, out, ms: +(performance.now() - t0).toFixed(0), ...result };
  if (asJson) process.stdout.write(JSON.stringify(line) + "\n");
  else {
    process.stdout.write(`${ok ? "PASS" : "FAIL"} ${name} (${checks} checks, ${line.ms} ms)${(line as { title?: string }).title ? ": " + (line as { title?: string }).title : ""}\n`);
    for (const f of failures) process.stdout.write(`  - ${f}\n`);
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

const cmd = ["bun", join(here, "run.ts"), "--entry", entry!, "--commands", desc!.commands, "--out", out, "--json"];
if (root) cmd.push("--root", root);
if (desc!.seed !== undefined) cmd.push("--seed", String(desc!.seed));
if (desc!.size) cmd.push("--size", desc!.size);
const r = Bun.spawnSync(cmd, { stdout: "pipe", stderr: "pipe" });
const stdout = new TextDecoder().decode(r.stdout);
const stderr = new TextDecoder().decode(r.stderr);
let summary: { tick: number; hash: string; entities: number; quit: boolean; tier: string; outputs: string[]; warnings: string[] } | null = null;
try {
  summary = JSON.parse(stdout.trim().split("\n").pop() ?? "");
} catch {
  summary = null;
}
if (r.exitCode !== 0 || !summary) {
  failures.push(`og run failed (exit ${r.exitCode}): ${stderr.trim() || stdout.trim()}`);
  finish();
}
result = { ...result, tick: summary!.tick, hash: summary!.hash, tier: summary!.tier, warnings: summary!.warnings };
// More runs of the same script (desc.runs), each in a fresh process into its
// own directory: only their end hashes are kept.
const hashes = [summary!.hash];
for (let i = 1; i < Math.max(1, Math.floor(desc!.runs ?? 1)); i++) {
  const again = cmd.map((a, k) => (cmd[k - 1] === "--out" ? join(out, `.run-${i + 1}`) : a));
  const r2 = Bun.spawnSync(again, { stdout: "pipe", stderr: "pipe" });
  const s2 = new TextDecoder().decode(r2.stdout).trim().split("\n").pop() ?? "";
  try {
    hashes.push((JSON.parse(s2) as { hash: string }).hash);
  } catch {
    failures.push(`run ${i + 1} of ${desc!.runs} failed (exit ${r2.exitCode}): ${new TextDecoder().decode(r2.stderr).trim()}`);
  }
}
result = { ...result, hashes };

const path = (f: string) => {
  const p = resolve(out, f);
  if (!existsSync(p)) throw new Error(`the run wrote no ${f} (it wrote ${summary!.outputs.map((o) => o.slice(out.length + 1)).join(", ") || "nothing"})`);
  return p;
};
const run: PlaytestRun = {
  tick: summary!.tick,
  hash: summary!.hash,
  hashes,
  entities: summary!.entities,
  quit: summary!.quit,
  tier: summary!.tier,
  files: summary!.outputs.map((o) => o.slice(out.length + 1)),
  warnings: summary!.warnings,
  json: <T>(f: string) => JSON.parse(readFileSync(path(f), "utf8")) as T,
  text: (f: string) => readFileSync(path(f), "utf8"),
  png: (f: string) => {
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
};
try {
  await desc!.check(run, t);
} catch (e) {
  failures.push(`check threw: ${e instanceof Error ? e.message : String(e)}`);
}
finish();
