// Water through the TypeScript API on the native host. og.physics.waterHeight
// answers an og.Water with a wind (og.Water.wind) with the surface its
// Gerstner waves draw at this tick: it rises and falls across the sea and
// over the ticks, is the water's rest level outside the surface and without
// a wind, and two runs read the same heights (the sea is computed in the
// tick with the engine's own math, never read back from the GPU).
// Whatever crosses any og.Water is an og.Splash, with or without og.Buoyancy.
// og.Buoyancy floats a crate on still water at the depth Archimedes gives,
// and a hull rides a gale's sea, the same on every run and through a
// restore.
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem, SHAPE_BOX } from "@origingame/engine";
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
  // Water without a wind is flat: its level, as every og.Water answers.
  expect(must(og.physics.waterHeight(w, w.find("pond")!, 500, 0))).toBeCloseTo(1, 6);
});

test("a system reads the sea's height inside the tick, as between ticks", () => {
  // A boat's system: og.physics.waterHeight inside a system reads through
  // its declared reads (og.Transform, og.Water), and answers what the same
  // call between ticks answers.
  const seen: number[] = [];
  const Float = defineSystem("Float", { reads: [og.Transform, og.Water] }, (_q: unknown, _t: unknown, world: World) => {
    seen.push(must(og.physics.waterHeight(world, world.find("sea")!, 7, -16.5)));
  });
  const r = createEngine(og.game({ start: seas, systems: [Float] }) as GameToken, {});
  if (!r.ok) throw new Error(r.reason);
  expect(r.value.advance(120).ok).toBeTrue();
  expect(seen.length).toBe(120);
  // Tick k's system sees the sea the world between ticks shows after k ticks.
  const w = run(119).engine.world;
  expect(seen[119]).toBeCloseTo(must(og.physics.waterHeight(w, w.find("sea")!, 7, -16.5)), 5);
  expect(Math.max(...seen) - Math.min(...seen)).toBeGreaterThan(0.01);
});

function body(world: World, name: string, position: { x: number; y: number; z: number }, size: { x: number; y: number; z: number }, density: number): void {
  const e = must(world.spawn({ name, position }));
  world.insert(e, og.RigidBody);
  world.insert(e, og.Collider, { shape: SHAPE_BOX, size, density });
  world.insert(e, og.Buoyancy, { samples: 6 });
}

/**
 * A gale's sea 6 km across at y 0, a still pond at y 1 beside it, a hull set
 * down on the sea where it stands (not dropped onto it) and a flat crate on the pond.
 */
function waters(world: World): void {
  const sea = must(world.spawn({ name: "sea", position: { x: 0, y: 0, z: 0 } }));
  world.insert(sea, og.Water, { size: { x: 6000, y: 6000 }, segments: 256, wind: { x: 7, y: -18.7 }, choppiness: 1 });
  const pond = must(world.spawn({ name: "pond", position: { x: 4000, y: 1, z: 0 } }));
  world.insert(pond, og.Water, { size: { x: 20, y: 20 } });
  // The hull starts at its resting draft (a quarter of its 1.6 m under) on
  // the sea where it is, not dropped onto it.
  const level = must(og.physics.waterHeight(world, sea, 0, -30));
  body(world, "hull", { x: 0, y: level + 0.4, z: -30 }, { x: 2, y: 0.8, z: 5 }, 0.5);
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
  const w = engine.world;
  expect(y(engine, "crate") - must(og.physics.waterHeight(w, w.find("pond")!, 4000, 0))).toBeCloseTo(0.1, 2);
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

test("a stone dropped into a pond is an og.Splash at its level", () => {
  // No og.Buoyancy anywhere: the pond takes the stone's splash by itself.
  const heard: { water: number; entity: number; y: number; speed: number }[] = [];
  const Listen = defineSystem("Listen", { reads: [og.Splash] }, (_q: unknown, _t: unknown, world: World) => {
    for (const s of world.events(og.Splash)) heard.push({ water: Number(s.water), entity: Number(s.entity), y: s.point.y, speed: s.speed });
  });
  const pond = (world: World) => {
    const water = must(world.spawn({ name: "pond", position: { x: 0, y: 1, z: 0 } }));
    world.insert(water, og.Water, { size: { x: 20, y: 20 } });
    const stone = must(world.spawn({ name: "stone", position: { x: 2, y: 3, z: 1 } }));
    world.insert(stone, og.RigidBody);
    world.insert(stone, og.Collider, { shape: SHAPE_BOX, size: { x: 0.4, y: 0.4, z: 0.4 } });
  };
  const r = createEngine(og.game({ start: pond, systems: [Listen] }) as GameToken, {});
  if (!r.ok) throw new Error(r.reason);
  expect(r.value.advance(90).ok).toBeTrue();
  const w = r.value.world;
  expect(heard.length).toBe(1);
  expect(heard[0]!.water).toBe(Number(w.find("pond")!));
  expect(heard[0]!.entity).toBe(Number(w.find("stone")!));
  expect(heard[0]!.y).toBeCloseTo(1, 5);
  // Two metres of free fall.
  expect(heard[0]!.speed).toBeLessThan(-5);
});
