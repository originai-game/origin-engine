// The vale itself: the terrain and its collider, the lake, the sky, the sun
// and its day running down to night, the fog, the post stack, the edge of
// the world and the four beacons on their knolls (a plinth, an iron bowl,
// the fire, smoke and embers waiting to be lit, a light, a scorch mark on
// the ground and a rune on the plinth's face towards the lake).
//
// Keep the numbers in step with tools/make-assets.py (BEACONS, LAKE, the
// knolls' heights) and assets/nav/vale.ogt (the plinths' boxes).

import { og, SHAPE_BOX, SHAPE_HEIGHTFIELD, WEATHER_NONE } from "@origingame/engine";
import type { Entity, Quaternion, TerrainData, Vector3, World } from "@origingame/engine";
import { Beacon, BEACONS, Game } from "./components.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
export function must(r: { ok: true } | { ok: false; reason: string }): void;
export function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The beacons' knolls (x, z), clockwise from the north-west. */
export const BEACON_AT: readonly { x: number; z: number }[] = [
  { x: -34, z: -30 },
  { x: 34, z: -32 },
  { x: 36, z: 26 },
  { x: -36, z: 24 },
];
/** The lake's centre and its surface. */
export const LAKE = { x: 0, z: -6 };
export const WATER_Y = 1;
/** Where a run starts: the south end of the spur path. */
export const START = { x: 0, z: 36 };
/** The plinth (also the navmesh's box): its size, and its top above the knoll's height. */
export const PLINTH = { width: 1.6, height: 1.2, top: 1 };
/** The bowl on the plinth. */
const BOWL = { width: 1.1, height: 0.34 };
/** The hours a run spans: the light is low at the start and night falls at the end. */
export const HOURS = { start: 15.5, night: 19.5, title: 16.6 };

/** The decal atlas's regions (textures/decals.png). */
const ATLAS = {
  scorch: { uvMin: { x: 0, y: 0 }, uvMax: { x: 0.5, y: 0.5 } },
  rune: { uvMin: { x: 0.5, y: 0 }, uvMax: { x: 1, y: 0.5 } },
} as const;

const DOWN: Quaternion = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2);

/** The terrain's height and normal at a point. */
export function ground(world: World, g: { terrain: number }, x: number, z: number): { height: number; normal: Vector3 } {
  const s = og.terrain.sample(world, { data: g.terrain as unknown as TerrainData, origin: { x: 0, y: 0, z: 0 }, x, z });
  return s.ok ? s.value : { height: 0, normal: { x: 0, y: 1, z: 0 } };
}

/** Builds the vale once, in `start`. */
export function buildVale(world: World): void {
  const g = must(world.resource(Game));
  const info = must(og.terrain.load(world, "terrain/vale.ogt"));
  const vale = must(world.spawn({ name: "vale", position: { x: 0, y: 0, z: 0 } }));
  must(world.insert(vale, og.Terrain, { data: info.data }));
  must(world.insert(vale, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry }));
  g.terrain = info.data as unknown as number;
  g.nav = must(og.nav.load(world, "nav/vale.ogt")).mesh as unknown as number;

  const lake = must(world.spawn({ name: "lake", position: { x: LAKE.x, y: WATER_Y, z: LAKE.z } }));
  must(world.insert(lake, og.Water, {
    size: { x: 34, y: 34 },
    segments: 64,
    waveStrength: 0.12,
    waveScale: 0.8,
    planarReflection: 0.5,
    reflectivity: 0.45,
    shadows: 0.8,
    caustics: 1,
    depthFade: 1.2,
    shallow: { r: 0.42, g: 0.7, b: 0.66 },
    deep: { r: 0.04, g: 0.14, b: 0.17 },
    foamWidth: 0.3,
  }));

  // The edge of the world: walls beyond the cliffs' foot, for a hero who climbs them anyway.
  const walls: [number, number, number, number][] = [[0, -52, 110, 2], [0, 52, 110, 2], [-52, 0, 2, 110], [52, 0, 2, 110]];
  walls.forEach(([x, z, w, d], i) => {
    const e = must(world.spawn({ name: `edge-${i}`, position: { x, y: 10, z } }));
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: w, y: 40, z: d } }));
  });

  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.bloomIntensity = 0.35;
  settings.environmentIntensity = 0.7;
  settings.decalAtlas = must(og.load.texture(world, "textures/decals.png"));
  const post = must(world.resource(og.PostEffects));
  Object.assign(post, {
    ao: true,
    aoRadius: 0.8,
    aoIntensity: 0.9,
    godRays: true,
    godRaysIntensity: 0.35,
    vignette: true,
    vignetteIntensity: 0.3,
    lut: must(og.load.lut(world, "luts/teal-orange.cube")),
    lutIntensity: 0.35,
    filmGrain: 0.035,
  });
  const sky = must(world.resource(og.Sky));
  Object.assign(sky, { enabled: true, sunSize: 1.2, glow: 0.45 });
  const fog = must(world.resource(og.Fog));
  Object.assign(fog, { enabled: true, density: 0.0035, start: 30, heightDensity: 0.02, height: 1, falloff: 0.6, maxOpacity: 0.75, sky: 0.15, color: { r: 0.7, g: 0.62, b: 0.55 } });
  must(world.resource(og.Weather)).kind = WEATHER_NONE;
  const sun = must(world.spawn({ name: "sun", position: { x: 0, y: 40, z: 0 } }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true }));
  const day = must(world.resource(og.DayNight));
  Object.assign(day, { enabled: true, start: HOURS.title, speed: 0, tilt: 20, azimuth: 0, lightIntensity: 1, tintFog: true, step: 5 });

  buildBeacons(world, g.terrain);
}

/**
 * Sets the time of day. og.DayNight counts unscaled ticks (a pause would not
 * hold it), so the clock is held here (speed 0) and moved by the game: the
 * Ui system sets it from the seconds played each tick.
 */
export function setHour(world: World, hour: number): void {
  const day = must(world.resource(og.DayNight));
  day.speed = 0;
  day.start = hour;
}

function buildBeacons(world: World, terrain: number): void {
  const stone = must(og.render.standardMaterial(world, { color: { r: 0.42, g: 0.4, b: 0.37 }, roughness: 0.92 }));
  const iron = must(og.render.standardMaterial(world, { color: { r: 0.12, g: 0.1, b: 0.09 }, roughness: 0.55, metalness: 0.8 }));
  const coals = must(og.render.standardMaterial(world, { color: { r: 0.05, g: 0.04, b: 0.04 }, roughness: 1, emissive: { r: 1, g: 0.3, b: 0.05 }, emissiveIntensity: 0 }));
  const plinth = must(og.render.boxGeometry(world, { width: PLINTH.width, height: PLINTH.height, depth: PLINTH.width }));
  const bowl = must(og.render.boxGeometry(world, { width: BOWL.width, height: BOWL.height, depth: BOWL.width }));
  const bed = must(og.render.boxGeometry(world, { width: BOWL.width - 0.2, height: 0.04, depth: BOWL.width - 0.2 }));
  const fire = must(og.load.emitter(world, "fx/fire.ogt"));
  const smoke = must(og.load.emitter(world, "fx/smoke.ogt"));
  const burst = must(og.load.emitter(world, "fx/burst.ogt"));
  BEACON_AT.forEach((at, i) => {
    const h = ground(world, { terrain }, at.x, at.z).height;
    const y = h + PLINTH.top - PLINTH.height / 2;
    const b = must(world.spawn({ name: `beacon-${i}`, position: { x: at.x, y, z: at.z } }));
    must(world.insert(b, og.Mesh, { geometry: plinth, material: stone, castShadow: true, receiveShadow: true }));
    must(world.insert(b, og.Collider, { shape: SHAPE_BOX, size: { x: PLINTH.width, y: PLINTH.height, z: PLINTH.width } }));
    const top = PLINTH.height / 2;
    must(world.insert(b, Beacon, { index: i, top: y + top + BOWL.height }));
    const child = (name: string, position: Vector3, quaternion?: Quaternion): Entity => must(world.spawn({ name: `beacon-${i}-${name}`, parent: b, position, ...(quaternion ? { quaternion } : {}) }));
    const pan = child("bowl", { x: 0, y: top + BOWL.height / 2, z: 0 });
    must(world.insert(pan, og.Mesh, { geometry: bowl, material: iron, castShadow: true, receiveShadow: true }));
    must(world.insert(pan, og.Collider, { shape: SHAPE_BOX, size: { x: BOWL.width, y: BOWL.height, z: BOWL.width } }));
    // Each bowl's coals have their own material, so one glowing leaves the others dark.
    const glow = i === 0 ? coals : must(og.render.standardMaterial(world, { color: { r: 0.05, g: 0.04, b: 0.04 }, roughness: 1, emissive: { r: 1, g: 0.3, b: 0.05 }, emissiveIntensity: 0 }));
    const embers = child("coals", { x: 0, y: top + BOWL.height + 0.02, z: 0 });
    must(world.insert(embers, og.Mesh, { geometry: bed, material: glow }));
    const flame = child("fire", { x: 0, y: top + BOWL.height + 0.05, z: 0 });
    must(world.insert(flame, og.ParticleSystem, { emitter: fire, seed: 11 + i, visible: false }));
    const plume = child("smoke", { x: 0, y: top + BOWL.height + 1, z: 0 });
    must(world.insert(plume, og.ParticleSystem, { emitter: smoke, seed: 21 + i, visible: false }));
    const sparks = child("burst", { x: 0, y: top + BOWL.height + 0.2, z: 0 });
    must(world.insert(sparks, og.ParticleSystem, { emitter: burst, seed: 31 + i, visible: false }));
    const light = child("light", { x: 0, y: top + BOWL.height + 1.2, z: 0 });
    must(world.insert(light, og.PointLight, { color: { r: 1, g: 0.56, b: 0.24 }, intensity: 0, distance: 16, castShadow: false }));
    // The scorch on the knoll, projected down; shown once the beacon burns.
    const scorch = child("scorch", { x: 0, y: h - y, z: 0 }, DOWN);
    must(world.set(scorch, og.Transform, { scale: { x: 4.6, y: 4.6, z: 2.4 } }));
    must(world.insert(scorch, og.Decal, { ...ATLAS.scorch, color: { r: 0.25, g: 0.2, b: 0.18 }, opacity: 0.85, visible: false }));
    // The rune on the face towards the lake: dark stone, then glowing once lit.
    const toLake = { x: LAKE.x - at.x, z: LAKE.z - at.z };
    const face = Math.abs(toLake.x) > Math.abs(toLake.z) ? { x: Math.sign(toLake.x), z: 0 } : { x: 0, z: Math.sign(toLake.z) };
    const yaw = Math.atan2(face.x, face.z);
    const rune = child("rune", { x: (face.x * PLINTH.width) / 2, y: 0.05, z: (face.z * PLINTH.width) / 2 }, og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw));
    must(world.set(rune, og.Transform, { scale: { x: 1, y: 1, z: 0.5 } }));
    must(world.insert(rune, og.Decal, { ...ATLAS.rune, color: { r: 0.3, g: 0.27, b: 0.24 }, opacity: 0.9, angleFade: 80 }));
  });
}

/** The beacon's parts, by name. */
export function part(world: World, i: number, name: string): Entity {
  const e = world.find(`beacon-${i}-${name}`);
  if (e === null) throw new Error(`beacon-${i}-${name} is missing`);
  return e;
}

/** Puts every beacon out (a new run or the title). */
export function douseBeacons(world: World): void {
  for (let i = 0; i < BEACONS; i++) {
    const b = world.find(`beacon-${i}`);
    if (b === null) continue;
    must(world.set(b, Beacon, { lit: false, litAt: 0 }));
    for (const p of ["fire", "smoke", "burst"]) must(world.set(part(world, i, p), og.ParticleSystem, { visible: false }));
    must(world.set(part(world, i, "light"), og.PointLight, { intensity: 0 }));
    must(world.set(part(world, i, "scorch"), og.Decal, { visible: false }));
    must(world.set(part(world, i, "rune"), og.Decal, { color: { r: 0.3, g: 0.27, b: 0.24 }, emissive: { r: 0, g: 0, b: 0 } }));
    setCoals(world, i, 0);
  }
}

/** Lights beacon i: the fire, the smoke, one burst of embers, the light, the scorch and the rune. */
export function lightBeacon(world: World, i: number, time: number): void {
  const b = world.find(`beacon-${i}`);
  if (b === null) return;
  must(world.set(b, Beacon, { lit: true, litAt: time }));
  const tick = world.tick;
  for (const p of ["fire", "smoke", "burst"]) must(world.set(part(world, i, p), og.ParticleSystem, { visible: true, startTick: tick }));
  must(world.set(part(world, i, "scorch"), og.Decal, { visible: true }));
  must(world.set(part(world, i, "rune"), og.Decal, { color: { r: 1, g: 0.75, b: 0.45 }, emissive: { r: 2.2, g: 0.9, b: 0.25 } }));
  setCoals(world, i, 3);
}

function setCoals(world: World, i: number, glow: number): void {
  const mesh = world.get(part(world, i, "coals"), og.Mesh);
  if (mesh.ok) must(og.render.updateMaterial(world, mesh.value.material, { emissiveIntensity: glow }));
}
