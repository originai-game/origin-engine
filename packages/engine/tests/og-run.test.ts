// `og run --headless` loads the hello world on the native host and prints
// the same world hash the web host computes.
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { ogBin } from "./support/og-bin.ts";

// The functional lane may run with a web module older than the native addon
// (scripts/check.sh bun-functional): the tests that compare the two hosts
// skip there and run in the full lane.
const STALE_WASM = process.env.OG_WASM_STALE === "1";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const spawn = (cmd: string[], cwd: string) => {
  const r = Bun.spawnSync(cmd, { cwd });
  return { exitCode: r.exitCode, stdout: new TextDecoder().decode(r.stdout), stderr: new TextDecoder().decode(r.stderr) };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");

const run = (cmd: string[], cwd: string) => {
  const r = Bun.spawnSync(cmd, { cwd });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${cmd.join(" ")}: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!);
};

test.skipIf(STALE_WASM)("og run --headless prints the hello world's hash after 600 ticks", () => {
  const og = run(
    [ogBin(), "run", "--headless", "packages/engine/examples/hello.ts", "--ticks", "600", "--setup", "spawn_wave:10000", "--json"],
    repo,
  );
  const web = run(["bun", "tests/support/hello-hash.ts", "--host", "web"], pkg);
  expect(og.tick).toBe(600);
  expect(og.entities).toBe(10000);
  expect(og.hash).toBe(web.hash);
}, 300_000);

test("a throw in GameDesc.start fails og run, named with its stack, with and without a command script", () => {
  const game = "packages/engine/tests/fixtures/start-throws/main.ts";
  for (const args of [["--ticks", "10"], ["--commands", "0.1:quit"], ["--commands", "0.1:quit", "--no-render"]]) {
    const r = spawn([ogBin(), "run", "--headless", game, ...args], repo);
    if (r.exitCode === 0) throw new Error(`og run ${args.join(" ")} exited 0: ${r.stderr}${r.stdout}`);
    expect(r.stderr).toContain("GameDesc.start threw");
    expect(r.stderr).toContain("the village has no well");
    expect(r.stderr).toContain("buildTheVillage");
  }
}, 120_000);

test("og run takes tick and seed from project.ogt for a GameDesc that leaves them unset", () => {
  const out = mkdtempSync(join(tmpdir(), "og-project-"));
  try {
    const r = spawn([ogBin(), "run", "--headless", "packages/engine/tests/fixtures/project-defaults", "--no-render", "--out", out, "--commands", "0:get:og.Time:time.json;0:get:og.Rng.seed:seed.json;0.1:quit"], repo);
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${r.stderr}${r.stdout}`);
    const read = (f: string) => (JSON.parse(readFileSync(join(out, f), "utf8")) as { value: unknown }).value;
    expect((read("time.json") as { rate: number }).rate).toBe(30);
    expect(read("seed.json")).toBe(5);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 120_000);

test("a restore that moves og.Time past the script's last scheduled tick still runs the steps after it, and quit ends the run", () => {
  // brasswick FINDINGS #13: the run ended at the first tick past the tick
  // its last step was scheduled for, so a Continue restoring a later save
  // ended it at once, without the steps after it, a quit or a warning.
  const game = "packages/engine/tests/fixtures/restore-forward";
  const first = spawn([ogBin(), "run", "--headless", game, "--no-render", "--commands", "5:print_save;5.1:quit"], repo);
  if (first.exitCode !== 0) throw new Error(`og run exited ${first.exitCode}: ${first.stderr}${first.stdout}`);
  const save = first.stdout.split("\n").find((l) => l.startsWith("og-save:1:"));
  if (!save) throw new Error(`no save printed: ${first.stdout}`);
  const out = mkdtempSync(join(tmpdir(), "og-restore-forward-"));
  try {
    // Tick 3 restores the save of tick 150 as it ends, so tick 150 follows in place of tick 4: the
    // get (due at 15) and the quit (due at 18) keep the 11 and 14 ticks they had left.
    const r = spawn([ogBin(), "run", "--headless", game, "--no-render", "--out", out, "--json", "--commands", `0.1:load:${save};0.5:get:og.Time.tick:tick.json;0.6:quit`], repo);
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${r.stderr}${r.stdout}`);
    const summary = JSON.parse(r.stdout.trim().split("\n").pop()!) as { tick: number; quit: boolean };
    expect(summary.quit).toBeTrue();
    expect(existsSync(join(out, "tick.json"))).toBeTrue();
    expect((JSON.parse(readFileSync(join(out, "tick.json"), "utf8")) as { value: number }).value).toBe(150 + 11);
    expect(summary.tick).toBe(150 + 14);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 120_000);
