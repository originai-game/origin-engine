// The P4 physical materials station, far from the others along x, built
// when first shown (og-tests' `materials-physical` scene as a game):
//
//   physical  every MeshPhysicalMaterial lobe on a row of spheres on a
//             plinth of coloured bars: clearcoat with its three maps,
//             sheen plain and mapped, iridescence on black and on metal,
//             anisotropy plain and swirled by its map; in front, clear,
//             frosted and amber glass, a transmission-mapped cube and a
//             high-ior ball refracting the plinth and the ground behind.

import { og } from "@origingame/engine";
import type { Geometry, Material, Texture, Vector3 as Vec3, World } from "@origingame/engine";

export const PM_STATIONS = ["physical"] as const;
export type PMStation = (typeof PM_STATIONS)[number];

/** Where the station lives along x. */
export const PM_ORIGIN: Record<PMStation, number> = { physical: 2600 };

export interface PMKit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  ground: (x: number, color: Vec3) => void;
  tex: (path: string) => Texture;
}

let k: PMKit;

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Prepares the station; it is built when first shown (showPM). */
export function buildPM(kit: PMKit): void {
  k = kit;
}

function build(world: World): void {
  const x0 = PM_ORIGIN.physical;
  k.ground(x0, { x: 0.42, y: 0.43, z: 0.45 });
  const put = (name: string, geometry: Geometry, d: Parameters<typeof og.render.standardMaterial>[1], at: Vec3, scale?: Vec3, turn = 0) => {
    const q = og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn);
    const e = must(world.spawn({ name, position: at, quaternion: q }));
    if (scale) world.set(e, og.Transform, { scale });
    world.insert(e, og.Mesh, { geometry, material: k.mat(d), castShadow: true, receiveShadow: true });
  };
  put("plinth", k.box, { map: k.tex("textures/bars.png"), roughness: 0.7 }, { x: x0, y: 0.35, z: -1.8 }, { x: 9, y: 0.7, z: 0.8 });
  const stripes = k.tex("textures/stripes.png");
  const bands = k.tex("textures/bands.png");
  const ramp = k.tex("textures/ramp.png");
  const sheen = k.tex("textures/sheen.png");
  const back: Parameters<typeof og.render.standardMaterial>[1][] = [
    { color: { r: 0.7, g: 0.06, b: 0.05 }, roughness: 0.8, clearcoat: 1, clearcoatRoughness: 0.08, clearcoatMap: bands, clearcoatRoughnessMap: ramp, clearcoatNormalMap: k.tex("textures/bumps.png") },
    { color: { r: 0.03, g: 0.04, b: 0.18 }, roughness: 0.9, sheen: 1, sheenColor: { r: 1, g: 0.45, b: 0.7 }, sheenRoughness: 0.35 },
    { color: { r: 0.02, g: 0.12, b: 0.05 }, roughness: 0.9, sheen: 1, sheenColor: { r: 1, g: 1, b: 1 }, sheenRoughness: 0.6, sheenColorMap: sheen, sheenRoughnessMap: sheen },
    { color: { r: 0.02, g: 0.02, b: 0.02 }, roughness: 0.15, iridescence: 1, iridescenceIOR: 1.3, iridescenceThicknessRange: { x: 200, y: 1200 }, iridescenceThicknessMap: ramp },
    { color: { r: 0.8, g: 0.8, b: 0.85 }, metalness: 1, roughness: 0.25, iridescence: 1, iridescenceIOR: 1.8, iridescenceThicknessRange: { x: 250, y: 450 } },
    { color: { r: 0.9, g: 0.9, b: 0.92 }, metalness: 1, roughness: 0.35, anisotropy: 0.9 },
    { color: { r: 1, g: 0.78, b: 0.34 }, metalness: 1, roughness: 0.4, anisotropy: 1, anisotropyRotation: 0.3, anisotropyMap: k.tex("textures/swirl.png") },
  ];
  back.forEach((d, i) => put(`physical-${i}`, k.sphere, d, { x: x0 + i * 1.2 - 3.6, y: 1.2, z: -1.8 }));
  const glass = { color: { r: 1, g: 1, b: 1 }, roughness: 0.02, transmission: 1, thickness: 1 };
  const front = (i: number, y = 0.5) => ({ x: x0 + i * 1.55 - 3.1, y, z: 0.9 });
  put("glass-clear", k.sphere, glass, front(0));
  put("glass-frosted", k.sphere, { ...glass, roughness: 0.35 }, front(1));
  put("glass-amber", k.sphere, { ...glass, roughness: 0.05, attenuationColor: { r: 0.95, g: 0.45, b: 0.08 }, attenuationDistance: 0.35 }, front(2));
  put("glass-striped", k.box, { ...glass, color: { r: 0.85, g: 0.9, b: 1 }, roughness: 0.1, transmissionMap: stripes, thickness: 0.85 }, front(3, 0.425), { x: 0.85, y: 0.85, z: 0.85 }, 0.5);
  put("glass-ior", k.sphere, { ...glass, color: { r: 0.95, g: 0.97, b: 1 }, ior: 2.33 }, front(4));
}

/** Shows the station: builds it the first time, then its camera and light. */
export function showPM(world: World, station: PMStation): void {
  if (world.find(`pm-${station}`) === null) {
    build(world);
    must(world.spawn({ name: `pm-${station}` }));
  }
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const sun = world.find("sun");
  const x = PM_ORIGIN[station];
  const eye = { x, y: 2.3, z: 6.8 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x, y: 0.8, z: -0.6 });
  if (sun !== null) {
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.5, y: -0.75, z: -0.45 }) });
    world.set(sun, og.DirectionalLight, { intensity: 3 });
  }
  settings.sky = true;
  settings.environmentIntensity = 0.9;
  settings.bloom = false;
  cam.exposure = 0.7;
}
