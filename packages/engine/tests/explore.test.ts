// `og explore` against fixture games with planted bugs their own playtests
// miss (tests/fixtures/explore-*), and a control without one. The search is
// seeded, and bounded here by ticks (not seconds), so each run below searches
// the same way on every machine; the budgets leave room over the ticks each
// finding took when this was written (in the comment beside each).
//
// Ways it could fail, and the test that catches each:
// - the search cannot get deep (the cells or the selection keep it near the
//   start), cannot time a press to a rare draw, or cannot put two inputs a
//   tick apart → "og explore finds …" for deep, rare and combo
// - a finding is not shrunk, does not replay from tick 0 to the hash it
//   found, differs in a fresh og run or on the web host, or its playtest is
//   not written → the same tests' repro checks
// - it reports a violation in a game without one, or does not report the
//   sometimes site it reached → "og explore reports nothing wrong in the
//   control …"
// - the planted bugs are not missed by the fixtures' playtests (so the
//   fixtures prove nothing) → "the fixtures' own playtests pass …"
import { expect, test } from "bun:test";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

const STALE_WASM = process.env.OG_WASM_STALE === "1";
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

type Finding = {
  kind: string;
  site: { kind: string; origin: string; name: string };
  tick: number;
  field?: string | null;
  value?: unknown;
  shrunk: { presses: { from: number; to: number } };
  repro: { commands: string; hash: string; file: string };
  replays: Record<string, string | null>;
  deterministic: boolean;
  playtest: string;
};
type Report = { findings: Finding[]; coverage: { uncovered: { name: string }[] }; used: { ticks: number; stopped: string[] } };

/** A copy of the fixture under target/evidence (og explore and og test write into the project). */
function game(name: string): string {
  const dir = join(repo, "target", "evidence", "explore", name);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dirname(dir), { recursive: true });
  cpSync(join(pkg, "tests", "fixtures", name), dir, { recursive: true });
  return dir;
}

function explore(name: string, ticks: number, extra: string[] = []): { report: Report; dir: string } {
  const dir = game(name);
  const out = join(dir, "og-out", "explore");
  const web = STALE_WASM ? "off" : "on";
  const r = Bun.spawnSync([ogBin(), "explore", dir, "--seed", "1", "--ticks", String(ticks), "--seconds", "600", "--web", web, "--out", out, "--json", ...extra], {
    cwd: repo,
    env: { ...process.env, OG_ENGINE_PACKAGE: pkg, OG_RENDERER: "none" },
  });
  const stdout = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`og explore ${name}: ${new TextDecoder().decode(r.stderr)}${stdout}`);
  return { report: JSON.parse(stdout.trim().split("\n").pop()!) as Report, dir };
}

/** The violation of `name`, replayed alike everywhere, with its repro and playtest written. */
function violation(report: Report, name: string): Finding {
  const f = report.findings.find((x) => x.kind === "violation" && x.site.name === name);
  if (!f) throw new Error(`no violation of '${name}' among ${JSON.stringify(report.findings.map((x) => `${x.kind} ${x.site.name}`))}`);
  expect(f.deterministic).toBeTrue();
  expect(f.replays.ogRun, "a fresh og run ends on the hash the search found").toBe(f.repro.hash);
  if (!STALE_WASM) expect(f.replays.web, "the web host ends on the same hash").toBe(f.repro.hash);
  expect(existsSync(join(repo, f.repro.file)) || existsSync(f.repro.file)).toBeTrue();
  const playtest = readFileSync(existsSync(f.playtest) ? f.playtest : join(repo, f.playtest), "utf8");
  expect(playtest).toContain(JSON.stringify(f.repro.commands));
  return f;
}

test("og explore finds the deep state: level 5's vault, 44 presses in", () => {
  // Seed 1 found it after 28,864 ticks of search.
  const { report } = explore("explore-deep", 60_000, ["--first"]);
  const f = violation(report, "in range Dungeon.x");
  expect(f.site.origin).toBe("default");
  expect(f.value).toBe(11);
  // Four levels of ten steps and a descent each, then the fifth descent: no fewer presses do it.
  expect(f.shrunk.presses.to).toBeGreaterThanOrEqual(44);
  expect(f.shrunk.presses.to).toBeLessThanOrEqual(f.shrunk.presses.from);
}, 600_000);

test("og explore finds the rare draw: the jackpot's payout, one stop in 500", () => {
  // Seed 1 found it after 42,334 ticks: the first jackpot draw of the fixture's seed is at tick 956.
  const { report } = explore("explore-rare", 90_000, ["--first"]);
  const f = violation(report, "in range Slots.coins");
  expect(f.value as number).toBeLessThan(0);
  // One spin at the jackpot's tick is the whole repro.
  expect(f.shrunk.presses.to).toBe(1);
}, 600_000);

test("og explore finds the two-step combination: a taunt the tick after the guard", () => {
  // Seed 1 found it after 101 ticks.
  const { report } = explore("explore-combo", 5_000, ["--first"]);
  const f = violation(report, "in range Fighter.power");
  expect(f.value as number).toBeGreaterThan(100);
  // The guard and the taunt, nothing else.
  expect(f.shrunk.presses.to).toBe(2);
  expect(f.repro.commands).toContain("key_down:KeyQ");
  expect(f.repro.commands).toContain("intent:Taunt");
}, 600_000);

test("og explore reports nothing wrong in the control, and the sometimes site it reached", () => {
  const { report } = explore("explore-control", 20_000);
  expect(report.findings.filter((f) => f.kind !== "reached")).toEqual([]);
  expect(report.findings.some((f) => f.kind === "reached" && f.site.name === "descends to level 3")).toBeTrue();
  expect(report.coverage.uncovered).toEqual([]);
  expect(report.used.ticks).toBeGreaterThanOrEqual(20_000);
}, 600_000);

test("the fixtures' own playtests pass: each planted bug is out of their reach", () => {
  for (const name of ["explore-deep", "explore-rare", "explore-combo", "explore-control"]) {
    const r = Bun.spawnSync([ogBin(), "test", game(name)], { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg, OG_RENDERER: "none" } });
    const out = new TextDecoder().decode(r.stdout) + new TextDecoder().decode(r.stderr);
    if (r.exitCode !== 0) throw new Error(`${name}: ${out}`);
    expect(out).toContain("1 suite(s) passed");
  }
}, 300_000);
