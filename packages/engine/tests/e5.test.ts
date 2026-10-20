// The E5 round through the TypeScript API on the native host: every new
// physics component is adopted from the IDL by og-physics (a world with all
// of them assembles and steps), the controllers move by what a system sets
// and report through og.CharacterEvent, ropes, cloth, fluids and water are
// read back through og.physics, a body crossing water splashes it, the
// day-night clock is a function of the tick, the readouts refuse entities
// without the component, and a world with every feature runs twice to the
// same hash and returns to it after a restore.
import { beforeAll, expect, test } from "bun:test";
import {
  og,
  defineResource,
  defineSystem,
  f32,
  u32,
  SHAPE_BOX,
  SHAPE_CAPSULE,
  SHAPE_SPHERE,
  CHARACTER_JUMPED,
  JOINT_REVOLUTE,
  MOTOR_VELOCITY,
  PIN_TOP,
} from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type CreateOptions, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

function start(game: unknown, options: CreateOptions = {}): Engine {
  const r = createEngine(game as GameToken, options);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function box(world: World, name: string, position: { x: number; y: number; z: number }, size: { x: number; y: number; z: number }) {
  const e = must(world.spawn({ name, position }));
  world.insert(e, og.Collider, { shape: SHAPE_BOX, size });
  return e;
}

/** A 3D world with a joint, a character, a vehicle, a rope, cloth, water and a ragdoll-free crate. */
function world3d(world: World): void {
  box(world, "floor", { x: 0, y: -0.5, z: 0 }, { x: 60, y: 1, z: 60 });
  const hero = must(world.spawn({ name: "hero", position: { x: 0, y: 1, z: 0 } }));
  world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 1 });
  world.insert(hero, og.CharacterController, { move: { x: 2, y: 0, z: 0 } });
  const door = must(world.spawn({ name: "door", position: { x: -5, y: 1, z: 3 } }));
  world.insert(door, og.RigidBody);
  world.insert(door, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 2, z: 0.1 } });
  world.insert(door, og.Joint, { kind: JOINT_REVOLUTE, anchor: { x: -0.5, y: 0, z: 0 }, otherAnchor: { x: -5.5, y: 1, z: 3 }, axis: { x: 0, y: 1, z: 0 }, motor: MOTOR_VELOCITY, motorTarget: 1 });
  const car = must(world.spawn({ name: "car", position: { x: 10, y: 1, z: 10 } }));
  world.insert(car, og.RigidBody, { mass: 800 });
  world.insert(car, og.Collider, { shape: SHAPE_BOX, size: { x: 1.8, y: 0.6, z: 4 } });
  world.insert(car, og.Vehicle, { throttle: 0.6 });
  const rope = must(world.spawn({ name: "rope", position: { x: 4, y: 5, z: -4 } }));
  world.insert(rope, og.Rope, { segments: 12, length: 3 });
  const cloth = must(world.spawn({ name: "cloth", position: { x: -4, y: 4, z: -4 } }));
  world.insert(cloth, og.Cloth, { columns: 8, rows: 8, pins: PIN_TOP });
  const pool = must(world.spawn({ name: "pool", position: { x: 0, y: 0.5, z: -12 } }));
  world.insert(pool, og.WaterSurface, { resolution: 32, size: 8 });
  const crate = must(world.spawn({ name: "crate", position: { x: 0, y: 3, z: -12 } }));
  world.insert(crate, og.RigidBody, { mass: 50 });
  world.insert(crate, og.Collider, { shape: SHAPE_BOX, size: { x: 0.8, y: 0.8, z: 0.8 } });
}

/** A 2D world with a platformer, a fluid and a water line. */
function world2d(world: World): void {
  box(world, "ground", { x: 0, y: -0.5, z: 0 }, { x: 40, y: 1, z: 1 });
  const hero = must(world.spawn({ name: "hero", position: { x: 0, y: 1, z: 0 } }));
  world.insert(hero, og.Collider, { shape: SHAPE_BOX, size: { x: 0.6, y: 1, z: 1 } });
  world.insert(hero, og.PlatformerController, { moveX: 1 });
  const fluid = must(world.spawn({ name: "fluid", position: { x: 8, y: 2, z: 0 } }));
  world.insert(fluid, og.Fluid2D, { count: 200, size: { x: 2, y: 2 } });
  const pond = must(world.spawn({ name: "pond", position: { x: -8, y: 0.5, z: 0 } }));
  world.insert(pond, og.WaterLine, { width: 6, depth: 2, columns: 48 });
  const rock = must(world.spawn({ name: "rock", position: { x: -8, y: 4, z: 0 } }));
  world.insert(rock, og.RigidBody);
  world.insert(rock, og.Collider, { shape: SHAPE_SPHERE, radius: 0.4 });
}

const Jumps = defineResource("E5Jumps", { jumped: u32(), hold: u32() });
/** Holds jump from tick 30 to 40 and counts the CharacterEvents of kind jumped. */
const Jumper = defineSystem("E5Jumper", { reads: [og.CharacterEvent], writes: [Jumps, og.PlatformerController] }, (q, time, world) => {
  const j = must(world.resource(Jumps));
  for (const ev of world.events(og.CharacterEvent)) if (ev.kind === CHARACTER_JUMPED) j.jumped += 1;
  for (const a of q.archetypes()) {
    const c = a.column(og.PlatformerController).jump;
    for (let i = 0; i < a.count; i++) c[i] = time.tick >= 30 && time.tick < 40 ? 1 : 0;
  }
});

test("every physics component of the round is adopted and steps in 3D", () => {
  const engine = start(og.game({ start: world3d }));
  expect(engine.advance(120).ok).toBeTrue();
  const w = engine.world;
  const hero = w.find("hero")!;
  const c = must(w.get(hero, og.CharacterController));
  expect(c.grounded).toBeTrue();
  expect(must(w.get(hero, og.Transform)).position.x).toBeGreaterThan(2.5);
  const door = w.find("door")!;
  expect(Math.abs(must(w.get(door, og.Joint)).value)).toBeGreaterThan(0.5);
  const rope = must(og.physics.ropePoints(w, w.find("rope")!));
  expect(rope.length).toBe(13);
  expect(rope[12]!.y).toBeLessThan(5 - 2.5);
  const cloth = must(og.physics.clothPoints(w, w.find("cloth")!));
  expect(cloth.length).toBe(64);
  expect(must(og.physics.vehicleWheels(w, w.find("car")!)).length).toBe(4);
  // The crate fell into the pool: it splashed, and the surface moved.
  const pool = w.find("pool")!;
  const levels = [-1, -0.5, 0, 0.5, 1].map((dx) => must(og.physics.waterHeight(w, pool, dx, -12)));
  expect(levels.some((y) => Math.abs(y - 0.5) > 0.001)).toBeTrue();
  expect(must(og.physics.waterHeight(w, pool, 30, 30))).toBeCloseTo(0.5, 6);
  // Readouts refuse an entity without the component.
  expect(og.physics.ropePoints(w, w.find("floor")!).ok).toBeFalse();
  // What the last step cost, phase by phase, and where it ran.
  const stats = must(og.physics.stats(w));
  expect(stats.phases.map((p) => p.name)).toEqual([
    "gather", "sync", "joints", "ragdolls", "water", "rapier",
    "readout", "controllers", "soft", "fluids", "fluids3d", "writeBack",
  ]);
  expect(stats.bodies).toBeGreaterThan(0);
  expect(stats.threads).toBeGreaterThanOrEqual(1);
  expect(stats.ms).toBeGreaterThan(0);
  expect(stats.phases.every((p) => p.ms >= 0 && p.ms <= stats.ms)).toBeTrue();
});

test("a platformer jumps on a held button and its fluid and water line are read back (2D)", () => {
  const engine = start(og.game({ resources: [Jumps], systems: [Jumper], start: world2d }), { profile: "2d" });
  expect(engine.advance(90).ok).toBeTrue();
  const w = engine.world;
  expect(must(w.resource(Jumps)).jumped).toBe(1);
  expect(must(w.get(w.find("hero")!, og.Transform)).position.x).toBeGreaterThan(3);
  const fluid = must(og.physics.fluidParticles(w, w.find("fluid")!));
  expect(fluid.length).toBe(200);
  expect(Math.min(...fluid.map((p) => p.y))).toBeGreaterThan(-0.2);
  const pond = w.find("pond")!;
  const surface = [-2, -1, 0, 1, 2].map((dx) => must(og.physics.waterHeight(w, pond, -8 + dx, 0)));
  expect(surface.some((y) => Math.abs(y - 0.5) > 0.001)).toBeTrue();
});

test("the day-night clock is the start plus the simulated time times the speed", () => {
  const Clock = defineResource("E5Clock", { hours: f32() });
  const Read = defineSystem("E5ReadClock", { reads: [og.DayNight], writes: [Clock] }, (_q, _t, world) => {
    must(world.resource(Clock)).hours = og.atmosphere.timeOfDay(world);
  });
  const engine = start(
    og.game({
      resources: [Clock],
      systems: [Read],
      start: (world: World) => {
        const d = must(world.resource(og.DayNight));
        d.enabled = true;
        d.start = 23;
        d.speed = 1;
      },
    }),
  );
  expect(engine.advance(120).ok).toBeTrue();
  // 119 ticks of 1/60 s at one game hour per second, past midnight.
  expect(og.atmosphere.timeOfDay(engine.world)).toBeCloseTo((23 + 120 / 60) % 24, 4);
  expect(must(engine.world.resource(Clock)).hours).toBeCloseTo((23 + 119 / 60) % 24, 4);
});

test("get answers the sun og.DayNight drives as it is drawn, and og.Time whole", () => {
  const engine = start(
    og.game({
      start: (world: World) => {
        const d = must(world.resource(og.DayNight));
        d.enabled = true;
        d.start = 12;
        d.speed = 0;
        for (const name of ["sun", "lamp"]) {
          const e = must(world.spawn({ name }));
          world.insert(e, og.DirectionalLight, { color: { r: 1, g: 1, b: 1 }, intensity: 2.4 });
        }
      },
    }),
  );
  const read = (script: string) => {
    expect(engine.probe.commands(script)).not.toBeNull();
    expect(engine.advance(1).ok).toBeTrue();
    return engine.takeOutputs().map((o) => (JSON.parse(o.data) as { value: Record<string, unknown> }).value);
  };
  const [sun, day, lamp, time] = read("0:get:sun.og.DirectionalLight;0:get:og.DayNight;0:get:lamp.og.DirectionalLight;0:get:og.Time");
  // Noon: the sun is overhead-ish, warm white and bright; the stored 2.4 is not what is drawn.
  expect(sun!.drivenBy).toBe("og.DayNight");
  expect(sun!.intensity).not.toBeCloseTo(2.4, 3);
  expect((sun!.toLight as number[])[1]).toBeGreaterThan(0.5);
  expect(day!.hours).toBeCloseTo(12, 4);
  expect(day!.lightIntensity).toBeCloseTo(sun!.intensity as number, 5);
  expect(day!.toLight).toEqual(sun!.toLight);
  // Only the first directional light is driven.
  expect("drivenBy" in lamp!).toBeFalse();
  expect(lamp!.intensity).toBeCloseTo(2.4, 5);
  // og.Time as a system reads it: the clock, its rate and the time scale.
  expect([time!.rate, time!.scale, time!.hitStop]).toEqual([60, 1, 0]);
  expect(time!.scaledDt).toBeCloseTo(1 / 60, 6);
  // Midnight: the moon, cold and dim, opposite the sun.
  // (a script's times count from when it is scheduled)
  expect(engine.advance(59).ok).toBeTrue();
  const [night] = read("0:set:og.DayNight.start=0;0:get:sun.og.DirectionalLight");
  expect((night!.toLight as number[])[1]).toBeGreaterThan(0);
  expect(night!.intensity as number).toBeLessThan(0.5);
  expect((night!.color as number[])[2]).toBeGreaterThan((night!.color as number[])[0]!);
});

test("a world with every feature hashes the same twice and after a restore", () => {
  const run = (profile: "2d" | "3d") => {
    const engine = start(og.game({ resources: [Jumps], systems: [Jumper], start: profile === "2d" ? world2d : world3d }), { profile });
    expect(engine.advance(150).ok).toBeTrue();
    const snap = engine.probe.snapshot() as { id: string; hash: string };
    const at = engine.hash();
    expect(engine.advance(150).ok).toBeTrue();
    const end = engine.hash();
    engine.probe.restore(snap.id);
    expect(engine.hash()).toBe(at);
    expect(engine.advance(150).ok).toBeTrue();
    expect(engine.hash()).toBe(end);
    return end;
  };
  for (const profile of ["3d", "2d"] as const) expect(run(profile)).toBe(run(profile));
}, 30_000); // two profiles, every feature, three runs each: about 2 s alone, over 5 s beside other builds
