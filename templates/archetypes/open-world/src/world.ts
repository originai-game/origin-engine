// The isle itself: the terrain and its collider, the endless sea over its
// floor, the sky, the sun and the day running through three days and two
// nights, the fog (darkening with the night), the post stack, the edge of the world, the camp on the
// south beach, the five landmarks (each a building, a standard that rises
// and a beacon that lights when it is claimed), the trees and boulders to
// gather from, the woods, and the bridge across the strait waiting to be
// built.
//
// Where everything stands is src/layout.ts, written by tools/make-assets.py
// beside the heightmap; heights come from the terrain here.

import { og, math, SHAPE_BOX, SHAPE_HEIGHTFIELD, WEATHER_NONE } from "@origingame/engine";
import type { Asset, Entity, TerrainData, Vector3, World } from "@origingame/engine";
import { Game, Kind, Node, Site } from "./components.ts";
import { BRIDGE, CAMP, ISLE_SIZE, ROCKS, SEA, SITES, TREES, WOODS } from "./layout.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
export function must(r: { ok: true } | { ok: false; reason: string }): void;
export function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The hours a day spans: a run starts at 7:00, night falls at 19:30 and the next day breaks at 7:00. */
export const HOURS = { dawn: 7, dusk: 19.5, title: 16.8 };

/** Each landmark's building: its model, scale, collider (metres) and turn. */
const BUILDINGS: readonly { model: string; scale: number; box: Vector3; yaw: number }[] = [
  { model: "building_tower_A_blue", scale: 5, box: { x: 4.6, y: 11, z: 4.6 }, yaw: 0.6 },
  { model: "building_windmill_blue", scale: 7, box: { x: 4.6, y: 8, z: 4.6 }, yaw: -0.9 },
  { model: "building_church_blue", scale: 5, box: { x: 5, y: 8, z: 5.6 }, yaw: 0.2 },
  { model: "building_well_blue", scale: 3.2, box: { x: 2, y: 2.6, z: 2.2 }, yaw: 0.4 },
  { model: "building_tower_B_blue", scale: 6, box: { x: 5.6, y: 15, z: 5.6 }, yaw: 2.6 },
];
/** The standard and the beacon: this far from the building's centre, towards the camp. */
const STANDARD_AT = 5.2;
/** The flag's top when raised, above the ground, and lowered. */
export const FLAG = { up: 4.2, down: 0.6 };

/** The bridge: its deck's width, its planks and how high its deck rides at the middle of the strait. */
const BRIDGE_WIDTH = 2.6;
export const PLANKS = 18;
const ARCH = 1.4;
/** The deck's colliders along the span. */
const DECKS = 6;

/** The terrain's height and normal at a point. */
export function ground(world: World, g: { terrain: number }, x: number, z: number): { height: number; normal: Vector3 } {
  const s = og.terrain.sample(world, { data: g.terrain as unknown as TerrainData, origin: { x: 0, y: 0, z: 0 }, x, z });
  return s.ok ? s.value : { height: 0, normal: { x: 0, y: 1, z: 0 } };
}

/** Where a site's standard and beacon stand: STANDARD_AT from its centre, towards the camp. */
export function standardOf(i: number): { x: number; z: number } {
  const s = SITES[i]!;
  const dx = CAMP.x - s.x;
  const dz = CAMP.z - s.z;
  const d = Math.sqrt(dx * dx + dz * dz);
  return { x: s.x + (dx / d) * STANDARD_AT, z: s.z + (dz / d) * STANDARD_AT };
}

/** Where the bridge's south landing is (its building site). */
export const BRIDGE_SITE = { x: BRIDGE.x, z: BRIDGE.south + 2.5 };

/** The deck's height at a fraction along the span (0 the south landing, 1 the north). */
export function deckY(world: World, g: { terrain: number }, t: number): number {
  const s = ground(world, g, BRIDGE.x, BRIDGE.south).height;
  const n = ground(world, g, BRIDGE.x, BRIDGE.north).height;
  return s + (n - s) * t + ARCH * math.sin(Math.PI * t) + 0.12;
}

/** Builds the isle once, in `start`. */
export function buildIsle(world: World): void {
  const g = must(world.resource(Game));
  const info = must(og.terrain.load(world, "terrain/isle.ogt"));
  const isle = must(world.spawn({ name: "isle", position: { x: 0, y: 0, z: 0 } }));
  must(world.insert(isle, og.Terrain, { data: info.data }));
  must(world.insert(isle, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry }));
  g.terrain = info.data as unknown as number;

  // The sea to the horizon, feeling the isle's floor: waves shoal and break into surf on the beaches.
  const sea = must(world.spawn({ name: "sea", position: { x: 0, y: SEA, z: 0 } }));
  must(world.insert(sea, og.Water, {
    size: { x: ISLE_SIZE, y: ISLE_SIZE },
    segments: 96,
    endless: true,
    seabed: isle,
    wind: { x: 4, y: 6 },
    swell: 0.35,
    choppiness: 0.5,
    shore: 1,
    shoreFoam: 1,
    waveStrength: 0.25,
    reflectivity: 0.5,
    depthFade: 3.2,
    shallow: { r: 0.3, g: 0.78, b: 0.74 },
    deep: { r: 0.02, g: 0.17, b: 0.3 },
    foamWidth: 0.5,
  }));

  // The edge of the world: walls a little inside the terrain's rim, out at sea.
  const e = ISLE_SIZE / 2 - 3;
  const walls: [number, number, number, number][] = [[0, -e, ISLE_SIZE, 2], [0, e, ISLE_SIZE, 2], [-e, 0, 2, ISLE_SIZE], [e, 0, 2, ISLE_SIZE]];
  walls.forEach(([x, z, w, d], i) => {
    const wall = must(world.spawn({ name: `edge-${i}`, position: { x, y: 10, z } }));
    must(world.insert(wall, og.Collider, { shape: SHAPE_BOX, size: { x: w, y: 60, z: d } }));
  });

  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.bloomIntensity = 0.3;
  settings.environmentIntensity = 0.8;
  const post = must(world.resource(og.PostEffects));
  Object.assign(post, {
    ao: true,
    aoRadius: 0.9,
    aoIntensity: 0.8,
    vignette: true,
    vignetteIntensity: 0.22,
    lut: must(og.load.lut(world, "luts/teal-orange.cube")),
    lutIntensity: 0.25,
  });
  const sky = must(world.resource(og.Sky));
  Object.assign(sky, { enabled: true, sunSize: 1.1, glow: 0.4 });
  const fog = must(world.resource(og.Fog));
  Object.assign(fog, { enabled: true, density: 0.0028, start: 60, heightDensity: 0.012, height: 0, falloff: 0.4, maxOpacity: 0.7, sky: 0.25, color: FOG_DAY });
  must(world.resource(og.Weather)).kind = WEATHER_NONE;
  const sun = must(world.spawn({ name: "sun", position: { x: 0, y: 60, z: 0 } }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true }));
  const day = must(world.resource(og.DayNight));
  Object.assign(day, { enabled: true, start: HOURS.title, speed: 0, tilt: 25, azimuth: 20, lightIntensity: 1, tintFog: true, step: 5, moonSize: 1.6, moonPhase: 0.5 });

  const lib = models(world);
  buildCamp(world, g, lib);
  buildSites(world, g, lib);
  buildNodes(world, g, lib);
  buildBridge(world, g, lib);
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
  // The fog's own colour follows the light too, so the far sea darkens with the sky at night.
  const up = smooth(5.5, 8, hour) * (1 - smooth(18.5, 20.5, hour));
  const fog = must(world.resource(og.Fog));
  fog.color = { r: FOG_NIGHT.r + (FOG_DAY.r - FOG_NIGHT.r) * up, g: FOG_NIGHT.g + (FOG_DAY.g - FOG_NIGHT.g) * up, b: FOG_NIGHT.b + (FOG_DAY.b - FOG_NIGHT.b) * up };
}

/** The fog by day and by night. */
const FOG_DAY = { r: 0.72, g: 0.8, b: 0.86 };
const FOG_NIGHT = { r: 0.06, g: 0.09, b: 0.17 };

function smooth(a: number, b: number, x: number): number {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
}

type Models = (name: string) => Asset;

/** The KayKit models (assets/models/kaykit), each loaded once. */
function models(world: World): Models {
  const loaded = new Map<string, Asset>();
  return (name) => {
    let m = loaded.get(name);
    if (m === undefined) {
      m = must(og.load.gltf(world, `models/kaykit/${name}.gltf`));
      loaded.set(name, m);
    }
    return m;
  };
}

function place(world: World, lib: Models, model: string, name: string, at: Vector3, scale: number, yaw: number, parent?: Entity): Entity {
  return must(world.instantiate(lib(model), {
    name,
    ...(parent !== undefined ? { parent } : {}),
    position: at,
    quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw),
    scale: { x: scale, y: scale, z: scale },
  }));
}

function solid(world: World, name: string, at: Vector3, size: Vector3, yaw = 0): Entity {
  const e = must(world.spawn({ name, position: at, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw) }));
  must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size }));
  return e;
}

/** The camp: a tent, a campfire that always burns, the lumber pile. */
function buildCamp(world: World, g: { terrain: number }, lib: Models): void {
  const h = ground(world, g, CAMP.x, CAMP.z).height;
  place(world, lib, "tent", "camp-tent", { x: CAMP.x - 4, y: h, z: CAMP.z + 2 }, 5, 0.5);
  solid(world, "camp-tent-solid", { x: CAMP.x - 4, y: h + 1.2, z: CAMP.z + 2 }, { x: 2.4, y: 2.4, z: 2.4 }, 0.5);
  place(world, lib, "resource_lumber", "camp-lumber", { x: CAMP.x + 4, y: h, z: CAMP.z + 3 }, 4, -0.4);
  const fire = must(world.spawn({ name: "camp-fire", position: { x: CAMP.x, y: h + 0.1, z: CAMP.z } }));
  must(world.insert(fire, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/fire.ogt")), seed: 3 }));
  const ring = place(world, lib, "rock_single_C", "camp-ring", { x: CAMP.x, y: h, z: CAMP.z }, 3, 0);
  must(world.set(ring, og.Model, { tint: { r: 0.6, g: 0.6, b: 0.6 } }));
  const light = must(world.spawn({ name: "camp-light", position: { x: CAMP.x, y: h + 1.4, z: CAMP.z } }));
  must(world.insert(light, og.PointLight, { color: { r: 1, g: 0.6, b: 0.3 }, intensity: 6, distance: 18, castShadow: false }));
}

/** The five landmarks: the building, the standard (a pole, the flag lowered), the brazier, its fire and light. */
function buildSites(world: World, g: { terrain: number }, lib: Models): void {
  const wood = must(og.render.standardMaterial(world, { color: { r: 0.42, g: 0.28, b: 0.16 }, roughness: 0.85 }));
  const iron = must(og.render.standardMaterial(world, { color: { r: 0.14, g: 0.12, b: 0.11 }, roughness: 0.5, metalness: 0.8 }));
  const pole = must(og.render.cylinderGeometry(world, { radiusTop: 0.07, radiusBottom: 0.09, height: FLAG.up + 0.3 }));
  const bowl = must(og.render.cylinderGeometry(world, { radiusTop: 0.5, radiusBottom: 0.3, height: 0.4 }));
  const stand = must(og.render.cylinderGeometry(world, { radiusTop: 0.12, radiusBottom: 0.18, height: 1.1 }));
  const fire = must(og.load.emitter(world, "fx/fire.ogt"));
  const smoke = must(og.load.emitter(world, "fx/smoke.ogt"));
  const burst = must(og.load.emitter(world, "fx/burst.ogt"));
  SITES.forEach((s, i) => {
    const b = BUILDINGS[i]!;
    const h = ground(world, g, s.x, s.z).height;
    const site = must(world.spawn({ name: `site-${i}`, position: { x: s.x, y: h, z: s.z } }));
    must(world.insert(site, Site, { index: i }));
    place(world, lib, b.model, `site-${i}-building`, { x: 0, y: -0.05, z: 0 }, b.scale, b.yaw, site);
    solid(world, `site-${i}-solid`, { x: s.x, y: h + b.box.y / 2, z: s.z }, b.box, b.yaw);
    const at = standardOf(i);
    const sh = ground(world, g, at.x, at.z).height;
    const standard = must(world.spawn({ name: `site-${i}-standard`, position: { x: at.x, y: sh, z: at.z } }));
    const p = must(world.spawn({ name: `site-${i}-pole`, parent: standard, position: { x: 0, y: (FLAG.up + 0.3) / 2, z: 0 } }));
    must(world.insert(p, og.Mesh, { geometry: pole, material: wood, castShadow: true }));
    must(world.insert(standard, og.Collider, { shape: SHAPE_BOX, size: { x: 0.3, y: 2, z: 0.3 }, offset: { x: 0, y: 1, z: 0 } }));
    // The flag: KayKit's pennant, lowered and greyed until the site is claimed.
    const flag = place(world, lib, "flag_blue", `site-${i}-flag`, { x: at.x, y: sh + FLAG.down, z: at.z }, 5, 0);
    must(world.set(flag, og.Model, { tint: { r: 0.45, g: 0.45, b: 0.45 } }));
    // The brazier beside the standard.
    const bx = at.x + 1.3;
    const bh = ground(world, g, bx, at.z).height;
    const brazier = must(world.spawn({ name: `site-${i}-brazier`, position: { x: bx, y: bh, z: at.z } }));
    must(world.insert(brazier, og.Collider, { shape: SHAPE_BOX, size: { x: 0.8, y: 1.3, z: 0.8 }, offset: { x: 0, y: 0.65, z: 0 } }));
    const leg = must(world.spawn({ name: `site-${i}-leg`, parent: brazier, position: { x: 0, y: 0.55, z: 0 } }));
    must(world.insert(leg, og.Mesh, { geometry: stand, material: iron, castShadow: true }));
    const pan = must(world.spawn({ name: `site-${i}-bowl`, parent: brazier, position: { x: 0, y: 1.25, z: 0 } }));
    must(world.insert(pan, og.Mesh, { geometry: bowl, material: iron, castShadow: true }));
    const flame = must(world.spawn({ name: `site-${i}-fire`, parent: brazier, position: { x: 0, y: 1.45, z: 0 } }));
    must(world.insert(flame, og.ParticleSystem, { emitter: fire, seed: 11 + i, visible: false }));
    const plume = must(world.spawn({ name: `site-${i}-smoke`, parent: brazier, position: { x: 0, y: 2.4, z: 0 } }));
    must(world.insert(plume, og.ParticleSystem, { emitter: smoke, seed: 21 + i, visible: false }));
    const sparks = must(world.spawn({ name: `site-${i}-burst`, parent: brazier, position: { x: 0, y: 1.6, z: 0 } }));
    must(world.insert(sparks, og.ParticleSystem, { emitter: burst, seed: 31 + i, visible: false }));
    const light = must(world.spawn({ name: `site-${i}-light`, parent: brazier, position: { x: 0, y: 2.4, z: 0 } }));
    must(world.insert(light, og.PointLight, { color: { r: 1, g: 0.62, b: 0.3 }, intensity: 0, distance: 22, castShadow: false }));
  });
}

/** The trees and boulders to gather from (each with what stays once gathered), and the woods. */
function buildNodes(world: World, g: { terrain: number }, lib: Models): void {
  let index = 0;
  const node = (kind: number, x: number, z: number, yaw: number, model: string, stump: string, scale: number, radius: number, box: Vector3): void => {
    const h = ground(world, g, x, z).height;
    const e = must(world.spawn({ name: `node-${index}`, position: { x, y: h, z } }));
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: box, offset: { x: 0, y: box.y / 2, z: 0 } }));
    const m = place(world, lib, model, `node-${index}-model`, { x, y: h - 0.05, z }, scale, yaw);
    const s = place(world, lib, stump, `node-${index}-stump`, { x, y: h - 0.05, z }, scale, yaw);
    must(world.set(s, og.Model, { visible: false }));
    must(world.insert(e, Node, { index, kind, radius, model: m as unknown as number, stump: s as unknown as number }));
    index++;
  };
  TREES.forEach(([x, z, yaw], i) => {
    const a = i % 2 === 0;
    node(Kind.Tree, x, z, yaw, a ? "tree_single_A" : "tree_single_B", a ? "tree_single_A_cut" : "tree_single_B_cut", 4.6 + (i % 3) * 0.4, 0.5, { x: 0.8, y: 1.4, z: 0.8 });
  });
  ROCKS.forEach(([x, z, yaw], i) => {
    node(Kind.Rock, x, z, yaw, i % 2 === 0 ? "resource_stone" : "rock_single_E", "rock_single_A", 5, 1.0, { x: 1.8, y: 1.2, z: 1.8 });
  });
  WOODS.forEach(([x, z, yaw], i) => {
    const h = ground(world, g, x, z).height;
    place(world, lib, i % 2 === 0 ? "trees_A_medium" : "trees_B_medium", `wood-${i}`, { x, y: h - 0.1, z }, 4, yaw);
    solid(world, `wood-${i}-solid`, { x, y: h + 1.5, z }, { x: 4.6, y: 3, z: 4.6 }, yaw);
  });
}

/** The node's parts. */
export function nodeOf(world: World, i: number): Entity {
  const e = world.find(`node-${i}`);
  if (e === null) throw new Error(`node-${i} is missing`);
  return e;
}

/**
 * The bridge: its planks along the span (hidden until built), its rails,
 * its deck's collider (out of the way under the sea until built) and the
 * lumber and stone waiting at the south landing.
 */
function buildBridge(world: World, g: { terrain: number }, lib: Models): void {
  const plankMat = must(og.render.standardMaterial(world, { color: { r: 0.55, g: 0.38, b: 0.22 }, roughness: 0.9 }));
  const postMat = must(og.render.standardMaterial(world, { color: { r: 0.36, g: 0.24, b: 0.14 }, roughness: 0.9 }));
  const span = BRIDGE.south - BRIDGE.north;
  const step = span / PLANKS;
  const plank = must(og.render.boxGeometry(world, { width: BRIDGE_WIDTH, height: 0.14, depth: step * 0.88 }));
  const post = must(og.render.cylinderGeometry(world, { radiusTop: 0.1, radiusBottom: 0.12, height: 1 }));
  for (let i = 0; i < PLANKS; i++) {
    const t = (i + 0.5) / PLANKS;
    const z = BRIDGE.south - t * span;
    const y = deckY(world, g, t);
    const dy = (deckY(world, g, t + 0.01) - deckY(world, g, t - 0.01)) / (0.02 * span);
    const tilt = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, math.atan(dy));
    const p = must(world.spawn({ name: `bridge-plank-${i}`, position: { x: BRIDGE.x, y: y - 6, z }, quaternion: tilt }));
    must(world.insert(p, og.Mesh, { geometry: plank, material: plankMat, castShadow: true, receiveShadow: true, visible: false }));
    if (i % 3 === 0) {
      for (const side of [-1, 1]) {
        const q = must(world.spawn({ name: `bridge-post-${i}-${side < 0 ? "w" : "e"}`, position: { x: BRIDGE.x + side * (BRIDGE_WIDTH / 2 - 0.1), y: y - 6, z } }));
        must(world.insert(q, og.Mesh, { geometry: post, material: postMat, castShadow: true, visible: false }));
      }
    }
  }
  // The deck's collider and rails: one box per sixth of the span (the arch), parked under the sea until built.
  for (let k = 0; k < DECKS; k++) {
    const t0 = k / DECKS;
    const t1 = (k + 1) / DECKS;
    const y0 = deckY(world, g, t0);
    const y1 = deckY(world, g, t1);
    const len = Math.sqrt((span / DECKS) ** 2 + (y1 - y0) ** 2);
    const pitch = math.atan2(y1 - y0, span / DECKS);
    const e = must(world.spawn({ name: `bridge-deck-${k}`, position: { x: BRIDGE.x, y: -40, z: BRIDGE.south - ((t0 + t1) / 2) * span }, quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, pitch) }));
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: BRIDGE_WIDTH, y: 0.3, z: len + 0.2 } }));
    for (const side of [-1, 1]) {
      const r = must(world.spawn({ name: `bridge-rail-${k}-${side < 0 ? "w" : "e"}`, position: { x: BRIDGE.x + side * (BRIDGE_WIDTH / 2 + 0.1), y: -40, z: BRIDGE.south - ((t0 + t1) / 2) * span }, quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, pitch) }));
      must(world.insert(r, og.Collider, { shape: SHAPE_BOX, size: { x: 0.2, y: 2, z: len + 0.2 } }));
    }
  }
  const h = ground(world, g, BRIDGE_SITE.x, BRIDGE_SITE.z).height;
  place(world, lib, "resource_lumber", "bridge-lumber", { x: BRIDGE_SITE.x - 2.4, y: h, z: BRIDGE_SITE.z + 1 }, 3.4, 0.3);
  place(world, lib, "resource_stone", "bridge-stone", { x: BRIDGE_SITE.x + 2.4, y: h, z: BRIDGE_SITE.z + 1.2 }, 3, 1.1);
}

/** The bridge's parts that rise into place, in order from the south: each plank and the posts beside it. */
export function bridgeParts(world: World, g: { terrain: number }): { entity: Entity; y: number; order: number }[] {
  const out: { entity: Entity; y: number; order: number }[] = [];
  for (let i = 0; i < PLANKS; i++) {
    const y = deckY(world, g, (i + 0.5) / PLANKS);
    const p = world.find(`bridge-plank-${i}`);
    if (p !== null) out.push({ entity: p, y, order: i });
    for (const s of ["w", "e"]) {
      const q = world.find(`bridge-post-${i}-${s}`);
      if (q !== null) out.push({ entity: q, y: y + 0.5, order: i });
    }
  }
  return out;
}

/** The deck's colliders and where each stands once built. */
export function deckParts(world: World, g: { terrain: number }): { entity: Entity; y: number }[] {
  const out: { entity: Entity; y: number }[] = [];
  for (let k = 0; k < DECKS; k++) {
    const y = (deckY(world, g, k / DECKS) + deckY(world, g, (k + 1) / DECKS)) / 2 - 0.15;
    for (const n of [`bridge-deck-${k}`, `bridge-rail-${k}-w`, `bridge-rail-${k}-e`]) {
      const e = world.find(n);
      if (e !== null) out.push({ entity: e, y: n.startsWith("bridge-rail") ? y + 1.1 : y });
    }
  }
  return out;
}

