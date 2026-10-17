// One host per process: a physics scene (a static floor, a ramp, 64 boxes and
// spheres dropped from Rng-chosen poses, a kinematic paddle moved by a
// system, a sensor counting entries) advanced --ticks fixed steps on the host
// named by --host. Prints JSON { host, tick, hash, ... } for hosts.test.ts.
//
//   bun tests/support/physics-hash.ts --host node|web [--ticks 600] [--profile 3d|2d]

import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { og, defineComponent, defineResource, defineSystem, u32, SHAPE_BOX, SHAPE_SPHERE, BODY_KINEMATIC } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1]! : fallback;
};
const which = opt("--host", "node");
const ticks = Number(opt("--ticks", "600"));
const profile = opt("--profile", "3d") as "2d" | "3d";
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

if (which === "node") {
  const { loadNativeHost } = await import("../../src/node/index.ts");
  loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}

const Tally = defineResource("PhTally", { entered: u32(), started: u32() });
const PaddleTag = defineComponent("PhPaddle", { lane: u32() });

/** The paddle sweeps back and forth along x: velocity only, no trigonometry. */
const Paddle = defineSystem("PhPaddleMove", { reads: [PaddleTag], writes: [og.Velocity] }, (q, time) => {
  for (const a of q.archetypes()) {
    const v = a.column(og.Velocity).linear;
    const dir = Math.floor(time.tick / 120) % 2 === 0 ? 2 : -2;
    for (let i = 0; i < a.count; i++) v[i * 3] = dir;
  }
});

/** Counts the events physics sent last tick, read inside a system. */
const Count = defineSystem("PhCount", { reads: [og.SensorEntered, og.CollisionStarted], writes: [Tally] }, (_q, _t, world) => {
  const t = world.resource(Tally);
  if (!t.ok) return;
  const entered = world.events(og.SensorEntered).length;
  const started = world.events(og.CollisionStarted).length;
  if (entered) t.value.entered += entered;
  if (started) t.value.started += started;
});

function scene(world: World): void {
  const floor = world.spawn({ name: "floor", position: { x: 0, y: -0.5, z: 0 } });
  if (!floor.ok) throw new Error(floor.reason);
  world.insert(floor.value, og.Collider, { shape: SHAPE_BOX, size: { x: 40, y: 1, z: 40 } });
  const ramp = world.spawn({ name: "ramp", position: { x: -3, y: 1, z: 0 }, quaternion: { x: 0, y: 0, z: 0.2588, w: 0.9659 } });
  if (!ramp.ok) throw new Error(ramp.reason);
  world.insert(ramp.value, og.Collider, { shape: SHAPE_BOX, size: { x: 6, y: 0.4, z: 4 } });
  const paddle = world.spawn({ name: "paddle", position: { x: 0, y: 0.5, z: 2 } });
  if (!paddle.ok) throw new Error(paddle.reason);
  world.insert(paddle.value, og.RigidBody, { kind: BODY_KINEMATIC });
  world.insert(paddle.value, og.Velocity);
  world.insert(paddle.value, PaddleTag);
  world.insert(paddle.value, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 1, z: 3 } });
  const sensor = world.spawn({ name: "sensor", position: { x: 3, y: 0.5, z: 0 } });
  if (!sensor.ok) throw new Error(sensor.reason);
  world.insert(sensor.value, og.Collider, { shape: SHAPE_BOX, size: { x: 2, y: 1, z: 6 }, sensor: 1 });
  const rng = world.resource(og.Rng);
  if (!rng.ok) throw new Error(rng.reason);
  for (let i = 0; i < 64; i++) {
    const e = world.spawn({
      name: `body-${i}`,
      position: { x: rng.value.range(-4, 4), y: 2 + i * 0.4, z: profile === "2d" ? 0 : rng.value.range(-2, 2) },
    });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, og.RigidBody, { linearDamping: 0.05 });
    world.insert(e.value, og.Collider, i % 2 ? { shape: SHAPE_SPHERE, radius: 0.35, restitution: 0.3 } : { shape: SHAPE_BOX, size: { x: 0.6, y: 0.6, z: 0.6 } });
  }
}

const game = og.game({ tick: 60, seed: 11, resources: [Tally], systems: [Paddle, Count], start: scene }) as unknown as GameToken;
const created = createEngine(game, { profile });
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const hashAtLoad = engine.hash();
const t0 = performance.now();
const advanced = engine.advance(ticks);
const ms = performance.now() - t0;
if (!advanced.ok) throw new Error(advanced.reason);
const dump = engine.probe.dump({ what: "world" }) as { hash: string; entities: number };
const body = engine.world.get(engine.world.find("body-63")!, og.Transform);
const tally = engine.world.resource(Tally);
console.log(
  JSON.stringify({
    host: which,
    profile,
    tick: advanced.value,
    hash: dump.hash,
    hashAtLoad,
    entities: dump.entities,
    body63: body.ok ? body.value.position : null,
    tally: tally.ok ? { entered: tally.value.entered, started: tally.value.started } : null,
    advanceMs: +ms.toFixed(2),
  }),
);
