// The phase 4 looks station, far from the others along x, built when first
// shown:
//
//   looks  a diorama that every look can be read on: a checkered ground,
//          a row of spheres from matte to chrome, a textured crate, a
//          stepped tree and a lantern that blooms. `look:<slug>` applies
//          one of the engine's presets (assets/looks/<slug>.ogt extends the
//          built-in look of that slug; `look:none` takes it off), so the
//          same scene is seen through every style of the catalogue.

import { og } from "@origingame/engine";
import type { Geometry, Material, Texture, Vector3 as Vec3, World } from "@origingame/engine";
import { LOOK_PRESETS } from "./look-presets.ts";

export const LOOK_STATIONS = ["looks"] as const;
export type LookStation = (typeof LOOK_STATIONS)[number];

/** Where the station lives along x. */
export const LOOK_ORIGIN: Record<LookStation, number> = { looks: 2800 };

/** The presets the station cycles, in the catalogue's order (look-presets.ts). */
export { LOOK_PRESETS };

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export interface LookKit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  tex: (path: string) => Texture;
}

let kit: LookKit;

/** Prepares the station; it is built when first shown (showLooks). */
export function buildLooks(k: LookKit): void {
  kit = k;
}

function build(world: World): void {
  const x0 = LOOK_ORIGIN.looks;
  const put = (name: string, geometry: Geometry, d: Parameters<typeof og.render.standardMaterial>[1], at: Vec3, scale: Vec3, turn = 0) => {
    const q = og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn);
    const e = must(world.spawn({ name, position: at, quaternion: q }));
    world.set(e, og.Transform, { scale });
    world.insert(e, og.Mesh, { geometry, material: kit.mat(d), castShadow: true, receiveShadow: true });
  };
  // The ground: a checkered tile, so the pixel grid, the dither and the
  // affine textures have something to show on.
  put("looks-ground", kit.box, { map: kit.tex("textures/tile.png"), roughness: 0.9 }, { x: x0, y: -0.1, z: 0 }, { x: 40, y: 0.2, z: 40 });
  // Spheres from matte to chrome, saturated hues for the palettes.
  const balls: Array<[string, Parameters<typeof og.render.standardMaterial>[1]]> = [
    ["red", { color: { r: 0.85, g: 0.12, b: 0.1 }, roughness: 0.85 }],
    ["gold", { color: { r: 1, g: 0.78, b: 0.34 }, metalness: 1, roughness: 0.3 }],
    ["blue", { color: { r: 0.15, g: 0.35, b: 0.9 }, roughness: 0.5 }],
    ["chrome", { color: { r: 0.95, g: 0.93, b: 0.9 }, metalness: 1, roughness: 0.05 }],
    ["green", { color: { r: 0.25, g: 0.7, b: 0.3 }, roughness: 0.7 }],
  ];
  balls.forEach(([name, d], i) => {
    put(`looks-${name}`, kit.sphere, d, { x: x0 - 3.2 + i * 1.6, y: 0.7, z: 0.6 }, { x: 1.4, y: 1.4, z: 1.4 });
  });
  // A textured crate, turned, and a stepped tree behind.
  put("looks-crate", kit.box, { map: kit.tex("textures/crate.png"), roughness: 0.8 }, { x: x0 + 2.8, y: 0.6, z: -2.2 }, { x: 1.2, y: 1.2, z: 1.2 }, 0.6);
  put("looks-trunk", kit.box, { color: { r: 0.35, g: 0.22, b: 0.12 }, roughness: 0.9 }, { x: x0 - 3.2, y: 0.9, z: -3 }, { x: 0.5, y: 1.8, z: 0.5 });
  for (let i = 0; i < 3; i++) {
    const s = 2.6 - i * 0.7;
    put(`looks-leaves-${i}`, kit.box, { color: { r: 0.18, g: 0.5, b: 0.22 }, roughness: 0.9 }, { x: x0 - 3.2, y: 2.0 + i * 0.75, z: -3 }, { x: s, y: 0.7, z: s }, 0.785 * i);
  }
  // A lantern: emissive, so bloom and the neon look have a light to catch.
  put("looks-lantern", kit.sphere, { color: { r: 1, g: 0.6, b: 0.2 }, emissive: { r: 1, g: 0.55, b: 0.15 }, emissiveIntensity: 6, roughness: 0.6 }, { x: x0 + 1.2, y: 2.2, z: -1.4 }, { x: 0.5, y: 0.5, z: 0.5 });
  put("looks-post", kit.box, { color: { r: 0.2, g: 0.2, b: 0.22 }, roughness: 0.6 }, { x: x0 + 1.2, y: 1, z: -1.4 }, { x: 0.12, y: 2, z: 0.12 });
  // A far wall, for the depth bands.
  put("looks-wall", kit.box, { color: { r: 0.6, g: 0.62, b: 0.7 }, roughness: 0.9 }, { x: x0, y: 2, z: -14 }, { x: 30, y: 4, z: 0.5 });
}

/** The station's own settings (a look replaces RenderSettings whole). */
function settle(world: World): void {
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.environmentIntensity = 0.8;
}

/**
 * Applies the preset `slug` (`none` takes every look off). A look owns
 * only the resources it names, so the station takes the last one off first:
 * each frame is one preset alone, never a pile of them.
 */
export function applyLookPreset(world: World, slug: string): void {
  const file = LOOK_PRESETS.includes(slug as (typeof LOOK_PRESETS)[number]) || slug === "none" ? slug : "none";
  resetLooks(world);
  if (file !== "none") {
    const look = must(og.load.look(world, `looks/${file}.ogt`));
    must(og.render.applyLook(world, look));
  }
  settle(world);
}

/** Takes every look off (every station's show calls it). */
export function resetLooks(world: World): void {
  const look = must(og.load.look(world, "looks/none.ogt"));
  must(og.render.applyLook(world, look));
  // A look owns only what it names: the none look has no `ui` line, so the
  // style the last preset dressed the HUD in stays until the UI is told
  // otherwise. Null is the prototype style (UiDesc.style).
  must(og.ui.configure(world, { style: null }));
  // The render settings a preset may set, back at their defaults (the
  // none look leaves RenderSettings alone: it holds the decal atlas).
  Object.assign(must(world.resource(og.RenderSettings)), {
    toneMapping: "aces", exposure: 1, bloom: false, bloomThreshold: 1, bloomKnee: 0.5, bloomIntensity: 0.6,
    autoExposure: false, adaptSpeedUp: 0, adaptSpeedDown: 0,
  });
}

/** Shows the station: builds it the first time, then its camera and light. */
export function showLooks(world: World, station: LookStation): void {
  if (world.find(`lk-${station}`) === null) {
    build(world);
    must(world.spawn({ name: `lk-${station}` }));
  }
  const cam = must(world.resource(og.Camera));
  const sun = world.find("sun");
  const x = LOOK_ORIGIN[station];
  const eye = { x: x + 0.4, y: 3.2, z: 8.5 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x, y: 1.1, z: -1 });
  cam.exposure = 0.8;
  if (sun !== null) {
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.45, y: -0.7, z: -0.5 }) });
    world.set(sun, og.DirectionalLight, { intensity: 3 });
  }
  settle(world);
}
