// The two hosts are one engine: the design's hello world (10,000 entities,
// one Move system) gives the same world hash after 600 ticks on the native
// addon and on the wasm module. Each host runs in its own process.
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";

// The functional lane may run with a web module older than the native addon
// (scripts/check.sh bun-functional): the tests that compare the two hosts
// skip there and run in the full lane.
const STALE_WASM = process.env.OG_WASM_STALE === "1";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe" }): { exited: Promise<number>; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

test("development digests agree across hosts and native lanes actually execute in parallel", async () => {
  const run = async (host: string, lanes: string) => {
    const result = Bun.spawn(["env", `OG_LANES=${lanes}`, "bun", "tests/support/determinism.ts", host], { cwd: pkg, stdout: "pipe", stderr: "pipe" });
    const [stdout, stderr, code] = await Promise.all([new Response(result.stdout).text(), new Response(result.stderr).text(), result.exited]);
    if (code !== 0) throw new Error(stderr);
    const value = JSON.parse(stdout.trim().split("\n").pop()!);
    expect(Object.keys(value.digests).sort()).toEqual(["anim", "boxes", "math", "physics2d", "physics3d", "text"]);
    for (const digest of Object.values(value.digests)) expect(digest).toMatch(/^[0-9a-f]{16}$/);
    return value;
  };
  // These isolated worlds are independent; do not sum three long scenarios
  // into one deadline while the full gate is compiling beside them.
  const [serial, parallel, web] = await Promise.all([run("node", "off"), run("node", "4"), run("web", "off")]);
  expect(parallel.digests).toEqual(serial.digests);
  expect(web.digests).toEqual(serial.digests);
  expect(serial.parallelCalls).toBe(0);
  expect(parallel.parallelCalls).toBeGreaterThan(0);
}, 180_000);

function hello(host: string, ...extra: string[]): { tick: number; hash: string; hashAtLoad: string; entities: number } {
  const r = Bun.spawnSync(["bun", "tests/support/hello-hash.ts", "--host", host, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!);
}

test.skipIf(STALE_WASM)("the native and the web host agree on the world hash after 600 ticks", () => {
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

function physics(host: string, ...extra: string[]) {
  const r = Bun.spawnSync(["bun", "tests/support/physics-hash.ts", "--host", host, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as {
    tick: number; hash: string; hashAtLoad: string; body63: { y: number } | null; tally: { entered: number; started: number; inside: number } | null;
    snapshot?: string; refused?: string; restored?: { tick: number; hash: string };
    lanes?: { threads: number; pool: string; parallelCalls: number; sites: Record<string, { calls: number }> };
    modules?: string[];
  };
}

for (const profile of ["3d", "2d"]) {
  test.skipIf(STALE_WASM)(`a ${profile} physics scene gives the same world hash on both hosts after 600 ticks`, () => {
    const node = physics("node", "--profile", profile);
    const web = physics("web", "--profile", profile);
    expect(node.tick).toBe(600);
    expect(web.hash).toBe(node.hash);
    expect(web.body63).toEqual(node.body63);
    expect(web.tally).toEqual(node.tally);
    // The simulation ran: bodies fell and settled above the floor, the
    // collision and sensor events reached a system, and bodies stayed in the sensor.
    expect(node.hash).not.toBe(node.hashAtLoad);
    expect(node.body63!.y).toBeGreaterThan(0);
    expect(node.body63!.y).toBeLessThan(6);
    expect(node.tally!.started).toBeGreaterThan(0);
    expect(node.tally!.entered).toBeGreaterThan(0);
    expect(node.tally!.inside).toBeGreaterThan(node.tally!.entered);
  }, 180_000);
}
