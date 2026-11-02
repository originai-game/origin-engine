// og.render.light2dAt (R11 batch 2): the 2D light reaching a point,
// evaluated on the CPU for game rules, inside a system as between ticks.
// Every expected value is worked out by hand from the lights' falloff.
//
// Ways it could fail, and the check that catches each:
// - an occluder read at its local transform, not under its parent → the
//   point behind the parented wall is lit
// - a polygon occluder's `points` ignored (the 1 × 1 rectangle used) or
//   not scaled with its entity → the point behind the triangle is lit
// - layers ignored → the red light (layer 2) reaches a layer-1 point
// - a system that did not declare the reads answered from identity poses
//   instead of refused
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const Seen = defineResource("LaSeen", { behind: f32(), open: f32(), triangle: f32(), red: f32(), refusals: u32() });
let refusal = "";

const Look = defineSystem("LaLook", { reads: [og.Light2D, og.Occluder2D, og.Transform, og.Parent], writes: [Seen] }, (_q, _t, world) => {
  const seen = must(world.resource(Seen));
  seen.behind = must(og.render.light2dAt(world, { x: 3, y: 0 })).r;
  seen.open = must(og.render.light2dAt(world, { x: 0, y: 3 })).r;
  seen.triangle = must(og.render.light2dAt(world, { x: -4, y: 0 })).r;
  seen.red = must(og.render.light2dAt(world, { x: 0, y: -3 }, 2)).r;
});
const Blind = defineSystem("LaBlind", { writes: [Seen] }, (_q, _t, world) => {
  const r = og.render.light2dAt(world, { x: 0, y: 3 });
  if (!r.ok) {
    must(world.resource(Seen)).refusals++;
    refusal = r.reason;
  }
});

function start(world: World): void {
  // White, intensity 2, radius 6, linear falloff, at the origin.
  must(world.insert(must(world.spawn({ name: "lamp" })), og.Light2D, { intensity: 2, radius: 6, falloff: 1 }));
  // Red on layer 2 only, below.
  must(world.insert(must(world.spawn({ name: "red", position: { x: 0, y: -1, z: 0 } })), og.Light2D, { color: { r: 1, g: 0, b: 0 }, radius: 4, falloff: 1, layers: 2 }));
  // A wall at x 2, through a parent at x 1.5 (the wall is 0.5 right of it).
  const holder = must(world.spawn({ name: "holder", position: { x: 1.5, y: 0, z: 0 } }));
  const wall = must(world.spawn({ name: "wall", parent: holder, position: { x: 0.5, y: 0, z: 0 } }));
  must(world.insert(wall, og.Occluder2D, { width: 0.4, height: 2 }));
  // A triangle at x -2, scaled 2: its points span x -1..1, y -1..1 in the world.
  const tri = must(world.spawn({ name: "triangle", position: { x: -2, y: 0, z: 0 }, scale: { x: 2, y: 2, z: 1 } }));
  must(world.insert(tri, og.Occluder2D, { points: [0.5, -0.5, 0.5, 0.5, -0.5, 0] }));
}

test("light2dAt sums the lights on the point's layers, shadowed by rectangles and polygons, in a system and between ticks", () => {
  const engine = must(createEngine(og.game({ tick: 60, seed: 1, resources: [Seen], systems: [Look, Blind], start }) as unknown as GameToken));
  must(engine.advance(3));
  const seen = must(engine.world.resource(Seen));
  // (1 - 3/6) * 2 = 1 beside the wall; behind the wall and the triangle nothing.
  expect(seen.open).toBe(1);
  expect(seen.behind).toBe(0);
  expect(seen.triangle).toBe(0);
  // The red light, 2 from (0, -3) of its 4: 1 - 2/4. The white one adds 1 there too.
  expect(seen.red).toBe(1.5);
  // Between ticks the same answers.
  expect(must(og.render.light2dAt(engine.world, { x: 0, y: 3 }))).toEqual({ r: 1, g: 1, b: 1 });
  expect(must(og.render.light2dAt(engine.world, { x: 3, y: 0 }))).toEqual({ r: 0, g: 0, b: 0 });
  // Short of the wall at x 2 the lamp reaches (an unparented wall, at x 0.5, would block it).
  expect(must(og.render.light2dAt(engine.world, { x: 1.2, y: 0 })).r).toBeCloseTo(1.6, 6);
  // Inside the triangle's bounding square but outside the triangle, behind its near edge:
  // the triangle shadows it (its 1 x 1 rectangle, scaled to that square, would hold it and not).
  expect(must(og.render.light2dAt(engine.world, { x: -2.9, y: 0.9 })).r).toBe(0);
  // Past the triangle's corner (its ray clears the edge at x -1 above y 1): 1 - d/6 of 2.
  expect(must(og.render.light2dAt(engine.world, { x: -2, y: 3.5 })).r).toBeCloseTo(2 * (1 - Math.sqrt(16.25) / 6), 6);
  // A system that did not declare the reads is refused, every tick.
  expect(seen.refusals).toBe(3);
  expect(refusal).toContain("og.Light2D");
  expect(og.render.light2dAt(engine.world, { x: Number.NaN, y: 0 }).ok).toBe(false);
  engine.destroy();
});
