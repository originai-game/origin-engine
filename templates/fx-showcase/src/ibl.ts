// The phase 4 image-based lighting station, far from the others along x,
// built when first shown:
//
//   environment  spheres from mirror chrome to matte on a plinth, lit by a
//                cooked HDR environment map (env/studio.ogt: the cook
//                prefilters env/studio.hdr) as og.Environment, the map
//                drawn behind them, turned 40 degrees; AgX tone mapping and
//                auto exposure that adapts over time (1 EV/s up, 2.5 down).

import { og } from "@origingame/engine";
import type { Geometry, Material, Vector3 as Vec3, World } from "@origingame/engine";

export const IBL_STATIONS = ["environment"] as const;
export type IBLStation = (typeof IBL_STATIONS)[number];

/** Where the station lives along x. */
export const IBL_ORIGIN: Record<IBLStation, number> = { environment: 2500 };

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export interface IBLKit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
}

let kit: IBLKit;

/** Prepares the station; it is built when first shown (showIBL). */
export function buildIBL(k: IBLKit): void {
  kit = k;
}

function buildStudio(world: World): void {
  const x0 = IBL_ORIGIN.environment;
  const put = (name: string, geometry: Geometry, d: Parameters<typeof og.render.standardMaterial>[1], at: Vec3, scale: Vec3) => {
    const e = must(world.spawn({ name, position: at }));
    world.set(e, og.Transform, { scale });
    world.insert(e, og.Mesh, { geometry, material: kit.mat(d), castShadow: true, receiveShadow: true });
  };
  put("ibl-plinth", kit.box, { color: { r: 0.18, g: 0.17, b: 0.16 }, roughness: 0.35 }, { x: x0, y: 0.15, z: 0 }, { x: 6.4, y: 0.3, z: 2.2 });
  const balls: Array<[string, Parameters<typeof og.render.standardMaterial>[1]]> = [
    ["mirror", { color: { r: 0.95, g: 0.93, b: 0.9 }, metalness: 1, roughness: 0 }],
    ["brushed", { color: { r: 0.95, g: 0.93, b: 0.9 }, metalness: 1, roughness: 0.35 }],
    ["gold", { color: { r: 1, g: 0.78, b: 0.34 }, metalness: 1, roughness: 0.18 }],
    ["lacquer", { color: { r: 0.7, g: 0.05, b: 0.04 }, roughness: 0.08 }],
    ["chalk", { color: { r: 0.8, g: 0.8, b: 0.78 }, roughness: 0.9 }],
  ];
  balls.forEach(([name, d], i) => {
    put(`ibl-${name}`, kit.sphere, d, { x: x0 - 2.4 + i * 1.2, y: 0.8, z: 0 }, { x: 1, y: 1, z: 1 });
  });
}

/** Puts the light back on the sky and the tone mapping on ACES (every station's show calls it). */
export function resetIBL(world: World): void {
  const env = must(world.resource(og.Environment));
  env.map = 0 as unknown as typeof env.map;
  env.background = false;
  const settings = must(world.resource(og.RenderSettings));
  settings.toneMapping = "aces";
  settings.adaptSpeedUp = 0;
  settings.adaptSpeedDown = 0;
}

/** Shows the station: builds it the first time, then its camera, map and look. */
export function showIBL(world: World, station: IBLStation): void {
  if (world.find(`ibl-${station}`) === null) {
    buildStudio(world);
    must(world.spawn({ name: `ibl-${station}` }));
  }
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const env = must(world.resource(og.Environment));
  const x = IBL_ORIGIN[station];
  const eye = { x: x + 0.3, y: 1.75, z: 5.4 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x, y: 0.75, z: 0 });
  env.map = must(og.load.environment(world, "env/studio.ogt"));
  env.intensity = 1;
  env.rotation = 40;
  env.background = true;
  env.backgroundIntensity = 1;
  // The studio's key light is the sun: its shadows under the spheres.
  const sun = world.find("sun");
  if (sun !== null) {
    const turn = (40 * Math.PI) / 180;
    const key = { x: -0.5, y: 0.55, z: 0.65 };
    const towards = { x: Math.cos(turn) * key.x + Math.sin(turn) * key.z, y: key.y, z: -Math.sin(turn) * key.x + Math.cos(turn) * key.z };
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -towards.x, y: -towards.y, z: -towards.z }) });
    world.set(sun, og.DirectionalLight, { intensity: 1.2 });
  }
  settings.sky = false;
  settings.bloom = true;
  settings.bloomIntensity = 0.25;
  settings.toneMapping = "agx";
  settings.autoExposure = true;
  settings.adaptSpeedUp = 1;
  settings.adaptSpeedDown = 2.5;
  cam.exposure = 1;
}
