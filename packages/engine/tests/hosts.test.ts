// The two hosts are one engine: the design's hello world (10,000 entities,
// one Move system) gives the same world hash after 600 ticks on the native
// addon and on the wasm module. Each host runs in its own process.
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
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
    snapshot?: string; refused?: string; restored?: { tick: number; hash: string };
    lanes?: { threads: number; pool: string; parallelCalls: number; sites: Record<string, { calls: number }> };
  };
}

for (const profile of ["3d", "2d"]) {
  test.skipIf(STALE_WASM)(`a ${profile} physics scene gives the same world hash on both hosts after 600 ticks`, () => {
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

// One lanes module per physics profile (owed row 71, docs/web-export.md
// §Variants): the 3D one in dist/web-publish, the 2D one in
// dist/web-publish-2d, each with only its profile's physics, wasm atomics
// and the publish members. Each steps its profile's scene to the native
// world, on the engine thread alone and with 1,024 bodies on a pool of four
// Workers (wasm-bindgen-rayon's, under bun as in a page; every step went to
// the pool): the same snapshot as the native host's at tick 600, which
// restored natively has the native world's hash (a publish module has no
// world dump of its own). And it refuses a world of the other profile.
for (const profile of ["3d", "2d"] as const) {
  test.skipIf(STALE_WASM)(`the ${profile} lanes module steps a ${profile} physics scene to the native world, on one thread and on four Workers, and refuses the other profile`, () => {
    const dir = mkdtempSync(join(tmpdir(), "og-lanes-profile-"));
    for (const [bodies, threads] of [["64", null], ["1024", "4"]] as const) {
      const node = physics("node", "--profile", profile, "--bodies", bodies, "--snapshot");
      const lanes = physics("lanes", "--profile", profile, "--bodies", bodies, "--snapshot", ...(threads ? ["--threads", threads] : []));
      const label = `${bodies} bodies, ${threads ?? 1} lane(s)`;
      expect([label, lanes.tick]).toEqual([label, 600]);
      if (threads) {
        expect([label, lanes.lanes!.pool, lanes.lanes!.threads]).toEqual([label, "web-workers", 4]);
        expect([label, lanes.lanes!.sites.physics.calls]).toEqual([label, 600]);
      } else {
        expect([label, lanes.lanes!.pool, lanes.lanes!.parallelCalls]).toEqual([label, "none", 0]);
      }
      expect([label, lanes.body63]).toEqual([label, node.body63]);
      expect([label, lanes.tally]).toEqual([label, node.tally]);
      expect(lanes.snapshot!.length).toBeGreaterThan(0);
      expect([label, lanes.snapshot]).toEqual([label, node.snapshot!]);
      const file = join(dir, `lanes-${profile}-${bodies}.snapshot`);
      writeFileSync(file, lanes.snapshot!);
      expect([label, physics("node", "--profile", profile, "--bodies", bodies, "--restore", file).restored]).toEqual([label, { tick: 600, hash: node.hash }]);
    }
    const other = profile === "2d" ? "3d" : "2d";
    expect(physics("lanes", "--profile", other, "--module", profile).refused).toContain(`without the physics${other} feature`);
  }, 180_000);
}

test.skipIf(STALE_WASM)("the probe's memory answers the same shape on both hosts, each with what its host can read", () => {
  type Memory = { host: string; linear: number | null; resident: number | null; heap: { live: number; peak: number; allocations: number } | null; world: { entities: number }; stream: { budget: number; used: number; peak: number } | null; audio: number | null };
  const run = (host: string) => {
    const r = Bun.spawnSync(["bun", "tests/support/probe-memory.ts", "--host", host], { cwd: pkg });
    const out = new TextDecoder().decode(r.stdout);
    if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
    return JSON.parse(out.trim().split("\n").pop()!) as { memory: Memory; worker: Memory | null };
  };
  const [node, web] = [run("node"), run("web")];
  expect(Object.keys(web.memory).sort()).toEqual(Object.keys(node.memory).sort());
  expect(Object.keys(node.memory).sort()).toEqual(["audio", "heap", "host", "linear", "resident", "stream", "world"]);
  expect([node.memory.host, web.memory.host]).toEqual(["native", "web"]);
  for (const m of [node.memory, web.memory]) {
    expect(m.world.entities).toBe(10000);
    expect(m.heap!.live).toBeGreaterThan(0);
    expect(m.heap!.peak).toBeGreaterThan(m.heap!.live - 1);
    expect(m.heap!.allocations).toBeGreaterThan(0);
    expect(m.stream!.budget).toBe(256 * 1024 * 1024);
  }
  // The wasm's linear memory holds the heap; a native process reads its resident memory instead.
  expect(web.memory.linear!).toBeGreaterThan(web.memory.heap!.live);
  expect(web.memory.resident).toBeNull();
  expect(node.memory.linear).toBeNull();
  expect(node.memory.resident!).toBeGreaterThan(node.memory.heap!.live);
  // The engine Worker's `memory` call is the probe's answer.
  expect(web.worker!.world.entities).toBe(10000);
  // Read a moment later, after the probe's call grew the memory or not: never smaller.
  expect(web.worker!.linear!).toBeGreaterThanOrEqual(web.memory.linear!);
}, 120_000);
