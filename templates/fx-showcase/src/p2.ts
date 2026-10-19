// The phase 2 render round's stations, far from the others along x, each
// built when first shown (like the E5 round's):
//
//   lights   a night plaza lit by 256 point and spot lights (Forward+:
//            clustered lights on WebGPU and native, the CPU tile table
//            on WebGL2)
//   moon     og.DayNight at 21:00: a waxing moon rising over the rooftops,
//            moonlit, with stars
//   materials every StandardMaterialDesc feature on one board: metalness and
//            roughness rows, emissive, a normal map, clearcoat, alphaTest,
//            transparent glass, a double-sided panel, unlit, a squashed
//            sphere (normals under a non-uniform scale)
//   skinned  the walker walking with a material shader (shaders/ripple.ogt,
//            set on its model's material with og.render.setMaterialShader):
//            rippling and banded, its shadow rippling with it

import { og } from "@origingame/engine";
import type { Asset, ClipAsset, Geometry, Material, SkeletonAsset, Texture, Vector3 as Vec3, World } from "@origingame/engine";

export const P2_STATIONS = ["lights", "moon", "materials", "skinned"] as const;
export type P2Station = (typeof P2_STATIONS)[number];

/** Where each station lives along x. */
export const P2_ORIGIN: Record<P2Station, number> = {
  lights: 1400,
  moon: 1500,
  materials: 1600,
  skinned: 1700,
};

export interface P2Kit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  ground: (x: number, color: Vec3) => void;
  tex: (path: string) => Texture;
  walker: Asset;
  rig: { skeleton: SkeletonAsset; clips: ReadonlyArray<{ clip: ClipAsset; name: string }> };
}

/** The walker's materials while the skinned station gives them its shader. */
let walkerMaterials: Material[] = [];

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** A small integer hash to [0, 1): the same lights on every run. */
function hash01(i: number, salt: number): number {
  let x = Math.imul(i, 0x9e3779b9) ^ Math.imul(salt, 0x85ebca6b);
  x ^= x >>> 16;
  x = Math.imul(x, 0x7feb352d);
  x ^= x >>> 15;
  x = Math.imul(x, 0x846ca68b);
  x ^= x >>> 16;
  return (x >>> 8) / 16777216;
}

/** A saturated colour of hue `h` (0..1), lifted a quarter towards white. */
function hue(h: number): { r: number; g: number; b: number } {
  const k = (n: number) => {
    const v = (n + h * 6) % 6;
    return 1 - Math.min(Math.max(Math.min(v, 4 - v), 0), 1);
  };
  const lift = (c: number) => c + (1 - c) * 0.25;
  return { r: lift(k(5)), g: lift(k(3)), b: lift(k(1)) };
}

/** How many lights the lights station has. */
export const PLAZA_LIGHTS = 256;

let builders: Partial<Record<P2Station, (world: World) => void>> = {};

let k: P2Kit;

/** Prepares the stations; each is built when first shown (showP2). */
export function buildP2(kit: P2Kit): void {
  k = kit;
  builders = {
    lights: (world: World) => {
      const x0 = P2_ORIGIN.lights;
      k.ground(x0, { x: 0.36, y: 0.35, z: 0.33 });
      const stone = k.mat({ color: { r: 0.8, g: 0.78, b: 0.74 }, roughness: 0.55 });
      const chrome = k.mat({ color: { r: 0.92, g: 0.9, b: 0.86 }, metalness: 1, roughness: 0.25 });
      for (let i = 0; i < 100; i++) {
        const gx = i % 10;
        const gz = Math.floor(i / 10);
        const at = { x: x0 - 18 + gx * 4, y: 0, z: -18 + gz * 4 };
        const e = must(world.spawn({ name: `plaza-${i}`, position: (gx + gz) % 2 === 0 ? { ...at, y: 1.2 } : { ...at, y: 0.5 } }));
        if ((gx + gz) % 2 === 0) {
          world.set(e, og.Transform, { scale: { x: 0.5, y: 2.4, z: 0.5 } });
          world.insert(e, og.Mesh, { geometry: k.box, material: stone, receiveShadow: true });
        } else {
          world.insert(e, og.Mesh, { geometry: k.sphere, material: chrome, receiveShadow: true });
        }
      }
      // 16 x 16 lights over the 40 m square: every eighth a spot looking
      // down, the rest points at hashed heights, hues and reaches.
      const side = Math.ceil(Math.sqrt(PLAZA_LIGHTS));
      const spacing = 40 / side;
      for (let i = 0; i < PLAZA_LIGHTS; i++) {
        const x = x0 - 20 + ((i % side) + 0.5 + (hash01(i, 1) - 0.5) * 0.6) * spacing;
        const z = -20 + (Math.floor(i / side) + 0.5 + (hash01(i, 2) - 0.5) * 0.6) * spacing;
        const color = hue(hash01(i, 3));
        if (i % 8 === 7) {
          const at = { x, y: 3.2, z };
          const down = og.transform.lookAt(at, { x: x + (hash01(i, 4) - 0.5), y: 0, z: z + (hash01(i, 5) - 0.5) });
          const e = must(world.spawn({ name: `spot-${i}`, position: at, quaternion: down }));
          world.insert(e, og.SpotLight, { color, intensity: 14, distance: 6, angle: 0.49, penumbra: 0.4 });
        } else {
          const e = must(world.spawn({ name: `lamp-${i}`, position: { x, y: 0.35 + hash01(i, 6) * 0.9, z } }));
          world.insert(e, og.PointLight, { color, intensity: 2.2 + hash01(i, 7) * 1.6, distance: 2.6 + hash01(i, 8) * 1.4 });
        }
      }
    },
    moon: (world: World) => {
      const x0 = P2_ORIGIN.moon;
      k.ground(x0, { x: 0.3, y: 0.32, z: 0.3 });
      const wall = k.mat({ color: { r: 0.62, g: 0.58, b: 0.52 }, roughness: 0.85 });
      const roof = k.mat({ color: { r: 0.36, g: 0.2, b: 0.16 }, roughness: 0.7 });
      // A row of houses on the far side of a square.
      for (let i = 0; i < 7; i++) {
        const h = 2.6 + hash01(i, 11) * 2.2;
        const at = { x: x0 - 9 + i * 3.1, y: h / 2, z: -10 - hash01(i, 12) * 2 };
        const e = must(world.spawn({ name: `house-${i}`, position: at }));
        world.set(e, og.Transform, { scale: { x: 2.8, y: h, z: 3 } });
        world.insert(e, og.Mesh, { geometry: k.box, material: wall, castShadow: true, receiveShadow: true });
        const r = must(world.spawn({ name: `roof-${i}`, position: { x: at.x, y: h + 0.35, z: at.z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 4) }));
        world.set(r, og.Transform, { scale: { x: 1.9, y: 1.9, z: 3.1 } });
        world.insert(r, og.Mesh, { geometry: k.box, material: roof, castShadow: true, receiveShadow: true });
      }
      const well = must(world.spawn({ name: "well", position: { x: x0 + 1, y: 0.5, z: -3 } }));
      world.set(well, og.Transform, { scale: { x: 1.6, y: 1, z: 1.6 } });
      world.insert(well, og.Mesh, { geometry: k.sphere, material: wall, castShadow: true, receiveShadow: true });
    },
    materials: (world: World) => {
      const x0 = P2_ORIGIN.materials;
      k.ground(x0, { x: 0.42, y: 0.43, z: 0.45 });
      const put = (name: string, geometry: Geometry, d: Parameters<typeof og.render.standardMaterial>[1], at: Vec3, extra?: { scale?: Vec3; quaternion?: ReturnType<typeof og.transform.fromAxisAngle> }) => {
        const e = must(world.spawn({ name, position: at, ...(extra?.quaternion ? { quaternion: extra.quaternion } : {}) }));
        if (extra?.scale) world.set(e, og.Transform, { scale: extra.scale });
        world.insert(e, og.Mesh, { geometry, material: k.mat(d), castShadow: true, receiveShadow: true });
      };
      const at = (col: number, row: number, y = 0.5) => ({ x: x0 + col * 1.3 - 2.6, y, z: 1.6 - row * 1.6 });
      const gold = { r: 1, g: 0.78, b: 0.34 };
      for (let i = 0; i < 5; i++) {
        const f = i / 4;
        put(`metal-${i}`, k.sphere, { color: gold, metalness: f, roughness: 0.3 }, at(i, 0));
        put(`rough-${i}`, k.sphere, { color: { r: 0.12, g: 0.3, b: 0.75 }, roughness: 0.05 + 0.95 * f }, at(i, 1));
      }
      put("emissive", k.sphere, { color: { r: 0.05, g: 0.05, b: 0.05 }, emissive: { r: 1, g: 0.45, b: 0.1 }, emissiveIntensity: 3 }, at(0, 2));
      const bumpy = { color: { r: 0.75, g: 0.72, b: 0.68 }, roughness: 0.45, normalMap: k.tex("textures/bumps.png") };
      put("bumps-sphere", k.sphere, bumpy, at(1, 2));
      put("bumps-box", k.box, bumpy, at(2, 2, 0.4), { scale: { x: 0.8, y: 0.8, z: 0.8 }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, 0.6) });
      put("coat-glossy", k.sphere, { color: { r: 0.7, g: 0.06, b: 0.05 }, roughness: 0.75, clearcoat: 1, clearcoatRoughness: 0.03 }, at(3, 2));
      put("coat-rough", k.sphere, { color: { r: 0.7, g: 0.06, b: 0.05 }, roughness: 0.75, clearcoat: 1, clearcoatRoughness: 0.35 }, at(4, 2));
      const panel = must(og.render.planeGeometry(world, { width: 1, height: 1 }));
      put("grille", panel, { map: k.tex("textures/grille.png"), alphaTest: 0.5, side: "double", roughness: 0.6 }, at(0, 3, 0.6));
      put("glass", k.sphere, { color: { r: 0.6, g: 0.9, b: 0.8 }, opacity: 0.35, transparent: true, roughness: 0.05 }, at(1, 3));
      put("two-sided", panel, { map: k.tex("textures/tile.png"), side: "double", roughness: 0.7 }, at(2, 3, 0.6), { quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, 2.6) });
      put("unlit", k.box, { color: { r: 0.95, g: 0.3, b: 0.7 }, unlit: true }, at(3, 3, 0.4), { scale: { x: 0.8, y: 0.8, z: 0.8 }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, 0.4) });
      put("squashed", k.sphere, { color: { r: 0.85, g: 0.85, b: 0.82 }, roughness: 0.2 }, at(4, 3, 0.25), { scale: { x: 1.15, y: 0.45, z: 1.15 } });
    },
    skinned: (world: World) => {
      const x0 = P2_ORIGIN.skinned;
      k.ground(x0, { x: 0.36, y: 0.38, z: 0.34 });
      const walk = k.rig.clips.find((c) => c.name === "walk") ?? k.rig.clips[0]!;
      const walker = must(world.instantiate(k.walker, { name: "rippling-walker", position: { x: x0, y: 0, z: 0 }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, 0.5) }));
      world.insert(walker, og.Skeleton, { asset: k.rig.skeleton, rootMotion: "off" });
      world.insert(walker, og.AnimationPlayer, { clip: walk.clip, speed: 1, loop: true });
      for (const [i, x] of [[0, -2.2], [1, 2.2]] as const) {
        const e = must(world.spawn({ name: `skinned-post-${i}`, position: { x: x0 + x, y: 1, z: -1.2 } }));
        world.set(e, og.Transform, { scale: { x: 0.3, y: 2, z: 0.3 } });
        world.insert(e, og.Mesh, { geometry: k.box, material: k.mat({ color: { r: 0.5, g: 0.5, b: 0.55 }, roughness: 0.8 }), castShadow: true, receiveShadow: true });
      }
    },
  };
}

/**
 * Puts the walker's own look back (the stylised and joints stations draw
 * it): the skinned station gives its materials the ripple shader.
 */
export function resetP2(world: World): void {
  for (const m of walkerMaterials) must(og.render.setMaterialShader(world, m, null));
  walkerMaterials = [];
}

/** Shows a P2 station: builds it the first time, then its camera and look. */
export function showP2(world: World, station: P2Station): void {
  if (world.find(`p2-${station}`) === null) {
    builders[station]?.(world);
    must(world.spawn({ name: `p2-${station}` }));
  }
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const sun = world.find("sun");
  const look = (eye: Vec3, target: Vec3) => {
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, target);
  };
  const x = P2_ORIGIN[station];
  switch (station) {
    case "lights":
      look({ x, y: 22, z: 26 }, { x, y: 0, z: -2 });
      if (sun !== null) world.set(sun, og.DirectionalLight, { intensity: 0 });
      settings.sky = false;
      settings.environmentIntensity = 0.03;
      settings.bloom = true;
      settings.bloomIntensity = 0.3;
      cam.background = { r: 0.003, g: 0.004, b: 0.01 };
      cam.exposure = 1.4;
      break;
    case "materials":
      look({ x: x, y: 7.4, z: 5.4 }, { x: x, y: 0, z: -1.3 });
      if (sun !== null) {
        world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.55, y: -0.6, z: -0.5 }) });
        world.set(sun, og.DirectionalLight, { intensity: 3.2 });
      }
      settings.sky = true;
      settings.environmentIntensity = 0.8;
      settings.bloom = false;
      cam.exposure = 0.7;
      break;
    case "skinned": {
      look({ x: x + 0.4, y: 2.2, z: 5.2 }, { x: x, y: 0.9, z: 0 });
      if (sun !== null) {
        world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.6, y: -0.55, z: -0.35 }) });
        world.set(sun, og.DirectionalLight, { intensity: 3 });
      }
      settings.sky = true;
      settings.environmentIntensity = 0.5;
      settings.bloom = false;
      // The walker's materials draw through the ripple shader while shown.
      const ripple = must(og.load.shader(world, "shaders/ripple.ogt"));
      walkerMaterials = [...must(og.load.gltfInfo(world, k.walker)).materials];
      for (const m of walkerMaterials) {
        must(og.render.setMaterialShader(world, m, ripple));
        must(og.render.setShaderParam(world, m, "bulge", [0.03]));
      }
      break;
    }
    case "moon": {
      // At 21:00 on this path, turned 63.4° about +y, the moon (opposite
      // the sun) stands 38° up straight down -z; the camera looks up at it
      // over the rooftops.
      look({ x, y: 1.7, z: 6 }, { x: x + 1.5, y: 10.6, z: -14 });
      const d = must(world.resource(og.DayNight));
      Object.assign(d, { enabled: true, start: 21, speed: 0, tilt: 30, azimuth: 63.4, lightIntensity: 1.6, tintFog: false, step: 5, moonSize: 2.2, moonPhase: 0.36 });
      settings.sky = true;
      settings.bloom = true;
      settings.bloomIntensity = 0.2;
      cam.exposure = 1.2;
      break;
    }
  }
}
