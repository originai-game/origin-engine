// Lanes (crates/og-lanes) change where the parallel systems run, never what
// they produce: the reference scene with 2000 more boxes, 160 more walkers
// and 48 falling crates, natively with lanes off (OG_LANES=off) and on four threads
// (OG_LANES=4), each in its own process, has the same world hash and the
// same captured pixels at ticks 60, 180 and 300 (animation palettes,
// extraction and the frustum cull reach the pixels, the physics step the
// world hash). A replay from the tick-60 snapshot with lanes switched at run
// time (lanes:<on|off>) reaches tick 180's hash and frame again. The on run
// split animation (dump_lanes counts each site), and never the cull: its
// 2,500 or so bounds are far below the 131,072 from which the frustum
// cull splits (tests/evidence/debt3/cull-lane-grain/NOTES.md; og-render's
// unit test checks the split cull keeps the same bounds); nor the
// extraction site: the crowd's og.Mesh rows are read from their columns
// (owed row 50, tests/evidence/P4/perf/), so nothing in the scene is decoded
// row by row in two grains' worth; nor the physics site: the crowd's 48
// crates are far below og-physics's body grain (200 bodies for every lane
// thread, 800 on four), so the small world steps on the engine thread and
// never wakes the pool;
// both runs cull on the CPU (OG_GFX_CULLING=cpu), since GPU culling never
// reaches the CPU cull. A headless pile of 1,000 boxes (./support/lanes-pile.ts)
// is the world that qualifies: on four lanes every step goes to the pool,
// with the same world hash as lanes off, and 760 boxes do not.
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const scene = join(repo, "crates", "og-tests", "reference-scene");

type Point = { tick: number; hash: string; frame: string };
type Run = {
  checkpoints: Point[];
  restored: string;
  replay: Point;
  lanes: {
    on: boolean;
    threads: number;
    pool: string;
    parallelCalls: number;
    parallelItems: number;
    sites: Record<"anim" | "extract" | "cull" | "physics", { calls: number; items: number }>;
  };
};

// Module level, not a hook: a cook on a loaded machine outlives a hook's timeout.
// An incremental cook: free when cooked/ is current, and a cooker
// version bump re-cooks instead of reading stale output.
const c = Bun.spawnSync([ogBin(), "cook", scene], { cwd: repo });
if (c.exitCode !== 0) throw new Error(new TextDecoder().decode(c.stderr));

function run(lanes: string, replayLanes: string): Run {
  const r = Bun.spawnSync(["bun", "tests/support/lanes-hash.ts", "--replay-lanes", replayLanes], { cwd: pkg, env: { ...process.env, OG_LANES: lanes, OG_GFX_CULLING: "cpu" } });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`OG_LANES=${lanes}: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Run;
}

test("lanes on and off give the same world hashes, the same pixels and the same replays", () => {
  const off = run("off", "on");
  const on = run("4", "off");
  // The on run ran in parallel and the off run did not: the comparison is never between two serial runs.
  expect([on.lanes.threads, on.lanes.pool]).toEqual([4, "native"]);
  expect(on.lanes.parallelCalls).toBeGreaterThan(0);
  expect(off.lanes.parallelCalls).toBe(0);
  // Animation split; the cull, under its threshold of bounds, did not.
  expect(on.lanes.sites.anim.calls).toBeGreaterThan(0);
  expect(on.lanes.sites.cull.calls).toBe(0);
  // 2,000 more og.Mesh entities and no row of them decoded on the lanes.
  expect(on.lanes.sites.extract.calls).toBe(0);
  // 48 crates step below the physics grain: the step never went to the pool.
  expect(on.lanes.sites.physics.calls).toBe(0);
  // The scene moves, so equal frames are not one still picture.
  expect(new Set(on.checkpoints.map((p) => p.frame)).size).toBe(3);
  expect(new Set(on.checkpoints.map((p) => p.hash)).size).toBe(3);
  expect(on.checkpoints).toEqual(off.checkpoints);
  // A replay from tick 60 with lanes switched reaches tick 180 again.
  for (const r of [on, off]) {
    expect(r.restored).toBe(r.checkpoints[0]!.hash);
    expect(r.replay).toEqual(r.checkpoints[1]!);
  }
  // The replay of the on run ran with lanes off.
  expect(on.lanes.on).toBe(false);
}, 600_000);

type Pile = { bodies: number; hash: string; lanes: Run["lanes"] };

function pile(lanes: string, bodies: number): Pile {
  const r = Bun.spawnSync(["bun", "tests/support/lanes-pile.ts", "--bodies", String(bodies)], { cwd: pkg, env: { ...process.env, OG_LANES: lanes } });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`OG_LANES=${lanes}, ${bodies} bodies: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Pile;
}

test("the physics step goes to the lanes from og-physics's threshold for the thread count, with the same world", () => {
  // Above og-physics's pool_bodies for four lanes (500), and below.
  const off = pile("off", 1000);
  const on = pile("4", 1000);
  const below = pile("4", 480);
  expect([on.lanes.threads, on.lanes.pool]).toEqual([4, "native"]);
  // The steps (241 ticks) went to the pool with their 1,000 bodies (the
  // contact readout splits besides once the pile has 1,024 pairs).
  expect(on.lanes.sites.physics.calls).toBeGreaterThan(200);
  expect(on.lanes.sites.physics.items).toBeGreaterThan(200 * 1000);
  expect(off.lanes.sites.physics.calls).toBe(0);
  // 480 boxes are under the threshold: on the engine thread, the pool never woken.
  expect(below.lanes.on).toBe(true);
  expect(below.lanes.sites.physics.calls).toBe(0);
  expect(on.hash).toBe(off.hash);
  expect(below.hash).not.toBe(on.hash);
}, 300_000);
