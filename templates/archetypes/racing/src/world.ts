// The circuit and the cars in the world: the meshes of src/mesh.ts on
// their materials, the sun, the sky, the fog and the post stack, the start
// gantry's lights, and one car per grid slot (its body, glass, trim,
// stripes and lamps on a body that rolls, four wheels that turn and steer,
// its tyre smoke, its sparks and its boost flame).

import { og, math } from "@origingame/engine";
import type { Entity, Material, World } from "@origingame/engine";
import { Car, FIELD, Game } from "./components.ts";
import { Builder, barrierMesh, carGlass, carLamps, carPaint, carStripe, carTrim, gantryMesh, gateMesh, groundMesh, roadMesh, sceneryMesh, standMesh, wheelMesh, WHEELS } from "./mesh.ts";
import { gridSlot, LENGTH, nearest, pointAt, racingLine, SAMPLES, WALL } from "./track.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
export function must(r: { ok: true } | { ok: false; reason: string }): void;
export function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The field's liveries and drivers' names (seat 0 drives slot 0, seat 1 slot 1). */
export const LIVERIES: readonly { name: string; color: { r: number; g: number; b: number } }[] = [
  { name: "You", color: { r: 0.85, g: 0.08, b: 0.06 } },
  { name: "Mara", color: { r: 0.06, g: 0.32, b: 0.85 } },
  { name: "Okafor", color: { r: 0.98, g: 0.72, b: 0.05 } },
  { name: "Lindqvist", color: { r: 0.08, g: 0.62, b: 0.32 } },
  { name: "Tanaka", color: { r: 0.55, g: 0.18, b: 0.78 } },
  { name: "Reyes", color: { r: 0.95, g: 0.45, b: 0.08 } },
];

/** The rivals' pace, each a share of Tuning.rivalPace (slot order). */
const PACE = [1, 1, 1.03, 0.99, 1.01, 0.97];

/** The racing line the rivals drive, and the speed it allows (src/track.ts racingLine). */
export const LINE = racingLine(2.4, 15, 15, 46);

function geometry(world: World, b: Builder) {
  return must(og.render.bufferGeometry(world, { ...b.arrays(), physics: false }));
}

function place(world: World, name: string, b: Builder, material: Material, shadows = true): Entity {
  const e = must(world.spawn({ name }));
  must(world.insert(e, og.Mesh, { geometry: geometry(world, b), material, castShadow: shadows, receiveShadow: true }));
  return e;
}

/** The start gantry's five lamps, red one by one through the countdown, green at GO. */
const LAMPS = 5;

/** Builds the circuit once, in `start`. */
export function buildCircuit(world: World): void {
  const flat = (roughness: number, metalness = 0) => must(og.render.standardMaterial(world, { color: { r: 1, g: 1, b: 1 }, roughness, metalness, flatShading: true }));
  const grid = Array.from({ length: FIELD }, (_, i) => gridSlot(i));
  place(world, "ground", groundMesh(), flat(1), false);
  place(world, "road", roadMesh(LINE.offset, grid), flat(0.85), false);
  place(world, "barriers", barrierMesh(), flat(0.6, 0.1));
  place(world, "scenery", sceneryMesh(), flat(0.95));
  place(world, "stand", standMesh(), flat(0.8));
  place(world, "gantry", gantryMesh(), flat(0.5, 0.4));
  place(world, "gates", gateMesh(), flat(0.6));

  // The gantry's lamps face the grid (the cars come at the line along -z).
  const p = SAMPLES[0]!;
  const lamp = must(og.render.boxGeometry(world, { width: 0.7, height: 0.7, depth: 0.2 }));
  for (let i = 0; i < LAMPS; i++) {
    const material = must(og.render.standardMaterial(world, { color: { r: 0.08, g: 0.05, b: 0.05 }, roughness: 0.4, emissive: { r: 1, g: 0.08, b: 0.04 }, emissiveIntensity: 0 }));
    const e = must(world.spawn({ name: `lamp-${i}`, position: { x: p.x + (i - (LAMPS - 1) / 2) * 1.1, y: 7.6, z: p.z + 0.5 } }));
    must(world.insert(e, og.Mesh, { geometry: lamp, material, castShadow: false }));
  }

  // A clear afternoon: a deep sky, a warm sun from the south-west, a thin haze on the hills.
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.bloomIntensity = 0.25;
  settings.environmentIntensity = 0.7;
  const post = must(world.resource(og.PostEffects));
  Object.assign(post, { ao: true, aoRadius: 0.9, aoIntensity: 0.8, vignette: true, vignetteIntensity: 0.22 });
  const toSun = { x: 0.42, y: 0.62, z: 0.66 };
  const sky = must(world.resource(og.Sky));
  Object.assign(sky, { enabled: true, zenith: { r: 0.12, g: 0.33, b: 0.8 }, horizon: { r: 0.52, g: 0.7, b: 0.93 }, ground: { r: 0.3, g: 0.36, b: 0.3 }, sunDirection: toSun, sunColor: { r: 1, g: 0.95, b: 0.85 }, sunSize: 1, glow: 0.3 });
  const fog = must(world.resource(og.Fog));
  Object.assign(fog, { enabled: true, density: 0.0008, start: 220, maxOpacity: 0.5, sky: 0.05, color: { r: 0.6, g: 0.74, b: 0.92 } });
  const sun = must(world.spawn({ name: "sun", position: { x: 0, y: 60, z: 0 } }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3.4, color: { r: 1, g: 0.96, b: 0.88 }, castShadow: true }));
  must(world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -toSun.x, y: -toSun.y, z: -toSun.z }) }));
}

/** The gantry's lamps: `lit` of them red, or every one green. */
export function setLamps(world: World, lit: number, green: boolean): void {
  for (let i = 0; i < LAMPS; i++) {
    const e = world.find(`lamp-${i}`);
    if (e === null) continue;
    const mesh = must(world.get(e, og.Mesh));
    const on = green || i < lit;
    must(og.render.updateMaterial(world, mesh.material, { emissive: green ? { r: 0.1, g: 1, b: 0.25 } : { r: 1, g: 0.08, b: 0.04 }, emissiveIntensity: on ? 6 : 0 }));
  }
}

/** The heading (src/view.ts's yaw: 0 along -z, growing to the left) of a direction on the ground. */
export function headingOf(tx: number, tz: number): number {
  return math.atan2(-tx, -tz);
}

/** A rotation about +y. */
export function yawQuat(yaw: number) {
  return { x: 0, y: math.sin(yaw / 2), z: 0, w: math.cos(yaw / 2) };
}

/** A car's grid slot: the rivals from pole, the seats' cars at the back (a race is won by passing). */
export function slotOf(index: number, seats: number): number {
  return index < seats ? FIELD - seats + index : index - seats;
}

/** Puts a car on its grid slot, standing, its meter at the start's. */
export function toGrid(world: World, car: Entity, slot: number): void {
  const g = gridSlot(slot);
  const at = pointAt(g.along, g.across);
  const heading = headingOf(at.tx, at.tz);
  must(world.set(car, og.Transform, { position: { x: at.x, y: 0, z: at.z }, quaternion: yawQuat(heading) }));
  must(world.set(car, Car, {
    heading, camYaw: heading, vx: 0, vz: 0, speed: 0, slip: 0, drifting: false, boosting: false, boost: 0.35, steer: 0, throttle: 0, brake: 0, boostHeld: false,
    laps: -1, checkpoint: 2, progress: g.along - LENGTH, along: g.along, lapStart: 0, lastLap: 0, bestLap: 0, checkpoint0: 0, checkpoint1: 0, best0: 0, best1: 0, splitAt: 0, splitDelta: 0, lapAt: 0, sample: nearest(at.x, at.z),
    finish: 0, out: false, place: slot + 1, wrongWay: 0, hits: 0, hitCool: 0, lane: 0,
  }));
}

/** Spawns the field: a car per grid slot, the seats' first (with og.PlayerInput), then the rivals. */
export function spawnCars(world: World, seats: number): Entity[] {
  const paint = carPaint();
  const shapes = {
    paint: geometry(world, paint),
    glass: geometry(world, carGlass()),
    trim: geometry(world, carTrim()),
    stripe: geometry(world, carStripe()),
    front: geometry(world, carLamps(true)),
    rear: geometry(world, carLamps(false)),
    wheel: geometry(world, wheelMesh()),
    flame: must(og.render.coneGeometry(world, { radius: 0.16, height: 0.9, radialSegments: 8 })),
  };
  const mat = (desc: Parameters<typeof og.render.standardMaterial>[1]) => must(og.render.standardMaterial(world, desc));
  const glass = mat({ color: { r: 0.05, g: 0.07, b: 0.1 }, roughness: 0.08, metalness: 0.6, flatShading: true });
  const trim = mat({ color: { r: 0.06, g: 0.06, b: 0.07 }, roughness: 0.55, flatShading: true });
  const white = mat({ color: { r: 0.95, g: 0.95, b: 0.95 }, roughness: 0.35, flatShading: true });
  const front = mat({ color: { r: 1, g: 0.98, b: 0.9 }, roughness: 0.2, emissive: { r: 1, g: 0.95, b: 0.8 }, emissiveIntensity: 1.6 });
  const rear = mat({ color: { r: 0.5, g: 0.02, b: 0.02 }, roughness: 0.3, emissive: { r: 1, g: 0.05, b: 0.03 }, emissiveIntensity: 0.8 });
  const wheel = mat({ color: { r: 1, g: 1, b: 1 }, roughness: 0.7, flatShading: true });
  const flame = mat({ color: { r: 0.4, g: 0.7, b: 1 }, roughness: 1, emissive: { r: 0.35, g: 0.7, b: 1 }, emissiveIntensity: 5, transparent: true, opacity: 0.85 });
  const smoke = must(og.load.emitter(world, "fx/smoke.ogt"));
  const sparks = must(og.load.emitter(world, "fx/sparks.ogt"));
  const cars: Entity[] = [];
  for (let i = 0; i < FIELD; i++) {
    const livery = LIVERIES[i]!;
    const car = must(world.spawn({ name: i < seats ? `car-${i}` : `rival-${i}` }));
    const body = must(world.spawn({ name: `car-${i}-body`, parent: car }));
    must(world.insert(body, og.Mesh, { geometry: shapes.paint, material: mat({ color: livery.color, roughness: 0.32, metalness: 0.25, flatShading: true }), castShadow: true, receiveShadow: true }));
    const part = (name: string, geometry: typeof shapes.paint, material: Material, shadow = true) => {
      const e = must(world.spawn({ name: `car-${i}-${name}`, parent: body }));
      must(world.insert(e, og.Mesh, { geometry, material, castShadow: shadow, receiveShadow: true }));
      return e;
    };
    part("glass", shapes.glass, glass);
    part("trim", shapes.trim, trim);
    part("stripe", shapes.stripe, white, false);
    part("front", shapes.front, front, false);
    part("rear", shapes.rear, rear, false);
    WHEELS.forEach((w, k) => {
      const e = must(world.spawn({ name: `car-${i}-wheel-${k}`, parent: car, position: { x: w[0], y: w[1], z: w[2] } }));
      must(world.insert(e, og.Mesh, { geometry: shapes.wheel, material: wheel, castShadow: true, receiveShadow: true }));
    });
    const fire = must(world.spawn({ name: `car-${i}-flame`, parent: body, position: { x: 0, y: 0.42, z: 2.55 }, quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, Math.PI / 2) }));
    must(world.insert(fire, og.Mesh, { geometry: shapes.flame, material: flame, castShadow: false, visible: false }));
    const puff = must(world.spawn({ name: `car-${i}-smoke`, parent: car, position: { x: 0, y: 0.25, z: 1.4 } }));
    must(world.insert(puff, og.ParticleSystem, { emitter: smoke, seed: 101 + i, visible: true, startTick: 0, stopTick: 1 }));
    const spark = must(world.spawn({ name: `car-${i}-sparks`, parent: car, position: { x: 0, y: 0.5, z: 0 } }));
    must(world.insert(spark, og.ParticleSystem, { emitter: sparks, seed: 201 + i, visible: false }));
    must(world.insert(car, Car, { index: i, seat: i < seats ? i : -1, body: body as unknown as number, pace: PACE[i]! }));
    if (i < seats) must(world.insert(car, og.PlayerInput, { player: i }));
    toGrid(world, car, slotOf(i, seats));
    cars.push(car);
  }
  const g = must(world.resource(Game));
  g.car0 = cars[0] as unknown as number;
  g.car1 = seats > 1 ? (cars[1] as unknown as number) : 0;
  return cars;
}

/** A car's part by name (`body`, `flame`, `smoke`, `sparks`, `wheel-<k>`). */
export function partOf(world: World, index: number, name: string): Entity {
  const e = world.find(`car-${index}-${name}`);
  if (e === null) throw new Error(`car-${index}-${name} is missing`);
  return e;
}

/** How far a car's middle may go from the centre line: the barrier's face less its half-width. */
export const REACH = WALL - 0.95;
