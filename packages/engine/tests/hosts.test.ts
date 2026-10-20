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

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

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

test.skipIf(STALE_WASM)("a probe argument of the wrong type is refused alike on both hosts, and the engine keeps running", () => {
  const run = (host: string) => {
    const r = Bun.spawnSync(["bun", "tests/support/probe-refusal.ts", "--host", host], { cwd: pkg });
    const out = new TextDecoder().decode(r.stdout);
    if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
    return JSON.parse(out.trim().split("\n").pop()!) as { refused: string; scheduled: number; tick: number; hash: string };
  };
  const [node, web] = [run("node"), run("web")];
  expect(web.refused).toBe("probe.commands: script must be a string, not object");
  expect(node.refused).toBe(web.refused);
  // After the refusal the same engine schedules, ticks and hashes as usual.
  expect(web.scheduled).toBe(1);
  expect(web.tick).toBe(30);
  expect(web.hash).toBe(node.hash);
}, 120_000);

function physics(host: string, ...extra: string[]) {
  const r = Bun.spawnSync(["bun", "tests/support/physics-hash.ts", "--host", host, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as {
    tick: number; hash: string; hashAtLoad: string; body63: { y: number } | null; tally: { entered: number; started: number } | null;
  };
}

for (const profile of ["3d", "2d"]) {
  test(`a ${profile} physics scene gives the same world hash on both hosts after 600 ticks`, () => {
    const node = physics("node", "--profile", profile);
    const web = physics("web", "--profile", profile);
    expect(node.tick).toBe(600);
    expect(web.hash).toBe(node.hash);
    expect(web.body63).toEqual(node.body63);
    // The simulation ran: bodies fell and settled above the floor, and the
    // collision and sensor events reached a system.
    expect(node.hash).not.toBe(node.hashAtLoad);
    expect(node.body63!.y).toBeGreaterThan(0);
    expect(node.body63!.y).toBeLessThan(6);
    expect(node.tally!.started).toBeGreaterThan(0);
    expect(node.tally!.entered).toBeGreaterThan(0);
  }, 180_000);
}

test.skipIf(STALE_WASM)("tweens, presets, hit stop, slow motion and camera feel give the same hashes on both hosts and replay from a snapshot", () => {
  const run = (host: string, ...extra: string[]) => {
    const r = Bun.spawnSync(["bun", "tests/support/feel-hash.ts", "--host", host, ...extra], { cwd: pkg });
    const out = new TextDecoder().decode(r.stdout);
    if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
    return JSON.parse(out.trim().split("\n").pop()!) as {
      checkpoints: { tick: number; hash: string }[];
      tally: { done: number; tags: number; frozen: number };
      end: string;
      replayed: string;
    };
  };
  const [node, web, again] = [run("node"), run("web"), run("node")];
  expect(node.checkpoints.map((c) => c.tick)).toEqual([20, 23, 40, 80, 140]);
  expect(web.checkpoints).toEqual(node.checkpoints);
  expect(again.checkpoints).toEqual(node.checkpoints);
  // The snapshot taken inside the hit stop replays into the same world.
  expect(node.replayed).toBe(node.end);
  expect(web.replayed).toBe(web.end);
  // The systems saw eight frozen ticks; every tween and group reported once
  // (sequence 3, parallel 4, squash and flash untagged; the yoyo never ends).
  expect(node.tally).toEqual({ done: 4, tags: 7, frozen: 8 });
  // The hit stop is in the hash: without it the worlds part after it starts.
  const free = run("node", "--no-hit-stop");
  expect(free.checkpoints[0]).toEqual(node.checkpoints[0]);
  expect(free.checkpoints[1]!.hash).not.toBe(node.checkpoints[1]!.hash);
  expect(free.tally.frozen).toBe(0);
}, 120_000);
