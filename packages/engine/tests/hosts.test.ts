// The two hosts are one engine: the design's hello world (10,000 entities,
// one Move system) gives the same world hash after 600 ticks on the native
// addon and on the wasm module. Each host runs in its own process.
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

function hello(host: string, ...extra: string[]): { tick: number; hash: string; hashAtLoad: string; entities: number } {
  const r = Bun.spawnSync(["bun", "tests/support/hello-hash.ts", "--host", host, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!);
}

test("the native and the web host agree on the world hash after 600 ticks", () => {
  const node = hello("node");
  const web = hello("web");
  expect(node.tick).toBe(600);
  expect(node.entities).toBe(10000);
  expect(web.hash).toBe(node.hash);
  // The hash measures the simulation: it moved from load, and a different
  // seed (other spawn positions) gives another world.
  expect(node.hash).not.toBe(node.hashAtLoad);
  expect(hello("node", "--seed", "7").hash).not.toBe(node.hash);
}, 120_000);
