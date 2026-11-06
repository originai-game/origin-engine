// The keep itself: the yard's floor, its walls, pillars, props, banners
// and torches (KayKit Dungeon Remastered, assets/models/keep/), the ground
// beyond the parapet, the colliders the heroes bump into (the navmesh is
// baked from the same boxes, src/layout.ts), the evening light, the
// torches' fire and the post stack.

import { og, math, SHAPE_BOX } from "@origingame/engine";
import type { Asset, World } from "@origingame/engine";
import { Game } from "./components.ts";
import { BANNERS, CHEST, PROPS, SCALE, solidBoxes, TILE, TORCHES, WALLS, YARD } from "./layout.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
export function must(r: { ok: true } | { ok: false; reason: string }): void;
export function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const UP = { x: 0, y: 1, z: 0 };
/** A floor tile's top above its origin in the kit (0.05 units): tiles are sunk by it, so the floor's top is y 0. */
const TILE_TOP = 0.05 * SCALE;

/** Builds the keep once, in `start`. */
export function buildKeep(world: World): void {
  const g = must(world.resource(Game));
  g.nav = must(og.nav.load(world, "nav/keep.ogt")).mesh as unknown as number;
  const model = (name: string): Asset => must(og.load.gltf(world, `models/keep/${name}.gltf`));
  const s = { x: SCALE, y: SCALE, z: SCALE };
  let n = 0;
  const place = (asset: Asset, x: number, y: number, z: number, yaw: number, scaleY = 1) =>
    must(world.instantiate(asset, { name: `keep-${n++}`, position: { x, y, z }, quaternion: og.transform.fromAxisAngle(UP, yaw), scale: { x: SCALE, y: SCALE * scaleY, z: SCALE } }));

  // The yard's paving, and a ring of trodden earth outside the parapet.
  const tile = model("floor_tile_large");
  const dirt = model("floor_dirt_large");
  for (let i = -4; i <= 4; i++) {
    for (let j = -5; j <= 5; j++) {
      const x = i * TILE;
      const z = j * TILE;
      const inside = x > YARD.minX && x < YARD.maxX && z > YARD.minZ && z < YARD.maxZ;
      if (inside) place(tile, x, -TILE_TOP, z, 0);
      else if (x > YARD.maxX + 1 || z > YARD.maxZ + 1 || x < YARD.minX - 1 || z < YARD.minZ - 1) place(dirt, x, -0.11 * SCALE, z, ((i + j) & 3) * (Math.PI / 2));
    }
  }
  // The ground beyond: one broad slab under everything, its top a hair below the floor's.
  const earth = must(og.render.standardMaterial(world, { color: { r: 0.2, g: 0.25, b: 0.15 }, roughness: 1 }));
  const slab = must(og.render.boxGeometry(world, { width: 120, height: 1, depth: 120 }));
  const ground = must(world.spawn({ name: "ground", position: { x: 0, y: -0.52, z: 0 } }));
  must(world.insert(ground, og.Mesh, { geometry: slab, material: earth, receiveShadow: true }));
  must(world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 120, y: 1, z: 120 } }));

  // The walls, the towers, the props and the chest: drawn, and each solid box a collider.
  const pieces = new Map<string, Asset>();
  const piece = (name: string) => {
    let a = pieces.get(name);
    if (a === undefined) pieces.set(name, (a = model(name)));
    return a;
  };
  for (const w of WALLS) {
    if (w.piece === "pillar") place(piece("pillar"), w.x, 0, w.z, 0, w.height);
    else place(piece(w.piece), w.x, 0, w.z, w.yaw, w.height);
  }
  for (const p of PROPS) {
    if (p.kind === "rubble_half") place(piece(p.kind), p.x - 1.4, 0, p.z, p.yaw, 1);
    else place(piece(p.kind), p.x, 0, p.z, p.yaw);
  }
  must(world.instantiate(piece("chest"), { name: "chest", position: { x: CHEST.x, y: 0.12 * SCALE, z: CHEST.z - 0.25 }, quaternion: og.transform.fromAxisAngle(UP, CHEST.yaw), scale: s }));
  for (const b of BANNERS) place(piece(b.shield ? "banner_shield_blue" : "banner_patternA_blue"), b.x, 0, b.z, b.yaw);
  solidBoxes().forEach((b, i) => {
    const e = must(world.spawn({ name: `solid-${i}`, position: { x: b.x, y: b.y, z: b.z } }));
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: b.w, y: b.h, z: b.d } }));
  });

  // The torches: a bracket on the wall, its fire and smoke, and a warm light that flickers (Torchlight).
  const torch = piece("torch");
  const fire = must(og.load.emitter(world, "fx/fire.ogt"));
  const smoke = must(og.load.emitter(world, "fx/smoke.ogt"));
  TORCHES.forEach((t, i) => {
    const out = { x: t.yaw === 0 ? 0 : 0.18, z: t.yaw === 0 ? 0.18 : 0 };
    const base = must(world.spawn({ name: `torch-${i}`, position: { x: t.x + out.x, y: 1.75, z: t.z + out.z } }));
    must(world.instantiate(torch, { name: `torch-${i}-bracket`, parent: base, quaternion: og.transform.fromAxisAngle(UP, t.yaw), scale: s }));
    const flame = must(world.spawn({ name: `torch-${i}-fire`, parent: base, position: { x: out.x * 0.6, y: 0.5, z: out.z * 0.6 }, scale: { x: 0.45, y: 0.5, z: 0.45 } }));
    must(world.insert(flame, og.ParticleSystem, { emitter: fire, seed: 11 + i }));
    const plume = must(world.spawn({ name: `torch-${i}-smoke`, parent: base, position: { x: out.x, y: 1.1, z: out.z }, scale: { x: 0.35, y: 0.35, z: 0.35 } }));
    must(world.insert(plume, og.ParticleSystem, { emitter: smoke, seed: 31 + i }));
    const light = must(world.spawn({ name: `torch-${i}-light`, parent: base, position: { x: out.x * 4, y: 0.7, z: out.z * 4 } }));
    must(world.insert(light, og.PointLight, { color: { r: 1, g: 0.58, b: 0.26 }, intensity: 9, distance: 9, castShadow: false }));
  });

  // The evening: a low warm sun over the west wall, a cool sky, a little haze.
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.bloomIntensity = 0.3;
  settings.environmentIntensity = 0.55;
  const post = must(world.resource(og.PostEffects));
  Object.assign(post, { ao: true, aoRadius: 0.6, aoIntensity: 0.8, vignette: true, vignetteIntensity: 0.32 });
  const sky = must(world.resource(og.Sky));
  Object.assign(sky, { enabled: true, sunSize: 1.1, glow: 0.4 });
  const fog = must(world.resource(og.Fog));
  Object.assign(fog, { enabled: true, density: 0.006, start: 40, heightDensity: 0, maxOpacity: 0.5, sky: 0.2, color: { r: 0.42, g: 0.44, b: 0.55 } });
  const sun = must(world.spawn({ name: "sun", position: { x: -20, y: 22, z: 6 } }));
  must(world.insert(sun, og.DirectionalLight, { color: { r: 1, g: 0.82, b: 0.62 }, intensity: 2.6, castShadow: true }));
  must(world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: -20, y: 22, z: 6 }, { x: 0, y: 0, z: 0 }) }));
}

/** The torches' flicker: each light's intensity wavers on its own beat (math.sin: the same on every peer). */
export function flicker(world: World, elapsed: number): void {
  TORCHES.forEach((_, i) => {
    const e = world.find(`torch-${i}-light`);
    if (e === null) return;
    const w = 0.82 + 0.1 * math.sin(elapsed * (9 + i)) + 0.08 * math.sin(elapsed * (23 + 3 * i) + i);
    must(world.set(e, og.PointLight, { intensity: 9 * w }));
  });
}
