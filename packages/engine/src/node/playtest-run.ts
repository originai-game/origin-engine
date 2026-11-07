// Runs one playtest file (`tests/*.og.ts`, a bun:test file) for `og test`:
// `bun test` on that file (playtest-files.ts), its runPlaytest calls playing
// the game bundle `og test` built (run-playtest.ts), then one result line
// over every test of the file: the tests and their failures, the checks,
// the frames left unchecked, the missing adapter, and the evidence of a
// failed run. A test that made no assertion fails (the preload).
//
// Simulation-only playtests can run without a renderer. Requested frames
// without a GPU are reported unchecked, never counted as passed.
//
//   bun src/node/playtest-run.ts --file <tests/x.og.ts> --entry <game bundle> [--root <project>] [--out <dir>] [--json] [--result <file>]

import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { bunTest } from "./playtest-files.ts";
import { encodePng } from "./png.ts";

declare const Bun: {
  spawn(cmd: string[], o: { cwd?: string; env?: Record<string, string | undefined>; stdin?: "ignore"; stdout?: "pipe"; stderr?: "pipe" }): { exited: Promise<number>; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
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
const name = basename(file!).replace(/\.og\.ts$/, "");
const out = resolve(opt("--out") ?? join(root ?? dirname(file!), "og-out", "test", name));
rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });
const t0 = performance.now();

const results = join(out, ".results.jsonl");
const junit = join(out, ".junit.xml");
writeFileSync(results, "");
const p = Bun.spawn(bunTest(file!, out, ["--reporter=junit", `--reporter-outfile=${junit}`]), {
  cwd: root ?? dirname(resolve(file!)),
  env: {
    ...process.env,
    OG_PLAYTEST_ENV: JSON.stringify(process.env),
    OG_PLAYTEST_ENTRY: resolve(entry!),
    OG_PLAYTEST_ROOT: root ? resolve(root) : "",
    OG_PLAYTEST_OUT: out,
    OG_PLAYTEST_RESULT: results,
    OG_PLAYTEST_SESSION_SOCKET: opt("--session-socket") ?? "",
    OG_PLAYTEST_SESSION_BUILD: opt("--session-build") ?? "",
  },
  stdin: "ignore",
  stdout: "pipe",
  stderr: "pipe",
});
const [code, stdout, stderr] = await Promise.all([p.exited, new Response(p.stdout).text(), new Response(p.stderr).text()]);

// The tests, from the junit report: each case and its failure.
const xml = existsSync(junit) ? readFileSync(junit, "utf8") : "";
const unescape = (s: string) => s.replace(/&#(\d+);/g, (_, n) => String.fromCharCode(Number(n))).replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&apos;/g, "'").replace(/&amp;/g, "&");
const attr = (tag: string, key: string) => new RegExp(`\\s${key}="([^"]*)"`).exec(tag)?.[1];
const root0 = /<testsuites\b[^>]*>/.exec(xml)?.[0] ?? "";
const tests = Number(attr(root0, "tests") ?? 0);
const assertionCount = Number(attr(root0, "assertions") ?? 0);
const failures: string[] = [];
for (const m of xml.matchAll(/<testcase\b([^>]*?)(\/>|>([\s\S]*?)<\/testcase>)/g)) {
  const failure = /<failure\b[^>]*message="([^"]*)"/.exec(m[3] ?? "");
  if (failure) failures.push(`${unescape(attr(m[1]!, "name") ?? "")}: ${unescape(failure[1]!).trim()}`);
}
if (tests === 0) failures.push(code === 0 ? `${file} has no test: write describe/test from bun:test that awaits runPlaytest` : `${file} does not load: ${(stderr.trim() || stdout.trim()).slice(-2000)}`);
else if (code !== 0 && failures.length === 0) failures.push(`bun test exited ${code}: ${(stderr.trim() || stdout.trim()).slice(-2000)}`);
const ok = code === 0 && tests > 0 && failures.length === 0;

// What each runPlaytest call reported, and the frames run.frame left unchecked.
type RunLine = { kind: "run"; out: string; checks: number; frameChecks: number; unchecked: string[]; adapterErrors: string[]; captures: string[]; assertions: { violations: unknown[]; uncovered: unknown[]; required: unknown[]; defaults: unknown[] }; tick?: number; hash?: string; hashes?: string[]; tier?: string; renderer?: string; adapterError?: string | null; warnings?: string[]; netHashes?: string[]; lockstep?: unknown[]; failed?: boolean };
type Line = RunLine | { kind: "unchecked"; files: string[] };
const lines = readFileSync(results, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l) as Line);
const runs = lines.filter((l): l is RunLine => l.kind === "run");
const unchecked = lines.flatMap((l) => (l.kind === "run" ? l.unchecked : l.files));
const adapterErrors = [...new Set(runs.flatMap((r) => r.adapterErrors))];
const assertionLog = { violations: runs.flatMap((r) => r.assertions.violations), uncovered: runs.flatMap((r) => r.assertions.uncovered), required: runs.flatMap((r) => r.assertions.required), defaults: runs.flatMap((r) => r.assertions.defaults) };
const frameChecks = runs.reduce((n, r) => n + r.frameChecks, 0);

// The evidence of a failed file: each run's world, snapshot and last frame; a passed one keeps none.
// When runPlaytest itself rejected, its runs are the failure; otherwise any run may be.
const failedRuns = runs.filter((r) => r.failed);
const evidence = new Set((ok ? [] : failedRuns.length > 0 ? failedRuns : runs).flatMap((r) => r.captures));
const failureCaptures = [];
for (const dir of runs.flatMap((r) => r.captures)) {
  if (!evidence.has(dir)) {
    rmSync(dir, { recursive: true, force: true });
    continue;
  }
  let frame: { status: "captured"; path: string } | { status: "unavailable"; reason: string } = { status: "unavailable", reason: "no renderer or no completed frame" };
  const raw = join(dir, "frame.rgba");
  try {
    if (existsSync(raw)) {
      const bytes = readFileSync(raw);
      if (bytes.length > 0) {
        if (bytes.length < 8) throw new Error("truncated frame header");
        const header = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        const width = header.getUint32(0, true), height = header.getUint32(4, true);
        if (width === 0 || height === 0 || bytes.length !== 8 + width * height * 4) throw new Error("invalid frame dimensions");
        const path = join(dir, "frame.png");
        writeFileSync(path, encodePng({ width, height, rgba: bytes.subarray(8) }));
        frame = { status: "captured", path };
      }
    }
  } catch (error) {
    frame = { status: "unavailable", reason: error instanceof Error ? error.message : String(error) };
  } finally {
    rmSync(raw, { force: true });
  }
  if (!existsSync(dir)) continue;
  const world = join(dir, "world.json"), snapshot = join(dir, "world.ogsave");
  failureCaptures.push({ run: dirname(dir), world: existsSync(world) ? world : null, snapshot: existsSync(snapshot) ? snapshot : null, frame });
}

const first = runs[0];
const status = !ok ? "failed" : unchecked.length > 0 ? "unchecked" : "passed";
const line = {
  playtest: name,
  file,
  ok,
  status,
  tests,
  // Every expect() the file's tests made, runPlaytest's own among them.
  checks: assertionCount,
  frameChecks,
  unchecked,
  uncheckedCount: unchecked.length,
  adapterErrors,
  failures,
  failureCaptures,
  out,
  ms: +(performance.now() - t0).toFixed(0),
  assertions: assertionLog,
  runs: runs.length,
  ...(first ? { tick: first.tick, hash: first.hash, hashes: first.hashes, tier: first.tier, renderer: first.renderer, adapterError: first.adapterError, warnings: first.warnings, ...(first.netHashes ? { netHashes: first.netHashes, lockstep: first.lockstep } : {}) } : {}),
};
if (resultFile) {
  mkdirSync(dirname(resolve(resultFile)), { recursive: true });
  writeFileSync(resultFile, JSON.stringify(line) + "\n");
}
// bun test's own report (each test, each failure's diff) goes to stderr, so stdout stays the result.
process.stderr.write(stdout + stderr);
if (asJson) process.stdout.write(JSON.stringify(line) + "\n");
else {
  process.stdout.write(`${!ok ? "FAIL" : unchecked.length > 0 ? "UNCHECKED" : "PASS"} ${name} (${tests} tests, ${assertionCount} checks, ${unchecked.length} GPU frame/expectation checks unchecked, ${line.ms} ms)\n`);
  for (const f of failures) process.stdout.write(`  - ${f.split("\n").slice(0, 8).join("\n    ")}\n`);
  for (const capture of failureCaptures) process.stdout.write(`  evidence: ${capture.world ?? "world unavailable"}; ${capture.frame.status === "captured" ? capture.frame.path : capture.frame.reason}\n`);
  for (const u of unchecked) process.stdout.write(`  ~ ${u}: no renderer, frame unchecked\n`);
  for (const u of assertionLog.uncovered as Array<{ run: string; kind: string; name: string; key?: string }>) process.stdout.write(`  ? ${u.run}: ${u.kind} '${u.name}'${u.key !== undefined ? ` [${u.key}]` : ""} never fired\n`);
}
process.exit(ok ? 0 : 1);
