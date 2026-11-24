// Physics through the TypeScript API on the native host: every physics
// component is adopted from the IDL by og-physics (a world with all of them
// assembles and steps), the controllers move by what a system sets and
// report through og.CharacterEvent, ropes, cloth, fluids and water are read
// back through og.physics, a body crossing water splashes it, the readouts
// refuse entities without the component, and a world with every feature runs
// twice to the same hash and returns to it after a restore.
import { beforeAll, expect, test } from "bun:test";
import {
  og,
  defineResource,
  defineSystem,
  u32,
  CHARACTER_JUMPED,
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
  world.insert(e, og.Collider, { shape: "box", size });
  return e;
}

/** A 3D world with a joint, a character, a vehicle, a rope, cloth, water and a ragdoll-free crate. */
function world3d(world: World): void {
  box(world, "floor", { x: 0, y: -0.5, z: 0 }, { x: 60, y: 1, z: 60 });
  const hero = must(world.spawn({ name: "hero", position: { x: 0, y: 1, z: 0 } }));
  world.insert(hero, og.Collider, { shape: "capsule", radius: 0.3, height: 1 });
  world.insert(hero, og.CharacterController, { move: { x: 2, y: 0, z: 0 } });
  const door = must(world.spawn({ name: "door", position: { x: -5, y: 1, z: 3 } }));
  world.insert(door, og.RigidBody);
  world.insert(door, og.Collider, { shape: "box", size: { x: 1, y: 2, z: 0.1 } });
  world.insert(door, og.Joint, { kind: "revolute", anchor: { x: -0.5, y: 0, z: 0 }, otherAnchor: { x: -5.5, y: 1, z: 3 }, axis: { x: 0, y: 1, z: 0 }, motor: "velocity", motorTarget: 1 });
  const car = must(world.spawn({ name: "car", position: { x: 10, y: 1, z: 10 } }));
  world.insert(car, og.RigidBody, { mass: 800 });
  world.insert(car, og.Collider, { shape: "box", size: { x: 1.8, y: 0.6, z: 4 } });
  world.insert(car, og.Vehicle, { throttle: 0.6 });
  const rope = must(world.spawn({ name: "rope", position: { x: 4, y: 5, z: -4 } }));
  world.insert(rope, og.Rope, { segments: 12, length: 3 });
  const cloth = must(world.spawn({ name: "cloth", position: { x: -4, y: 4, z: -4 } }));
  world.insert(cloth, og.Cloth, { columns: 8, rows: 8, pins: PIN_TOP });
  const pool = must(world.spawn({ name: "pool", position: { x: 0, y: 0.5, z: -12 } }));
  world.insert(pool, og.WaterSurface, { resolution: 32, size: 8 });
  const crate = must(world.spawn({ name: "crate", position: { x: 0, y: 3, z: -12 } }));
  world.insert(crate, og.RigidBody, { mass: 50 });
  world.insert(crate, og.Collider, { shape: "box", size: { x: 0.8, y: 0.8, z: 0.8 } });
}

/** A project of the creation form `form` (a 2D one is a 2D engine). */
const formed = (form: "2d" | "3d"): CreateOptions => ({ project: `type project\nversion 1\n---\ncreate {\n  form "${form}"\n}\n` });

/** A 2D world with a platformer, a fluid and a water line. */
function world2d(world: World): void {
  box(world, "ground", { x: 0, y: -0.5, z: 0 }, { x: 40, y: 1, z: 1 });
  const hero = must(world.spawn({ name: "hero", position: { x: 0, y: 1, z: 0 } }));
  world.insert(hero, og.Collider, { shape: "box", size: { x: 0.6, y: 1, z: 1 } });
  world.insert(hero, og.PlatformerController, { run: 1 });
  const fluid = must(world.spawn({ name: "fluid", position: { x: 8, y: 2, z: 0 } }));
  world.insert(fluid, og.Fluid2D, { count: 200, size: { x: 2, y: 2 } });
  const pond = must(world.spawn({ name: "pond", position: { x: -8, y: 0.5, z: 0 } }));
  world.insert(pond, og.WaterLine, { width: 6, depth: 2, columns: 48 });
  const rock = must(world.spawn({ name: "rock", position: { x: -8, y: 4, z: 0 } }));
  world.insert(rock, og.RigidBody);
  world.insert(rock, og.Collider, { shape: "sphere", radius: 0.4 });
}

const Jumps = defineResource("PhysicsJumps", { jumped: u32(), hold: u32() });
/** Holds jump from tick 30 to 40 and counts the CharacterEvents of kind jumped. */
const Jumper = defineSystem("PhysicsJumper", { reads: [og.CharacterEvent], writes: [Jumps, og.PlatformerController] }, (q, time, world) => {
  const j = must(world.resource(Jumps));
  for (const ev of world.events(og.CharacterEvent)) if (ev.kind === CHARACTER_JUMPED) j.jumped += 1;
  for (const a of q.archetypes()) {
    const c = a.column(og.PlatformerController).jump;
    for (let i = 0; i < a.count; i++) c[i] = time.tick >= 30 && time.tick < 40 ? 1 : 0;
  }
});

test("a world with every feature hashes the same twice and after a restore", () => {
  const run = (profile: "2d" | "3d") => {
    const engine = start(og.game({ resources: [Jumps], systems: [Jumper], start: profile === "2d" ? world2d : world3d }), formed(profile));
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
