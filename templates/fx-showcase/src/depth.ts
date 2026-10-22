// The depth wave's render stations (docs/depth.md §6, stream S6), far from
// the others along x, built when first shown:
//
//   lod          2,000 trees (models/tree.glb, cooked into three levels of
//                detail: 3,160, 1,580 and 790 triangles) on a 40 by 50
//                grid, seen from one end: the renderer draws each at the
//                level its size on screen calls for (RenderSettings.lodBias
//                1); `set:RenderSettings.lodBias=0` draws every one whole,
//                and `debug_view:lod` colours them by level.
//   reflections  a closed room lit by two lamps and a strip of daylight: a
//                chrome sphere and a gold one on a wet, mirror-smooth floor,
//                an og.ReflectionProbe filling the room so they reflect the
//                walls and not the sky, and PostEffects.ssr laying the
//                frame's own reflection on the floor (the probe on WebGL2).

import { og } from "@origingame/engine";
import type { Geometry, Material, Vector3 as Vec3, World } from "@origingame/engine";

export const DEPTH_STATIONS = ["lod", "reflections"] as const;
export type DepthStation = (typeof DEPTH_STATIONS)[number];

/** Where each station lives along x. */
export const DEPTH_ORIGIN: Record<DepthStation, number> = { lod: 3000, reflections: 3500 };

/** The tree field's size. */
export const TREE_COLUMNS = 40;
export const TREE_ROWS = 50;
const SPACING = 4;

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export interface DepthKit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
}

let kit: DepthKit;

/** Prepares the stations; each is built when first shown (showDepth). */
export function buildDepth(k: DepthKit): void {
  kit = k;
}

function put(world: World, name: string, geometry: Geometry, d: Parameters<typeof og.render.standardMaterial>[1], at: Vec3, scale: Vec3, shadows = true) {
  const e = must(world.spawn({ name, position: at }));
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material: kit.mat(d), castShadow: shadows, receiveShadow: true });
  return e;
}

function buildField(world: World): void {
  const x0 = DEPTH_ORIGIN.lod;
  const tree = must(og.load.gltf(world, "models/tree.glb"));
  put(world, "lod-ground", kit.box, { color: { r: 0.22, g: 0.3, b: 0.14 }, roughness: 1 }, { x: x0, y: -0.05, z: -100 }, { x: 200, y: 0.1, z: 230 }, false);
  // A fixed hash, not og.rng: the layout is the same for any seed.
  const hash = (i: number) => {
    const h = Math.sin(i * 12.9898 + 78.233) * 43758.5453;
    return h - Math.floor(h);
  };
  for (let row = 0; row < TREE_ROWS; row++) {
    for (let col = 0; col < TREE_COLUMNS; col++) {
      const i = row * TREE_COLUMNS + col;
      const x = x0 + (col - TREE_COLUMNS / 2 + 0.5) * SPACING + (hash(i) - 0.5) * 2;
      const z = -6 - row * SPACING + (hash(i + 7919) - 0.5) * 2;
      const s = 0.8 + 0.4 * hash(i + 104729);
      const e = must(world.instantiate(tree, { name: `tree-${i}`, position: { x, y: 0, z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, hash(i + 31) * 6.28) }));
      world.set(e, og.Transform, { scale: { x: s, y: s, z: s } });
    }
  }
}

function buildRoom(world: World): void {
  const x0 = DEPTH_ORIGIN.reflections;
  const wall = (name: string, color: { r: number; g: number; b: number }, at: Vec3, scale: Vec3, emissive = 0) =>
    put(world, `refl-${name}`, kit.box, { color, roughness: 0.8, ...(emissive > 0 ? { emissive: color, emissiveIntensity: emissive } : {}) }, { x: x0 + at.x, y: at.y, z: at.z }, scale);
  // The room: 10 m wide, 4 m high, 10 m deep, closed (the sky never reaches in).
  wall("back", { r: 0.75, g: 0.18, b: 0.12 }, { x: 0, y: 2, z: -5 }, { x: 10, y: 4, z: 0.2 });
  wall("left", { r: 0.12, g: 0.35, b: 0.75 }, { x: -5, y: 2, z: 0 }, { x: 0.2, y: 4, z: 10 });
  wall("right", { r: 0.85, g: 0.8, b: 0.65 }, { x: 5, y: 2, z: 0 }, { x: 0.2, y: 4, z: 10 });
  wall("front", { r: 0.85, g: 0.8, b: 0.65 }, { x: 0, y: 2, z: 5 }, { x: 10, y: 4, z: 0.2 });
  wall("ceiling", { r: 0.9, g: 0.9, b: 0.88 }, { x: 0, y: 4.1, z: 0 }, { x: 10.2, y: 0.2, z: 10.2 });
  wall("strip", { r: 1, g: 0.95, b: 0.85 }, { x: 0, y: 3.95, z: -1 }, { x: 6, y: 0.08, z: 0.6 }, 6);
  wall("panel", { r: 0.95, g: 0.7, b: 0.25 }, { x: -4.85, y: 2.2, z: -2 }, { x: 0.05, y: 1.4, z: 2.4 }, 3);
  // The wet floor: dark, mirror-smooth tiles of a slightly rougher grout.
  put(world, "refl-floor", kit.box, { color: { r: 0.05, g: 0.05, b: 0.06 }, roughness: 0.04 }, { x: x0, y: -0.1, z: 0 }, { x: 10, y: 0.2, z: 10 }, false);
  put(world, "refl-chrome", kit.sphere, { color: { r: 0.95, g: 0.95, b: 0.95 }, metalness: 1, roughness: 0 }, { x: x0 - 1.1, y: 1, z: 0 }, { x: 2, y: 2, z: 2 });
  put(world, "refl-gold", kit.sphere, { color: { r: 1, g: 0.78, b: 0.34 }, metalness: 1, roughness: 0.2 }, { x: x0 + 1.4, y: 0.6, z: 0.8 }, { x: 1.2, y: 1.2, z: 1.2 });
  put(world, "refl-block", kit.box, { color: { r: 0.2, g: 0.6, b: 0.3 }, roughness: 0.5 }, { x: x0 + 2.6, y: 0.6, z: -2.4 }, { x: 1.2, y: 1.2, z: 1.2 });
  for (const [name, x, z, c] of [["lamp-a", -3, -3, { r: 1, g: 0.75, b: 0.45 }], ["lamp-b", 3, 2, { r: 0.6, g: 0.75, b: 1 }]] as const) {
    const lamp = must(world.spawn({ name: `refl-${name}`, position: { x: x0 + x, y: 3.2, z } }));
    world.insert(lamp, og.PointLight, { color: c, intensity: 30, distance: 12, castShadow: false });
  }
  const probe = must(world.spawn({ name: "refl-probe", position: { x: x0, y: 1.6, z: 0 } }));
  world.insert(probe, og.ReflectionProbe, { size: { x: 10, y: 4.2, z: 10 }, resolution: 128, mode: "baked", blendDistance: 0.5 });
}

/** Puts the frame's level of detail and reflections back (every station's show calls it). */
export function resetDepth(world: World): void {
  const settings = must(world.resource(og.RenderSettings));
  settings.lodBias = 1;
  const post = must(world.resource(og.PostEffects));
  post.ssr = false;
}

/** Shows a station: builds it the first time, then its camera and light. */
export function showDepth(world: World, station: DepthStation): void {
  if (world.find(`depth-${station}`) === null) {
    if (station === "lod") buildField(world);
    else buildRoom(world);
    must(world.spawn({ name: `depth-${station}` }));
  }
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const post = must(world.resource(og.PostEffects));
  const sun = world.find("sun");
  const x = DEPTH_ORIGIN[station];
  const look = (eye: Vec3, target: Vec3) => {
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, target);
  };
  if (station === "lod") {
    look({ x, y: 7, z: 4 }, { x, y: 1, z: -60 });
    settings.sky = true;
    settings.environmentIntensity = 0.5;
    if (sun !== null) {
      world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.7, z: -0.5 }) });
      world.set(sun, og.DirectionalLight, { intensity: 3 });
    }
    return;
  }
  look({ x: x + 0.2, y: 1.7, z: 4.6 }, { x: x - 0.2, y: 0.8, z: -1 });
  // The sky lights the room's IBL only where the probe does not reach: the
  // room is closed, and the sun is off inside it.
  settings.sky = true;
  settings.environmentIntensity = 1;
  settings.bloom = true;
  settings.bloomIntensity = 0.3;
  if (sun !== null) world.set(sun, og.DirectionalLight, { intensity: 0 });
  post.ssr = true;
  post.ssrMaxDistance = 12;
  post.ssrThickness = 0.3;
  post.ssrSteps = 48;
  post.ssrFade = 0.15;
}
