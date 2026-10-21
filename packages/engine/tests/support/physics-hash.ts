// One host per process: a physics scene (a static floor, a ramp, 64 boxes and
// spheres dropped from Rng-chosen poses, a kinematic paddle moved by a
// system, a sensor counting entries) advanced --ticks fixed steps on the host
// named by --host. Prints JSON { host, tick, hash, ... } for hosts.test.ts.
//
//   bun tests/support/physics-hash.ts --host node|web|lanes [--ticks 600] [--profile 3d|2d]
//       [--bodies N] [--snapshot] [--threads N] [--module 3d|2d] [--restore <file>]
//
// --bodies: the dropped bodies, 64 by default; past 64 each further 64 drop
// the same way beside the first, 9 m further out on alternate sides (1,024
// is a world whose step goes to the lanes pool: og-physics's grain is 500
// bodies on two to four lanes).
//
// lanes: the lanes module of the profile (wasm atomics and the publish
// members: dist/web-publish for 3d, dist/web-publish-2d for 2d, one module
// per physics profile since owed row 71; --module takes the other profile's,
// which refuses the world), on --threads Workers when given
// (wasm-bindgen-rayon's pool), else on the engine thread alone. A publish
// module has no world dump, so its hash is empty and --snapshot (the
// world's snapshot, which every host writes alike) is what is compared.
// --restore: the native host restores that snapshot after loading the scene
// and prints the restored world's hash instead of stepping.

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
const threads = argv.includes("--threads") ? Number(opt("--threads", "0")) : undefined;
const restore = argv.includes("--restore") ? opt("--restore", "") : "";
const bodies = Number(opt("--bodies", "64"));
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
let lanesReport: (() => string) | null = null;

if (which === "node") {
  const { loadNativeHost } = await import("../../src/node/index.ts");
  loadNativeHost();
} else if (which === "lanes") {
  const dir = join(pkg, "dist", opt("--module", profile) === "2d" ? "web-publish-2d" : "web-publish");
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(dir, "origin-engine-lanes.js"));
  await loadWebHost(glue, readFileSync(join(dir, "origin-engine-lanes_bg.wasm")), threads);
  lanesReport = glue.og_web_lanes;
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
  // 200 m across: the paddle really sweeps since the kinematic rule (a
  // kinematic body with a Velocity moves by it), and a body it flings must
  // still land on the floor for the settle check below.
  world.insert(floor.value, og.Collider, { shape: SHAPE_BOX, size: { x: 200, y: 1, z: 200 } });
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
  for (let i = 0; i < bodies; i++) {
    const g = Math.floor(i / 64);
    const out = Math.ceil(g / 2) * 9 * (g % 2 ? 1 : -1);
    const e = world.spawn({
      name: `body-${i}`,
      position: { x: rng.value.range(-4, 4) + out, y: 2 + (i % 64) * 0.4, z: profile === "2d" ? 0 : rng.value.range(-2, 2) },
    });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, og.RigidBody, { linearDamping: 0.05 });
    world.insert(e.value, og.Collider, i % 2 ? { shape: SHAPE_SPHERE, radius: 0.35, restitution: 0.3 } : { shape: SHAPE_BOX, size: { x: 0.6, y: 0.6, z: 0.6 } });
  }
}

const game = og.game({ tick: 60, seed: 11, resources: [Tally], systems: [Paddle, Count], start: scene }) as unknown as GameToken;
const created = createEngine(game, { profile });
if (!created.ok) {
  // The refusal itself is the answer when the module lacks the profile.
  console.log(JSON.stringify({ host: which, profile, refused: created.reason }));
  process.exit(0);
}
const engine = created.value;
if (restore) {
  const r = engine.world.restore(readFileSync(restore, "utf8"));
  if (!r.ok) throw new Error(r.reason);
  console.log(JSON.stringify({ host: which, profile, restored: { tick: engine.world.tick, hash: engine.hash() } }));
  process.exit(0);
}
const publish = which === "lanes";
const hashAtLoad = publish ? "" : engine.hash();
const t0 = performance.now();
const advanced = engine.advance(ticks);
const ms = performance.now() - t0;
if (!advanced.ok) throw new Error(advanced.reason);
const dump = publish ? { hash: "", entities: 0 } : (engine.probe.dump({ what: "world" }) as { hash: string; entities: number });
const snapshot = argv.includes("--snapshot") ? engine.world.snapshot() : null;
if (snapshot && !snapshot.ok) throw new Error(snapshot.reason);
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
    ...(snapshot ? { snapshot: snapshot.value } : {}),
    ...(lanesReport ? { lanes: JSON.parse(lanesReport()) } : {}),
  }),
);
// The lanes glue's wasm-bindgen-rayon snippet listens for a pool Worker's
// init message on the global scope, which keeps bun's event loop alive.
process.exit(0);
