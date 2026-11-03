// og.Buoyancy through the TypeScript API on the native host: a crate floats
// on a still og.Water at the depth Archimedes gives, and a hull rides a
// gale's sea (og.Water.wind), the same on every run and through a restore
// (its lift is computed in the tick from the water the runtime reads as
// each tick starts, never from the GPU).
import { beforeAll, expect, test } from "bun:test";
import { og, SHAPE_BOX } from "@origingame/engine";
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

function body(world: World, name: string, position: { x: number; y: number; z: number }, size: { x: number; y: number; z: number }, density: number): void {
  const e = must(world.spawn({ name, position }));
  world.insert(e, og.RigidBody);
  world.insert(e, og.Collider, { shape: SHAPE_BOX, size, density });
  world.insert(e, og.Buoyancy, { samples: 6 });
}

/** A gale's sea 6 km across at y 0, a still pond at y 1 beside it, a hull on the sea and a flat crate on the pond. */
function waters(world: World): void {
  const sea = must(world.spawn({ name: "sea", position: { x: 0, y: 0, z: 0 } }));
  world.insert(sea, og.Water, { size: { x: 6000, y: 6000 }, segments: 256, wind: { x: 7, y: -18.7 }, choppiness: 1 });
  const pond = must(world.spawn({ name: "pond", position: { x: 4000, y: 1, z: 0 } }));
  world.insert(pond, og.Water, { size: { x: 20, y: 20 } });
  body(world, "hull", { x: 0, y: -4, z: -30 }, { x: 2, y: 0.8, z: 5 }, 0.5);
  body(world, "crate", { x: 4000, y: 2, z: 0 }, { x: 2, y: 0.5, z: 2 }, 0.6);
}

function start(): Engine {
  const r = createEngine(og.game({ start: waters }) as GameToken, {});
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const y = (engine: Engine, name: string) => must(engine.world.get(engine.world.find(name)!, og.Transform)).position.y;

test("a crate floats on still water at the depth Archimedes gives", () => {
  const engine = start();
  expect(engine.advance(900).ok).toBeTrue();
  // Density 0.6 in water of density 2: 0.3 of its 0.5 m under, its centre
  // 0.1 m over the pond's level 1. (A flat crate: a cube that light floats
  // tipped on an edge, as a wooden cube does.)
  expect(y(engine, "crate")).toBeCloseTo(1.1, 2);
  const b = must(engine.world.get(engine.world.find("crate")!, og.Buoyancy));
  expect(b.submerged).toBeCloseTo(0.3, 1);
}, 30_000);

test("a hull rides the gale's sea, the same on every run and through a restore", () => {
  const run = () => {
    const engine = start();
    const heights: number[] = [];
    const above: number[] = [];
    const wet: number[] = [];
    for (let i = 0; i < 6; i++) {
      expect(engine.advance(50).ok).toBeTrue();
      const w = engine.world;
      const at = must(w.get(w.find("hull")!, og.Transform)).position;
      heights.push(at.y);
      wet.push(must(w.get(w.find("hull")!, og.Buoyancy)).submerged);
      above.push(at.y - must(og.physics.waterHeight(w, w.find("sea")!, at.x, at.z)));
    }
    const snap = engine.probe.snapshot() as { id: string; hash: string };
    const at = engine.hash();
    expect(engine.advance(120).ok).toBeTrue();
    const end = engine.hash();
    engine.probe.restore(snap.id);
    expect(engine.hash()).toBe(at);
    expect(engine.advance(120).ok).toBeTrue();
    expect(engine.hash()).toBe(end);
    return { heights, above, wet, end };
  };
  const a = run();
  // It rides the waves, lifted and dropped metres by them, its centre
  // always within half a metre of the surface under it and its hull wet
  // through every crest (never sunk, never flung clear: density 0.5 in
  // water of 2 rests a quarter under).
  expect(Math.max(...a.heights) - Math.min(...a.heights)).toBeGreaterThan(1);
  for (const h of a.above) expect(Math.abs(h)).toBeLessThan(0.5);
  for (const s of a.wet) expect(s).toBeGreaterThan(0.1);
  const b = run();
  expect(b.heights).toEqual(a.heights);
  expect(b.end).toBe(a.end);
}, 60_000);
