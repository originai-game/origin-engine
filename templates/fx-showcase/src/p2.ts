// The phase 2 render round's stations, far from the others along x, each
// built when first shown (like the E5 round's):
//
//   lights   a night plaza lit by 256 point and spot lights (Forward+:
//            clustered lights on WebGPU and native, the CPU tile table
//            on WebGL2)
//   moon     og.DayNight at 21:00: a waxing moon rising over the rooftops,
//            moonlit, with stars

import { og } from "@origingame/engine";
import type { Geometry, Material, Vector3 as Vec3, World } from "@origingame/engine";

export const P2_STATIONS = ["lights", "moon"] as const;
export type P2Station = (typeof P2_STATIONS)[number];

/** Where each station lives along x. */
export const P2_ORIGIN: Record<P2Station, number> = {
  lights: 1400,
  moon: 1500,
};

export interface P2Kit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  ground: (x: number, color: Vec3) => void;
}

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

/** Prepares the stations; each is built when first shown (showP2). */
export function buildP2(k: P2Kit): void {
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
  };
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
