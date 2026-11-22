// og.play on both hosts: the hit-volumes fixture's game (a hit volume on a
// posed hand swept through thin dummies and a wall, levers behind and
// before the wall for og.play.candidates) ends on the same world hash and
// the same counts on the native addon and the wasm module. Each host runs
// in its own process.
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cookProject } from "./support/og-bin.ts";

// The functional lane may run with a web module older than the native addon
// (scripts/check.sh bun-functional): the tests that compare the two hosts
// skip there and run in the full lane.
const STALE_WASM = process.env.OG_WASM_STALE === "1";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "tests", "fixtures", "hit-volumes");

function run(host: string) {
  const r = Bun.spawnSync(["bun", "tests/support/play-hash.ts", "--host", host, "--project", project], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as { host: string; tick: number; hash: string; facts: Record<string, number> };
}

test.skipIf(STALE_WASM)("swept hit volumes and candidates come out the same on both hosts", () => {
  cookProject(project, repo);
  const node = run("node");
  expect(node.tick).toBe(180);
  // Every dummy and the wall struck once a whole turn, never twice in one.
  expect(node.facts.dups).toBe(0);
  expect(node.facts.dummyA).toBeGreaterThanOrEqual(node.facts.turns!);
  expect(node.facts.leversHidden).toBe(1);
  const web = run("web");
  expect(web).toEqual({ ...node, host: "web" });
}, 180_000);
