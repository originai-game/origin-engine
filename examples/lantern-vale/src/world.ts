// The vale itself: the terrain and its collider, the sun and its dusk, the
// trees, the pile of rocks the hero can shove, the four lanterns (a
// lacquered voxel post and cage, an emissive glow, a light, a burst of
// sparks) and the far side's pines, ruined arch and banner, which stream in
// with the `far-side` group of scenes/main.ogt as the camera nears them.
//
// Keep LANTERNS, START and the terrain's numbers in step with
// tools/make-assets.ts.

import { og, SHAPE_CYLINDER, SHAPE_HEIGHTFIELD, SHAPE_SPHERE } from "@origingame/engine";
import type { Asset, Entity, TerrainData, World } from "@origingame/engine";
import { Game, Lantern, Rock } from "./components.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
export function must(r: { ok: true } | { ok: false; reason: string }): void;
export function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The lanterns (x, z): the fourth stands on the far side. */
export const LANTERNS: readonly { x: number; z: number }[] = [
  { x: -22, z: 8 },
  { x: 20, z: 12 },
  { x: -16, z: -14 },
  { x: 14, z: -30 },
];
/** Where a run starts: the south end of the valley. */
export const START = { x: 0, z: 26 };
/** The collision layers: the lanterns are found by og.physics.overlap on their own layer. */
export const LAYER = { world: 1, lantern: 2, rock: 4, hero: 8 } as const;
/** The hours a run spans: the sun low at the start, setting (its shadows going) past 17, night at the end. */
export const HOURS = { start: 15.5, night: 19.5, title: 16.2, dawn: 6.6 };
/** The pile of rocks: its centre on the meadow, beside the path north. */
export const PILE = { x: 4.5, z: 10 };

const TREES: readonly [number, number, number][] = [
  [-8, 22, 0.9], [9, 24, 1.1], [-14, 18, 1], [15, 19, 0.85], [-27, 16, 1.2], [27, 20, 1], [-30, 2, 1.1],
  [-6, 4, 0.8], [10, 2, 1], [28, -2, 1.15], [-26, -8, 0.95], [-4, -6, 1.05], [24, -10, 0.9], [-31, -18, 1.2],
  [6, 18, 0.75], [-18, 26, 1],
];
const PINES: readonly [number, number, number][] = [
  [-6, -24, 1], [0, -30, 1.2], [-12, -28, 0.9], [22, -22, 1.1], [26, -32, 1], [4, -36, 1.15], [-20, -32, 1.05], [18, -38, 0.95],
];

/** The terrain's height at (x, z). */
export function groundY(world: World, g: { terrain: number }, x: number, z: number): number {
  const s = og.terrain.sample(world, { data: g.terrain as unknown as TerrainData, x, z });
  return s.ok ? s.value.height : 0;
}

/** A model placed on the ground, turned about y, uniformly scaled. */
function place(world: World, model: Asset, name: string, x: number, y: number, z: number, yaw: number, scale: number): Entity {
  return must(world.instantiate(model, { name, position: { x, y, z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw), scale: { x: scale, y: scale, z: scale } }));
}

/** Builds the vale once, in `start`. The look (looks/lantern-vale.ogt) already set the frame's treatment. */
export function buildVale(world: World): void {
  const g = must(world.resource(Game));
  const info = must(og.terrain.load(world, "terrain/vale.ogt"));
  const vale = must(world.spawn({ name: "vale", position: { x: 0, y: 0, z: 0 } }));
  must(world.insert(vale, og.Terrain, { data: info.data }));
  must(world.insert(vale, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry, layers: LAYER.world }));
  g.terrain = info.data as unknown as number;

  const sun = must(world.spawn({ name: "sun", position: { x: 0, y: 40, z: 0 } }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 2.4, castShadow: true }));
  const day = must(world.resource(og.DayNight));
  Object.assign(day, { enabled: true, start: HOURS.title, speed: 0, tilt: 25, azimuth: 0, lightIntensity: 1, tintFog: true });

  // Trees: static trunks.
  const tree = must(og.load.gltf(world, "models/tree.vox"));
  TREES.forEach(([x, z, s], i) => {
    const e = place(world, tree, `tree-${i}`, x, groundY(world, g, x, z) - 0.05, z, i * 1.7, s);
    must(world.insert(e, og.Collider, { shape: SHAPE_CYLINDER, radius: 0.3 * s, height: 2, offset: { x: 0, y: 1, z: 0 }, layers: LAYER.world }));
  });

  // The pile: ten boulders the hero's controller shoves about (pushBodies).
  const rock = must(og.load.gltf(world, "models/rock.vox"));
  const spots: [number, number, number][] = [[-1, 0, -1], [0, 0, -1], [1, 0, -1], [-1, 0, 0], [0, 0, 0], [1, 0, 0], [-0.5, 0, 1], [0.5, 0, 1], [-0.5, 1, -0.5], [0.5, 1, 0]];
  spots.forEach(([dx, dy, dz], i) => {
    const x = PILE.x + dx * 1.05;
    const z = PILE.z + dz * 1.05;
    const y = groundY(world, g, x, z) + 0.02 + dy * 0.75;
    const e = place(world, rock, `rock-${i}`, x, y, z, i * 2.3, 1);
    must(world.insert(e, og.RigidBody, { linearDamping: 0.8, angularDamping: 0.8 }));
    must(world.insert(e, og.Velocity));
    must(world.insert(e, og.Collider, { shape: SHAPE_SPHERE, radius: 0.42, offset: { x: 0, y: 0.38, z: 0 }, density: 900, friction: 0.9, layers: LAYER.rock }));
    must(world.insert(e, Rock, { x0: x, z0: z }));
  });

  buildLanterns(world, g);
  buildFarSide(world, g);
}

/** The four lanterns: a lacquered voxel model on a solid on the lantern layer, a glow, a light and a burst. */
function buildLanterns(world: World, g: { terrain: number }): void {
  const model = must(og.load.gltf(world, "models/lantern.vox"));
  const glowGeo = must(og.render.sphereGeometry(world, { radius: 0.17, widthSegments: 12, heightSegments: 8 }));
  const burst = must(og.load.emitter(world, "fx/burst.ogt"));
  LANTERNS.forEach((at, i) => {
    const y = groundY(world, g, at.x, at.z) - 0.05;
    const e = place(world, model, `lantern-${i}`, at.x, y, at.z, 0, 1);
    must(world.insert(e, og.Collider, { shape: SHAPE_CYLINDER, radius: 0.38, height: 2.6, offset: { x: 0, y: 1.3, z: 0 }, layers: LAYER.lantern }));
    must(world.insert(e, Lantern, { index: i }));
    // Each glow its own material, so one lit leaves the others dark.
    const mat = must(og.render.standardMaterial(world, { color: { r: 0.25, g: 0.18, b: 0.08 }, roughness: 0.4, emissive: { r: 1, g: 0.62, b: 0.22 }, emissiveIntensity: 0 }));
    const glow = must(world.spawn({ name: `lantern-${i}-glow`, parent: e, position: { x: 0, y: 2.0, z: 0 } }));
    must(world.insert(glow, og.Mesh, { geometry: glowGeo, material: mat, castShadow: false }));
    const light = must(world.spawn({ name: `lantern-${i}-light`, parent: e, position: { x: 0, y: 2.0, z: 0 } }));
    must(world.insert(light, og.PointLight, { color: { r: 1, g: 0.62, b: 0.28 }, intensity: 0, distance: 11, castShadow: false }));
    const sparks = must(world.spawn({ name: `lantern-${i}-burst`, parent: e, position: { x: 0, y: 2.1, z: 0 } }));
    must(world.insert(sparks, og.ParticleSystem, { emitter: burst, seed: 31 + i, visible: false }));
  });
}

/** The far side: its models and banner belong to the far-side streaming group (scenes/main.ogt). */
function buildFarSide(world: World, g: { terrain: number }): void {
  const pine = must(og.load.gltf(world, "models/pine.vox"));
  PINES.forEach(([x, z, s], i) => {
    const e = place(world, pine, `pine-${i}`, x, groundY(world, g, x, z) - 0.05, z, i * 0.9, s);
    must(world.insert(e, og.Collider, { shape: SHAPE_CYLINDER, radius: 0.25 * s, height: 2, offset: { x: 0, y: 1, z: 0 }, layers: LAYER.world }));
  });
  const ruins = must(og.load.gltf(world, "models/ruins.vox"));
  const rx = 2;
  const rz = -22;
  place(world, ruins, "ruins", rx, groundY(world, g, rx, rz) - 0.1, rz, 0.25, 1);
  // A banner hung in the arch: a textured plane (a grey 1x1 until the group is resident).
  const banner = must(og.load.texture(world, "textures/banner.png"));
  const cloth = must(og.render.standardMaterial(world, { map: banner, roughness: 0.9, side: "double" }));
  const plane = must(og.render.planeGeometry(world, { width: 1.2, height: 1.9 }));
  const b = must(world.spawn({ name: "banner", position: { x: rx - 0.2, y: groundY(world, g, rx, rz) + 2.2, z: rz + 0.1 }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, 0.25) }));
  must(world.insert(b, og.Mesh, { geometry: plane, material: cloth, castShadow: true }));
}

/**
 * The models' materials, once their glTFs have loaded (true when all are
 * done): the voxel models are cel-shaded one by one (toon), the lanterns
 * lacquered (clearcoat) and the slimes left glassy (their glTF's
 * transmission). The look cannot do the toon: MaterialStyle.toon reaches
 * every lit material that leaves toon at its default, and a toon material
 * ignores clearcoat and transmission (FINDINGS.md).
 */
export function styleModels(world: World): boolean {
  let done = true;
  const each = (path: string, desc: Parameters<typeof og.render.updateMaterial>[2]) => {
    const info = og.load.gltfInfo(world, must(og.load.gltf(world, path)));
    if (!info.ok) {
      done = false;
      return;
    }
    for (const m of info.value.materials) must(og.render.updateMaterial(world, m, desc));
  };
  for (const path of ["models/hero.vox", "models/tree.vox", "models/rock.vox", "models/pine.vox", "models/ruins.vox"]) each(path, { toon: true });
  each("models/lantern.vox", { roughness: 0.55, clearcoat: 1, clearcoatRoughness: 0.06 });
  return done;
}

/** Sets the time of day og.DayNight is held at (the game moves it, so a pause holds it). */
export function setHour(world: World, hour: number): void {
  const day = must(world.resource(og.DayNight));
  day.speed = 0;
  day.start = hour;
}

/** The lantern's parts by name. */
export function part(world: World, i: number, name: string): Entity {
  const e = world.find(`lantern-${i}-${name}`);
  if (e === null) throw new Error(`lantern-${i}-${name} is missing`);
  return e;
}

/** Puts every lantern out (a new run or the title). */
export function douseLanterns(world: World): void {
  for (let i = 0; i < LANTERNS.length; i++) {
    const e = world.find(`lantern-${i}`);
    if (e === null) continue;
    must(world.set(e, Lantern, { lit: false, litAt: 0, glow: 0 }));
    must(world.set(part(world, i, "burst"), og.ParticleSystem, { visible: false }));
    setGlow(world, i, 0);
  }
}

/** A lantern's glass and light at `glow` (0 dark, 1 full). */
export function setGlow(world: World, i: number, glow: number): void {
  must(world.set(part(world, i, "light"), og.PointLight, { intensity: glow * 7 }));
  const mesh = world.get(part(world, i, "glow"), og.Mesh);
  if (mesh.ok) must(og.render.updateMaterial(world, mesh.value.material, { emissiveIntensity: glow * 6 }));
}
