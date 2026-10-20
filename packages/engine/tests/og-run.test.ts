// `og run --headless` loads the hello world on the native host and prints
// the same world hash the web host computes.
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
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
    expect(r.exitCode, `${args.join(" ")}: ${r.stderr}`).not.toBe(0);
    expect(r.stderr).toContain("GameDesc.start threw");
    expect(r.stderr).toContain("the village has no well");
    expect(r.stderr).toContain("buildTheVillage");
  }
}, 120_000);
