// The arena itself: the outpost's landing field inside a ring of canyon
// walls (assets/scenes/arena.ogt, written by tools/make-assets.py from its
// LAYOUT: KayKit's Space Base Bits for the cover, each with a box
// collider), the deck under it, the dusk light (a low sun, the sky, a haze,
// the lamp posts' cold lights and the gates' red glow), the post stack, the
// foes' navmesh (assets/nav/arena.ogt, baked from the same scene), and the
// effect pools a shot draws from: impact sparks, coolant sprays, bullet
// marks on the walls and the tracers.

import { og, math, SHAPE_BOX } from "@origingame/engine";
import type { Entity, Quaternion, Vector3, World } from "@origingame/engine";
import { Game, MARKS } from "./components.ts";
import { creation } from "./creation.ts";
import { ISOMETRIC } from "./view.ts";

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

/** A named point of the arena's scene (spawn-<i>, gate-<i>, duel-<i>). */
export function point(world: World, name: string): Vector3 {
  return must(world.get(entity(world, name), og.Transform)).position;
}

/** The gates the foes come through (gate-<i> in the scene): north, west, east. */
export const GATES = 3;
/** The lamp posts (tools/make-assets.py LAMPS): their lights hang 2 m up. */
const LAMPS: readonly [number, number][] = [[-4.6, -16.8], [4.6, -16.8], [-16.8, -4.6], [-16.8, 4.6], [16.8, -4.6], [16.8, 4.6], [-4.2, 16.4], [4.2, 16.4]];
/** The gates' openings (tools/make-assets.py GATES): a red lintel light over each. */
const GATE_AT: readonly [number, number][] = [[0, -21], [-21, 0], [21, 0]];

/** The decal atlas's cells (textures/marks.png). */
export const ATLAS = {
  hole: { uvMin: { x: 0, y: 0 }, uvMax: { x: 0.5, y: 0.5 } },
  scorch: { uvMin: { x: 0.5, y: 0 }, uvMax: { x: 1, y: 0.5 } },
  stripes: { uvMin: { x: 0, y: 0.5 }, uvMax: { x: 0.5, y: 1 } },
  chevron: { uvMin: { x: 0.5, y: 0.5 }, uvMax: { x: 1, y: 1 } },
} as const;

/** Impact pools: sparks off the arena, coolant off a foe, and the downed foes' bursts. */
export const SPARKS = 6;
export const SPRAYS = 6;
export const POPS = 4;

/** -90° about +x: a decal's +z (its projection) turned onto the floor's +y. */
export const DOWN: Quaternion = { x: -Math.SQRT1_2, y: 0, z: 0, w: Math.SQRT1_2 };

/** a × b: b's rotation, then a's. */
export function qmul(a: Quaternion, b: Quaternion): Quaternion {
  return {
    x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
    y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
    z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
    w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
  };
}

/** A turn of `a` radians about +y. */
export function yawQ(a: number): Quaternion {
  return { x: 0, y: math.sin(a / 2), z: 0, w: math.cos(a / 2) };
}

/** Builds the arena once, in `start`. */
/** The outpost's skyline beyond the ring (tools/make-assets.py SKYLINE). */
const SKYLINE = ["hab-s", "hab-sw", "tower-far-ne", "tower-far-w", "depot-far-e", "truck", "rocks-far-0", "rocks-far-1"];

/**
 * Isometric: the canyon ring and the skyline on the camera's side are not
 * drawn (their colliders stay), as a twin-stick cuts away the near walls,
 * so the field is never behind a 6 m block.
 */
function cutAway(world: World): void {
  const yaw = ((creation.camera as { yaw?: number }).yaw ?? 45) * (Math.PI / 180);
  const toward = { x: math.sin(yaw), z: math.cos(yaw) };
  const names = [...SKYLINE];
  for (let i = 0; world.find(`wall-${i}`) !== null; i++) names.push(`wall-${i}`);
  for (let i = 0; world.find(`gate-back-${i}`) !== null; i++) names.push(`gate-back-${i}`);
  for (const name of names) {
    const e = world.find(name);
    if (e === null) continue;
    const p = must(world.get(e, og.Transform)).position;
    if (p.x * toward.x + p.z * toward.z > 6) must(world.set(e, og.Model, { visible: false }));
  }
}

export function buildArena(world: World): void {
  const g = must(world.resource(Game));
  must(world.instantiate(must(og.load.scene(world, "scenes/arena.ogt"))));
  g.nav = must(og.nav.load(world, "nav/arena.ogt")).mesh as unknown as number;
  if (ISOMETRIC) cutAway(world);

  // The deck: 4 m panels over the floor's collider, out past the canyon ring.
  const deck = must(og.render.planeGeometry(world, { width: 90, height: 90, uvRepeat: { x: 22.5, y: 22.5 } }));
  const plate = must(og.render.standardMaterial(world, { map: must(og.load.texture(world, "textures/floor.png")), roughness: 0.86, metalness: 0.1 }));
  const floor = must(world.spawn({ name: "deck", position: { x: 0, y: 0, z: 0 }, quaternion: DOWN }));
  must(world.insert(floor, og.Mesh, { geometry: deck, material: plate, castShadow: false, receiveShadow: true }));

  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.bloomIntensity = 0.45;
  settings.environmentIntensity = 0.55;
  settings.decalAtlas = must(og.load.texture(world, "textures/marks.png"));
  const post = must(world.resource(og.PostEffects));
  Object.assign(post, { ao: true, aoRadius: 0.9, aoIntensity: 0.9, vignette: true, vignetteIntensity: 0.32, filmGrain: 0.03, godRays: true, godRaysIntensity: 0.25 });
  const sky = must(world.resource(og.Sky));
  Object.assign(sky, { enabled: true, sunSize: 1.4, glow: 0.6 });
  const fog = must(world.resource(og.Fog));
  // The isometric camera stands 60 m off (src/view.ts), so its haze starts past that: the field stays crisp.
  Object.assign(fog, { enabled: true, density: 0.004, start: ISOMETRIC ? 75 : 30, heightDensity: 0.01, height: 0.3, falloff: 0.6, maxOpacity: 0.45, sky: 0.2, color: { r: 0.62, g: 0.42, b: 0.38 } });
  const day = must(world.resource(og.DayNight));
  // Dusk over the outpost, held: the sun low in the west-south-west, warm on the cover's faces.
  Object.assign(day, { enabled: true, start: 17.7, speed: 0, tilt: 25, azimuth: 35, lightIntensity: 1.1, tintFog: true, step: 5 });
  const sun = must(world.spawn({ name: "sun", position: { x: 0, y: 40, z: 0 } }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true }));

  LAMPS.forEach(([x, z], i) => {
    const lamp = must(world.spawn({ name: `lamp-light-${i}`, position: { x, y: 2.1, z } }));
    must(world.insert(lamp, og.PointLight, { color: { r: 0.45, g: 0.75, b: 1 }, intensity: 7, distance: 9, castShadow: false }));
  });
  const lintel = must(og.render.boxGeometry(world, { width: 6, height: 0.35, depth: 0.35 }));
  const red = must(og.render.standardMaterial(world, { color: { r: 0.2, g: 0.03, b: 0.02 }, emissive: { r: 1, g: 0.12, b: 0.04 }, emissiveIntensity: 3 }));
  GATE_AT.forEach(([x, z], i) => {
    const across = x === 0;
    const q = yawQ(across ? 0 : Math.PI / 2);
    const bar = must(world.spawn({ name: `gate-lintel-${i}`, position: { x, y: 4.6, z }, quaternion: q }));
    must(world.insert(bar, og.Mesh, { geometry: lintel, material: red, castShadow: false }));
    const glow = must(world.spawn({ name: `gate-light-${i}`, position: { x: x * 0.92, y: 3.2, z: z * 0.92 } }));
    must(world.insert(glow, og.PointLight, { color: { r: 1, g: 0.2, b: 0.08 }, intensity: 9, distance: 10, castShadow: false }));
    // The gate's chevron on the deck, pointing in.
    const yaw = math.atan2(-x, -z);
    const mark = must(world.spawn({ name: `gate-mark-${i}`, position: { x: x * 0.8, y: 0, z: z * 0.8 }, quaternion: qmul(yawQ(yaw), DOWN) }));
    must(world.set(mark, og.Transform, { scale: { x: 2.4, y: 2.4, z: 1 } }));
    must(world.insert(mark, og.Decal, { ...ATLAS.chevron, color: { r: 1, g: 0.5, b: 0.3 }, opacity: 0.8, emissive: { r: 0.8, g: 0.15, b: 0.05 } }));
  });
  // Hazard stripes round the core and along the pad's edge.
  [[0, 3.2, 0], [0, -3.2, 0], [3.2, 0, 1], [-3.2, 0, 1]].forEach(([x, z, turn], i) => {
    const q = qmul(yawQ((turn! * Math.PI) / 2), DOWN);
    const e = must(world.spawn({ name: `stripes-${i}`, position: { x: x!, y: 0, z: z! }, quaternion: q }));
    must(world.set(e, og.Transform, { scale: { x: 3.2, y: 0.9, z: 1 } }));
    must(world.insert(e, og.Decal, { ...ATLAS.stripes, color: { r: 1, g: 1, b: 1 }, opacity: 0.75 }));
  });

  // The edge of the world, past the ring, for a trooper who jumps the walls anyway.
  ([[0, -30, 70, 2], [0, 30, 70, 2], [-30, 0, 2, 70], [30, 0, 2, 70]] as const).forEach(([x, z, w, d], i) => {
    const e = must(world.spawn({ name: `edge-${i}`, position: { x, y: 10, z } }));
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: w, y: 40, z: d } }));
  });

  buildPools(world);
}

function buildPools(world: World): void {
  const pool = (name: string, path: string, n: number, seed: number) => {
    const emitter = must(og.load.emitter(world, path));
    for (let i = 0; i < n; i++) {
      const e = must(world.spawn({ name: `${name}-${i}`, position: { x: 0, y: -50, z: 0 } }));
      must(world.insert(e, og.ParticleSystem, { emitter, seed: seed + i }));
    }
  };
  pool("sparks", "fx/sparks.ogt", SPARKS, 100);
  pool("spray", "fx/oil.ogt", SPRAYS, 200);
  pool("pop", "fx/pop.ogt", POPS, 300);
  pool("smoke", "fx/smoke.ogt", POPS, 400);
  pool("gatefx", "fx/gate.ogt", 3, 500);
  for (let i = 0; i < MARKS; i++) {
    const e = must(world.spawn({ name: `mark-${i}`, position: { x: 0, y: -50, z: 0 } }));
    must(world.set(e, og.Transform, { scale: { x: 0.22, y: 0.22, z: 0.3 } }));
    must(world.insert(e, og.Decal, { ...ATLAS.hole, color: { r: 1, g: 1, b: 1 }, opacity: 0.9, angleFade: 75, visible: false }));
  }
}

/** The rotation that turns +y (an emitter's direction) onto `n`. */
export function upTo(n: Vector3): Quaternion {
  const d = n.y;
  if (d > 0.9999) return { x: 0, y: 0, z: 0, w: 1 };
  if (d < -0.9999) return { x: 1, y: 0, z: 0, w: 0 };
  // axis = y × n, angle = acos(y · n), as a half-angle quaternion.
  const ax = n.z, az = -n.x;
  const w = 1 + d;
  const len = Math.sqrt(ax * ax + az * az + w * w);
  return { x: ax / len, y: 0, z: az / len, w: w / len };
}

/** Restarts pooled effect `name-<i>` at a point, turned to a direction. */
export function burst(world: World, name: string, i: number, at: Vector3, up: Vector3 = { x: 0, y: 1, z: 0 }): void {
  const e = entity(world, `${name}-${i}`);
  must(world.set(e, og.Transform, { position: at, quaternion: upTo(up) }));
  must(world.set(e, og.ParticleSystem, { startTick: world.tick }));
}

/** Clears every bullet mark (a new run). */
export function clearMarks(world: World): void {
  for (let i = 0; i < MARKS; i++) must(world.set(entity(world, `mark-${i}`), og.Decal, { visible: false }));
}
