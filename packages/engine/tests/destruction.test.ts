// og.destruction on both hosts: examples/demolition-yard's wall, pillar and
// pane (each tuned by og.destruction.material) blown, struck and hit give
// the same world hash, the same og.Fractured events and the same props'
// state on the native addon and the wasm module: the fracture, the stress
// solver and the chunks' bodies are one deterministic simulation. Each
// host runs in its own process.
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
const project = join(repo, "examples", "demolition-yard");

function run(host: string) {
  const r = Bun.spawnSync(["bun", "tests/support/destruction-hash.ts", "--host", host, "--project", project], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as { host: string; tick: number; hash: string; hashAtLoad: string; fractured: number; chunks: number; info: Record<string, { intact: number; islands: number; health: number }> };
}

test.skipIf(STALE_WASM)("a wall, a pillar and a pane break the same way on both hosts", () => {
  cookProject(project, repo);
  const node = run("node");
  const web = run("web");
  expect(node.tick).toBe(240);
  // Each prop broke: the wall is down to its anchored stubs; the pillar
  // lost the struck band and keeps its stump (concrete's impactThreshold
  // lets its own falling top land on it, 302c3067f), so more than half of
  // its bonds hold; the pane is all shards.
  expect(node.fractured).toBeGreaterThan(0);
  expect(node.info.wall!.health).toBeLessThan(0.5);
  expect(node.info.pillar!.health).toBeLessThan(0.75);
  expect(node.info.pillar!.health).toBeGreaterThan(0.25);
  expect(node.info.pane!.intact).toBe(0);
  expect(node.hash).not.toBe(node.hashAtLoad);
  expect(web).toEqual({ ...node, host: "web" } as never);
}, 180_000);

// The yard as og run plays it (its own game, commands and beats, the native
// host): its charge leaves two piers of standing brickwork, one a third of
// the wall's 2.4 m, nothing spanning the hole, and the struck pillar a
// stump that its own falling top does not crush. Each held in the frames
// the art review reads, and each broken by a change that passed the
// wall-alone test (a charge reaching the piers' feet, M7's 0.55 and 0.75 m).
test("the demolition yard leaves two piers, a clear hole and a pillar stump", () => {
  cookProject(project, repo);
  const r = Bun.spawnSync(["bun", "tests/support/yard-piers.ts", "--project", project], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${new TextDecoder().decode(r.stderr)}${out}`);
  const y = JSON.parse(out.trim().split("\n").pop()!) as { left: number; right: number; spans: number; stump: number };
  expect(Math.min(y.left, y.right)).toBeGreaterThan(0.5);
  expect(Math.max(y.left, y.right)).toBeGreaterThan(0.8);
  expect(y.spans).toBe(0);
  expect(y.stump).toBeGreaterThan(0.6);
}, 180_000);
