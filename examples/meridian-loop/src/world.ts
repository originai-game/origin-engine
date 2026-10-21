// The circuit and the cars, built once in `start`:
//
// - models/track.glb (the road and the walls, each its own triangle-mesh
//   collider: the road grips, the walls are slick), models/scenery.glb (the
//   embankments, the neon tubes, the gantry and gates, the skyline) and
//   models/ground.glb (the grid), with a box collider for the ground under
//   the jump's gap;
// - a moon (the shadowed directional light) and a few coloured lamps along
//   the circuit;
// - the player's car: a dynamic chassis with og.Vehicle (a raycast car, its
//   tuning from the Tuning resource) and four wheel models placed each tick
//   from og.physics.vehicleWheels (src/fx.ts);
// - the ghost: a body-less model and four wheels placed each tick from the
//   best lap's recorded poses (src/ghost.ts);
// - the tapes, the tyre smoke, the sparks, the landing dust and the ring of
//   skid-mark decals.

import { og, SHAPE_BOX, SHAPE_TRIMESH } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { Mark, Racer, Tuning } from "./components.ts";
import { spawnTapes } from "./tapes.ts";
import { at, SAMPLES, WIDTH } from "./track.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
export function must(r: { ok: true } | { ok: false; reason: string }): void;
export function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export function entity(world: World, name: string): Entity {
  const e = world.find(name);
  if (e === null) throw new Error(`${name} is missing`);
  return e;
}

/** Collision layers: the circuit, the car. */
export const LAYER = { track: 1, car: 2 } as const;

/** Skid marks kept on the road at once (og.Decal draws at most 32). */
export const MARKS = 28;

/** Samples behind the line the car waits on the grid. */
export const GRID_BACK = 5;

/** A yaw (about +y) as a quaternion. */
export function yaw(angle: number): { x: number; y: number; z: number; w: number } {
  return { x: 0, y: Math.sin(angle / 2), z: 0, w: Math.cos(angle / 2) };
}

/** The pose on the circuit k samples past the line: on the road's centre, facing along it, a little above. */
export function poseAt(k: number, lift = 0.45): { position: { x: number; y: number; z: number }; quaternion: { x: number; y: number; z: number; w: number } } {
  const s = at(k);
  // Facing +z along the tangent: the yaw that turns +z onto (tx, tz).
  const heading = Math.atan2(s.tx, s.tz);
  return { position: { x: s.x, y: s.y + lift, z: s.z }, quaternion: yaw(heading) };
}

export function buildCircuit(world: World): void {
  const track = must(og.load.gltf(world, "models/track.glb", { castShadow: false, receiveShadow: true }));
  must(world.instantiate(track, { name: "track" }));
  const info = must(og.load.gltfInfo(world, track));
  // Mesh 0 is the road, mesh 1 the walls (tools/circuit.ts).
  info.geometries.forEach((geo, i) => {
    const c = must(world.spawn({ name: i === 0 ? "road" : "walls" }));
    must(world.insert(c, og.Collider, { shape: SHAPE_TRIMESH, mesh: geo, layers: LAYER.track, friction: i === 0 ? 1 : 0.05, restitution: i === 0 ? 0 : 0.1 }));
  });
  must(world.instantiate(must(og.load.gltf(world, "models/scenery.glb", { castShadow: true, receiveShadow: true })), { name: "scenery" }));
  must(world.instantiate(must(og.load.gltf(world, "models/ground.glb", { receiveShadow: true })), { name: "ground" }));
  // The ground as a body: what a car falling short of the jump lands on.
  const floor = must(world.spawn({ name: "floor", position: { x: 100, y: -0.53, z: 0 } }));
  must(world.insert(floor, og.Collider, { shape: SHAPE_BOX, size: { x: 1600, y: 1, z: 1600 }, layers: LAYER.track, friction: 0.9 }));

  // The moon: high behind the start straight, a cool light that casts the cars' shadows.
  const moon = must(world.spawn({ name: "moon", position: { x: 0, y: 60, z: 0 }, quaternion: { x: -0.42, y: 0.28, z: 0.14, w: 0.85 } }));
  must(world.insert(moon, og.DirectionalLight, { color: { r: 0.62, g: 0.66, b: 1 }, intensity: 1.6, castShadow: true, shadowDistance: 90, shadowSoftness: 0.02, cascadeBlend: 0.15 }));
  // Lamps along the circuit: warm on the straights, magenta at the hairpin, cyan at the gantry.
  const lamps: [number, number, number, number][] = [
    [0, 1, 0.3, 0.85],
    [70, 1, 0.2, 0.7],
    [150, 0.3, 0.9, 1],
    [235, 1, 0.65, 0.3],
    [300, 1, 0.65, 0.3],
    [370, 1, 0.15, 0.6],
    [430, 0.4, 0.8, 1],
  ];
  lamps.forEach(([k, r, g, b], i) => {
    const s = at(k);
    const e = must(world.spawn({ name: `lamp-${i}`, position: { x: s.x - s.rx * (WIDTH / 2 + 3), y: s.y + 7, z: s.z - s.rz * (WIDTH / 2 + 3) } }));
    must(world.insert(e, og.PointLight, { color: { r, g, b }, intensity: 60, distance: 38 }));
  });
}

export function spawnCars(world: World): void {
  const t = must(world.resource(Tuning));
  const grid = poseAt(-GRID_BACK);
  const car = must(world.spawn({ name: "car", position: grid.position, quaternion: grid.quaternion }));
  must(world.insert(car, og.RigidBody, { mass: 1000, angularDamping: 0.4, ccd: 1 }));
  // A flat chassis box: its centre (the centre of mass) low between the wheels, so the car corners without rolling over.
  must(world.insert(car, og.Collider, { shape: SHAPE_BOX, size: { x: 1.7, y: 0.25, z: 4 }, friction: 0.05, restitution: 0.1, layers: LAYER.car, mask: LAYER.track }));
  must(world.insert(car, og.Velocity, {}));
  must(world.insert(car, og.Vehicle, {
    wheelBase: 2.6,
    track: 1.7,
    wheelRadius: 0.35,
    mountHeight: -0.3,
    suspensionRest: 0.3,
    suspensionTravel: 0.18,
    suspensionStiffness: 50,
    suspensionDamping: 4.5,
    grip: t.grip,
    engineForce: t.engineForce,
    brakeForce: t.brakeForce,
    maxSpeed: t.maxSpeed,
    maxSteer: t.maxSteer,
  }));
  must(world.insert(car, og.Model, { asset: must(og.load.gltf(world, "models/car.glb", { castShadow: true })) }));
  must(world.insert(car, Racer, {}));
  const wheel = must(og.load.gltf(world, "models/wheel.glb", { castShadow: true }));
  for (let i = 0; i < 4; i++) {
    const w = must(world.spawn({ name: `wheel-${i}`, position: grid.position }));
    must(world.insert(w, og.Model, { asset: wheel }));
  }
  // Head and tail light pools: a spot ahead would cost a shadow map; a point light under the nose is enough at night.
  const nose = must(world.spawn({ name: "headlight", parent: car, position: { x: 0, y: 1.6, z: 5 } }));
  must(world.insert(nose, og.PointLight, { color: { r: 0.85, g: 0.9, b: 1 }, intensity: 8, distance: 14 }));

  // The ghost: no body, drawn where the best lap's car was (src/ghost.ts); parked under the world until a lap starts.
  const ghost = must(world.spawn({ name: "ghost", position: { x: 0, y: -50, z: 0 } }));
  must(world.insert(ghost, og.Model, { asset: must(og.load.gltf(world, "models/ghost.glb")) }));
  must(world.insert(ghost, Racer, { ghost: true }));
  const gw = must(og.load.gltf(world, "models/ghost-wheel.glb"));
  for (let i = 0; i < 4; i++) {
    const w = must(world.spawn({ name: `ghost-wheel-${i}`, position: { x: 0, y: -50, z: 0 } }));
    must(world.insert(w, og.Model, { asset: gw }));
  }
  spawnTapes(world);
}

export function spawnEffects(world: World): void {
  const smoke = must(og.load.emitter(world, "fx/smoke.ogt"));
  for (const side of ["l", "r"]) {
    const e = must(world.spawn({ name: `smoke-${side}`, position: { x: 0, y: -50, z: 0 } }));
    // Not emitting until a slide starts (stopTick before startTick).
    must(world.insert(e, og.ParticleSystem, { emitter: smoke, seed: side === "l" ? 11 : 12, startTick: 0, stopTick: 1 }));
  }
  const sparks = must(world.spawn({ name: "sparks", position: { x: 0, y: -50, z: 0 } }));
  must(world.insert(sparks, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/sparks.ogt")), seed: 21, startTick: 0, stopTick: 1 }));
  const dust = must(world.spawn({ name: "dust", position: { x: 0, y: -50, z: 0 } }));
  must(world.insert(dust, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/dust.ogt")), seed: 31, startTick: 0, stopTick: 1 }));
  // The skid marks: a ring of decals laid behind the rear wheels while they slide (src/fx.ts).
  const rs = must(world.resource(og.RenderSettings));
  rs.decalAtlas = must(og.load.texture(world, "textures/skid.png"));
  for (let i = 0; i < MARKS; i++) {
    const e = must(world.spawn({ name: `mark-${i}`, position: { x: 0, y: -60, z: 0 }, scale: { x: 0.32, y: 1.2, z: 0.6 } }));
    must(world.insert(e, og.Decal, { opacity: 0, angleFade: 80 }));
    must(world.insert(e, Mark, { slot: i }));
  }
}

/** Puts the car on the grid (or k samples past the line), still. */
export function placeCar(world: World, k: number): void {
  const car = entity(world, "car");
  const p = poseAt(k);
  must(world.set(car, og.Transform, { position: p.position, quaternion: p.quaternion }));
  must(world.set(car, og.Velocity, { linear: { x: 0, y: 0, z: 0 }, angular: { x: 0, y: 0, z: 0 } }));
  must(world.set(car, og.Vehicle, { throttle: 0, steer: 0, brake: 1 }));
  must(world.set(car, Racer, { steer: 0, steerIn: 0, throttleIn: 0, brakeIn: 0, air: 0 }));
}

void SAMPLES;
