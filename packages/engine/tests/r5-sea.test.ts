// R5's sea through the TypeScript API on the native host: og.physics.waterHeight
// answers an og.Water with a wind (og.Water.wind) with the surface its
// Gerstner waves draw at this tick. The height rises and falls across the
// sea and over the ticks, within the wind's sea's reach; it is the water's
// rest level outside the surface and without a wind; and two runs of the same
// game read the same heights (the sea is computed in the tick with the
// engine's own math, never read back from the GPU).
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** A sea 400 m across at y 2 under a 10 m/s wind along -z, and a still pond beside it. */
function seas(world: World): void {
  const sea = must(world.spawn({ name: "sea", position: { x: 0, y: 2, z: 0 } }));
  world.insert(sea, og.Water, { size: { x: 400, y: 400 }, segments: 128, wind: { x: 0, y: -10 }, choppiness: 0.8 });
  const pond = must(world.spawn({ name: "pond", position: { x: 500, y: 1, z: 0 } }));
  world.insert(pond, og.Water, { size: { x: 20, y: 20 } });
}

function run(ticks: number): { engine: Engine; heights: number[] } {
  const r = createEngine(og.game({ start: seas }) as GameToken, {});
  if (!r.ok) throw new Error(r.reason);
  const engine = r.value;
  expect(engine.advance(ticks).ok).toBeTrue();
  const w = engine.world;
  const sea = w.find("sea")!;
  const heights = [-30, -10, 0, 7, 25, 60].map((x) => must(og.physics.waterHeight(w, sea, x, x * 0.5 - 20)));
  return { engine, heights };
}

test("a wind's sea answers waterHeight with its waves, the same on every run", () => {
  const a = run(120);
  // A 10 m/s sea: 2.1 m significant height, so the surface stands within a
  // few metres of the rest level 2 and is not flat.
  for (const h of a.heights) {
    expect(Number.isFinite(h)).toBeTrue();
    expect(Math.abs(h - 2)).toBeLessThan(4);
  }
  const spread = Math.max(...a.heights) - Math.min(...a.heights);
  expect(spread).toBeGreaterThan(0.3);
  // The same game, run again, reads the same surface.
  const b = run(120);
  expect(b.heights).toEqual(a.heights);
  // A second later the waves have moved on.
  const c = run(180);
  expect(c.heights).not.toEqual(a.heights);

  const w = a.engine.world;
  // Outside the surface's 400 m: the rest level.
  expect(must(og.physics.waterHeight(w, w.find("sea")!, 260, 0))).toBeCloseTo(2, 6);
  // Water without a wind has no sea: refused as before when it has no
  // simulated surface either.
  expect(og.physics.waterHeight(w, w.find("pond")!, 500, 0).ok).toBeFalse();
});
