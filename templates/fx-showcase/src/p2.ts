// The phase 2 stations, far from the others along x.
//
//   terrain  a 256 m valley (terrain/valley.ogt: og.terrain.load, og.Terrain
//            and a SHAPE_HEIGHTFIELD collider) with grass, a path, rock on
//            the slopes and sand on the shore; a lake (og.Water with a
//            planar reflection, shadows and caustics) under a mountain; a
//            character (og.CharacterController) walking the path over a knoll

import { og, defineComponent, defineSystem, f32, f64, u32 } from "@origingame/engine";
import { SHAPE_CAPSULE, SHAPE_HEIGHTFIELD, WEATHER_NONE } from "@origingame/engine";
import type { Entity, TerrainData, Vector3 as Vec3, World } from "@origingame/engine";

export const P2_STATIONS = ["terrain"] as const;
export type P2Station = (typeof P2_STATIONS)[number];

/** Where each station lives along x. */
export const P2_ORIGIN: Record<P2Station, number> = { terrain: 3000 };

/** The terrain station's handles and state, on the entity "p2". */
export const P2 = defineComponent("P2", {
  /** Index into P2_STATIONS + 1 (0: another station). */
  station: u32(0),
  /** Ticks since the station was shown. */
  shown: u32(0),
  terrain: f64(0),
  hero: f64(0),
  /** The terrain's height under the hero (og.terrain.sample), for the playtest. */
  ground: f32(0),
  /** The ground's slope there, in degrees. */
  slope: f32(0),
  /** The terrain's height where the hero started. */
  start: f32(0),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Where the hero starts and stops walking along the path (local x). */
const WALK_FROM = -40;
const WALK_TO = -16;
const PATH_Z = 36;

/** Per tick on the terrain station: the hero walks the path east, and the ground under it is sampled. */
export const P2Show = defineSystem("P2Show", { reads: [og.Transform], writes: [P2, og.CharacterController] }, (_q, _time, world) => {
  const e = world.find("p2");
  if (e === null) return;
  const s = must(world.get(e, P2));
  if (s.station === 0) return;
  const x = P2_ORIGIN.terrain;
  const hero = s.hero as Entity;
  const p = must(world.get(hero, og.Transform)).position;
  const walk = p.x < x + WALK_TO ? 2.4 : 0;
  // Follow the path's gentle bend (tools/make-terrain.py: z = 36 + 2.5 sin(x / 9)).
  const dz = (2.5 / 9) * Math.cos((p.x - x) / 9) * walk;
  world.set(hero, og.CharacterController, { move: { x: walk, y: 0, z: dz } });
  const under = must(og.terrain.sample(world, { data: s.terrain as unknown as TerrainData, origin: { x, y: 0, z: 0 }, x: p.x, z: p.z }));
  world.set(e, P2, { shown: s.shown + 1, ground: under.height, slope: under.slope });
});

/** Turns the phase 2 stations off (every station starts from it). */
export function resetP2(world: World): void {
  const e = world.find("p2");
  if (e !== null) world.set(e, P2, { station: 0, shown: 0 });
}

/** Shows a phase 2 station: its world (built the first time), camera, light and atmosphere. */
export function showP2(world: World, station: P2Station): void {
  if (world.find(`p2-${station}`) === null) {
    buildTerrain(world);
    must(world.spawn({ name: `p2-${station}` }));
  }
  const e = world.find("p2");
  if (e !== null) world.set(e, P2, { station: P2_STATIONS.indexOf(station) + 1, shown: 0 });
  const x = P2_ORIGIN.terrain;
  const cam = must(world.resource(og.Camera));
  const eye = { x: x - 38, y: 21, z: 64 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: x - 8, y: 1, z: -16 });
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.bloomIntensity = 0.3;
  settings.environmentIntensity = 0.6;
  const sun = world.find("sun");
  if (sun !== null) {
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.55, y: -0.45, z: 0.7 }) });
    world.set(sun, og.DirectionalLight, { intensity: 3.2 });
  }
  const sky = must(world.resource(og.Sky));
  Object.assign(sky, {
    enabled: true,
    zenith: { r: 0.18, g: 0.34, b: 0.62 },
    horizon: { r: 0.62, g: 0.7, b: 0.78 },
    ground: { r: 0.2, g: 0.2, b: 0.18 },
    sunDirection: { x: -0.55, y: 0.45, z: -0.7 },
    sunSize: 1,
    glow: 0.3,
  });
  const fog = must(world.resource(og.Fog));
  Object.assign(fog, { enabled: true, density: 0.002, start: 60, heightDensity: 0, sky: 0, color: { r: 0.62, g: 0.7, b: 0.78 } });
  must(world.resource(og.Weather)).kind = WEATHER_NONE;
}

/** The valley, its lake and the hero. */
function buildTerrain(world: World): void {
  const x = P2_ORIGIN.terrain;
  const info = must(og.terrain.load(world, "terrain/valley.ogt"));
  const ground = must(world.spawn({ name: "valley", position: { x, y: 0, z: 0 } }));
  world.insert(ground, og.Terrain, { data: info.data });
  world.insert(ground, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry });
  const lake = must(world.spawn({ name: "lake", position: { x: x - 2, y: 2.5, z: -14 } }));
  world.insert(lake, og.Water, {
    size: { x: 64, y: 60 },
    segments: 64,
    waveStrength: 0.15,
    planarReflection: 0.5,
    shadows: 0.8,
    caustics: 1,
  });
  const start: Vec3 = { x: x + WALK_FROM, y: 0, z: PATH_Z + 2.5 * Math.sin(WALK_FROM / 9) };
  start.y = must(og.terrain.sample(world, { data: info.data, origin: { x, y: 0, z: 0 }, x: start.x, z: start.z })).height + 1;
  const hero = must(world.spawn({ name: "terrain-hero", position: start }));
  world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 0.9 });
  world.insert(hero, og.CharacterController, { maxSlope: 45 });
  const body = must(world.spawn({ name: "terrain-hero-body", position: { x: 0, y: 0, z: 0 }, parent: hero }));
  world.set(body, og.Transform, { scale: { x: 0.6, y: 1.5, z: 0.6 } });
  world.insert(body, og.Mesh, { geometry: must(og.render.sphereGeometry(world, { radius: 0.5 })), material: must(og.render.standardMaterial(world, { color: { r: 0.85, g: 0.16, b: 0.1 }, roughness: 0.45 })), castShadow: true, receiveShadow: true });
  const p2 = world.find("p2");
  if (p2 !== null) world.set(p2, P2, { terrain: info.data as unknown as number, hero: hero as unknown as number, start: start.y - 1 });
}
