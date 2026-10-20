// Lanes (crates/og-lanes) change where the parallel systems run, never what
// they produce: the reference scene with 2000 more boxes, 160 more walkers
// and 48 falling crates, natively with lanes off (OG_LANES=off) and on four threads
// (OG_LANES=4), each in its own process, has the same world hash and the
// same captured pixels at ticks 60, 180 and 300 (animation palettes,
// extraction and the frustum cull reach the pixels, the physics step the
// world hash). A replay from the tick-60 snapshot with lanes switched at run
// time (lanes:<on|off>) reaches tick 180's hash and frame again. The on run
// split animation and the cull (dump_lanes counts each site), and never the
// extraction site: the crowd's og.Mesh rows are read from their columns
// (owed row 50, tests/evidence/P4/perf/), so nothing in the scene is decoded
// row by row in two grains' worth;
// both runs cull on the CPU (OG_GFX_CULLING=cpu), since GPU culling never
// reaches the CPU cull.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
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
if (!existsSync(join(scene, "cooked", ".cook.ogt"))) {
  const c = Bun.spawnSync([ogBin(), "cook", scene], { cwd: repo });
  if (c.exitCode !== 0) throw new Error(new TextDecoder().decode(c.stderr));
}

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
  // Each lane site of the enlarged scene split at least once, not only one of them.
  for (const site of ["anim", "cull"] as const) {
    expect([site, on.lanes.sites[site].calls > 0]).toEqual([site, true]);
  }
  // 2,000 more og.Mesh entities and no row of them decoded on the lanes.
  expect(on.lanes.sites.extract.calls).toBe(0);
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
