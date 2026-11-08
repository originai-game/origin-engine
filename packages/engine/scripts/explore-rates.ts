// og explore's find rates: a measurement, not a check (tests/explore.test.ts
// checks one seed each within a tick budget). Each planted-bug fixture is
// searched once per seed with --first, the control for its whole budget, and
// the table says how many seeds found the bug and after how many ticks and
// milliseconds of search. Times depend on the build (OG_RELEASE=1
// scripts/build.sh for a release addon) and on the machine's load; ticks do
// not.
//
//   bun scripts/explore-rates.ts [--seeds N] [--seconds S] [--jobs J] [--json]
//
// Runs `og explore` (OG_BIN, else native/'s) on copies under
// target/evidence/explore-rates/, J searches at a time.

import { cpSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "../tests/support/og-bin.ts";

declare const Bun: {
  spawn(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "pipe"; stderr: "pipe" }): { exited: Promise<number>; stdout: ReadableStream<Uint8Array> };
};

const argv = process.argv.slice(2);
const opt = (name: string, fallback: number) => {
  const i = argv.indexOf(name);
  return i < 0 ? fallback : Number(argv[i + 1]);
};
const seeds = opt("--seeds", 10);
const seconds = opt("--seconds", 60);
const jobs = opt("--jobs", 3);
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");

const fixtures = [
  { name: "explore-deep", bug: "in range Dungeon.x" },
  { name: "explore-rare", bug: "in range Slots.coins" },
  { name: "explore-combo", bug: "in range Fighter.power" },
  { name: "explore-control", bug: null },
];

type Run = { fixture: string; seed: number; found: boolean; ticks: number; ms: number; violations: number };

async function one(fixture: (typeof fixtures)[number], seed: number): Promise<Run> {
  const dir = join(repo, "target", "evidence", "explore-rates", `${fixture.name}-${seed}`);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dirname(dir), { recursive: true });
  cpSync(join(pkg, "tests", "fixtures", fixture.name), dir, { recursive: true });
  const out = join(dir, "og-out", "explore");
  const flags = ["--seed", String(seed), "--seconds", String(fixture.bug ? seconds : Math.min(seconds, 30)), "--ticks", "100000000", "--web", "off", "--no-shrink", "--out", out, "--json"];
  if (fixture.bug) flags.push("--first");
  const p = Bun.spawn([ogBin(), "explore", dir, ...flags], { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg, OG_RENDERER: "none" }, stdout: "pipe", stderr: "pipe" });
  await new Response(p.stdout).text();
  if ((await p.exited) !== 0) throw new Error(`og explore ${fixture.name} --seed ${seed} failed`);
  const report = JSON.parse(readFileSync(join(out, "report.json"), "utf8")) as {
    findings: { kind: string; site: { name: string }; found: { afterTicks: number; afterMs: number } }[];
    used: { ticks: number; searchMs: number };
  };
  const hit = report.findings.find((f) => f.kind === "violation" && f.site.name === fixture.bug);
  return {
    fixture: fixture.name,
    seed,
    found: !!hit,
    ticks: hit ? hit.found.afterTicks : report.used.ticks,
    ms: hit ? hit.found.afterMs : report.used.searchMs,
    violations: report.findings.filter((f) => f.kind !== "reached").length,
  };
}

const queue = fixtures.flatMap((f) => Array.from({ length: f.bug ? seeds : Math.min(seeds, 3) }, (_, i) => [f, i + 1] as const));
const runs: Run[] = [];
await Promise.all(
  Array.from({ length: jobs }, async () => {
    for (let next = queue.shift(); next; next = queue.shift()) runs.push(await one(next[0], next[1]));
  }),
);

const range = (xs: number[]) => (xs.length === 0 ? "-" : `${Math.min(...xs)}–${Math.max(...xs)}`);
const rows = fixtures.map((f) => {
  const mine = runs.filter((r) => r.fixture === f.name);
  const found = mine.filter((r) => r.found);
  return f.bug
    ? { fixture: f.name, result: `${found.length}/${mine.length} found`, ticks: range(found.map((r) => r.ticks)), ms: range(found.map((r) => Math.round(r.ms))) }
    : { fixture: f.name, result: `${mine.reduce((a, r) => a + r.violations, 0)} violations in ${mine.length} runs`, ticks: range(mine.map((r) => r.ticks)), ms: range(mine.map((r) => Math.round(r.ms))) };
});
if (argv.includes("--json")) process.stdout.write(JSON.stringify({ seeds, seconds, jobs, rows, runs }) + "\n");
else {
  process.stdout.write(`| fixture | result | ticks | ms |\n|---|---|---|---|\n`);
  for (const r of rows) process.stdout.write(`| ${r.fixture} | ${r.result} | ${r.ticks} | ${r.ms} |\n`);
}
