// The stair itself: the level model (models/stair.glb, written by
// tools/level.ts from src/layout.ts) with a triangle-mesh collider per
// primitive, the sun and its late afternoon (og.DayNight held at the run's
// hour), the braziers (fire, light and a crackle heard from where each
// stands), the banners (og.Cloth in the wind), the crypt's chain and lamp
// (og.Rope), the summit gate (a collider and a navmesh obstacle until three
// sigils raise it), the bell, the waystones, the sigils and the herbs.

import { og, SHAPE_BOX, SHAPE_TRIMESH } from "@origingame/engine";
import type { Entity, Geometry, Material, Value, World } from "@origingame/engine";
import { Game, HOURS, Near, Pickup, Waystone } from "./components.ts";
import { BANNERS, BELL, BRAZIERS, CRYPT, GATE, HERBS, SIGILS, WAYSTONES } from "./layout.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
export function must(r: { ok: true } | { ok: false; reason: string }): void;
export function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export function entity(world: World, name: string): Entity {
  const e = world.find(name);
  if (e === null) throw new Error(`${name} is missing`);
  return e;
}

/** Collision layers: the level, the courier, the foes, the gate, the cloth's own. */
export const LAYER = { level: 1, hero: 2, foe: 4, gate: 8 } as const;

type GameV = Value<typeof Game>;

/** The ground under (x, z), from y (a ray straight down against the level); y when nothing is there. */
export function groundAt(world: World, x: number, y: number, z: number): number {
  const hit = og.physics.raycast(world, { origin: { x, y: y + 1.5, z }, direction: { x: 0, y: -1, z: 0 }, maxDistance: 40, filter: { mask: LAYER.level } });
  return hit.ok && hit.value ? hit.value.point.y : y;
}

/** Builds the stair once, in `start`. The look already set the frame's treatment. */
export function buildStair(world: World, g: GameV): void {
  const asset = must(og.load.gltf(world, "models/stair.glb", { castShadow: true, receiveShadow: true }));
  must(world.instantiate(asset, { name: "stair" }));
  const info = must(og.load.gltfInfo(world, asset));
  // One static triangle-mesh collider per primitive of the level.
  info.geometries.forEach((geo, i) => {
    const c = must(world.spawn({ name: `stair-collider-${i}` }));
    must(world.insert(c, og.Collider, { shape: SHAPE_TRIMESH, mesh: geo, layers: LAYER.level, friction: 0.8 }));
  });

  const sun = must(world.spawn({ name: "sun", position: { x: 0, y: 40, z: 0 } }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 2.6, castShadow: true, shadowDistance: 70, shadowSoftness: 0.02, cascadeBlend: 0.15 }));
  const day = must(world.resource(og.DayNight));
  // The sun rises in the north-east and sets in the south-west, so the evening light falls on the stair's faces.
  Object.assign(day, { enabled: true, start: HOURS.title, speed: 0, tilt: 30, azimuth: 45, lightIntensity: 1.4, tintFog: true, moonSize: 1.4, moonPhase: 0.3 });

  buildBraziers(world);
  buildBanners(world);
  buildCrypt(world);
  buildGate(world, g);
  buildBell(world);
  buildWaystones(world);
  buildPickups(world);
}

function sphere(world: World, radius: number): Geometry {
  return must(og.render.sphereGeometry(world, { radius, widthSegments: 16, heightSegments: 10 }));
}

function box(world: World, x: number, y: number, z: number): Geometry {
  return must(og.render.boxGeometry(world, { width: x, height: y, depth: z }));
}

function buildBraziers(world: World): void {
  const fire = must(og.load.emitter(world, "fx/fire.ogt"));
  const crackle = must(og.load.sound(world, "audio/crackle.wav"));
  BRAZIERS.forEach((b, i) => {
    const e = must(world.spawn({ name: `brazier-${i}`, position: { x: b.x, y: b.y + 1.12, z: b.z } }));
    must(world.insert(e, og.ParticleSystem, { emitter: fire, seed: 100 + i }));
    must(world.insert(e, og.PointLight, { color: { r: 1, g: 0.55, b: 0.22 }, intensity: 9, distance: 9 }));
    // Heard from where it stands: panned and fading with distance, dulled far off and behind walls.
    must(world.insert(e, og.AudioSource, { sound: crackle, bus: "sfx", loop: true, spatial: true, refDistance: 1.5, maxDistance: 18, rolloffFactor: 1.6, distanceCutoff: 2500, occlude: true, volume: 0.55, priority: 60 }));
  });
}

function buildBanners(world: World): void {
  const tex = must(og.load.texture(world, "textures/banner.png"));
  const cloth = must(og.render.standardMaterial(world, { map: tex, roughness: 0.95, side: "double", sheen: 0.6, sheenColor: { r: 0.9, g: 0.75, b: 0.45 }, sheenRoughness: 0.7 }));
  BANNERS.forEach((b, i) => {
    // Hung from the crossbar: the cloth's top-left corner, laid along the entity's x and down its -y.
    const e = must(world.spawn({ name: `banner-${i}`, position: { x: b.x + 0.15, y: b.y + 4.15, z: b.z + 0.14 } }));
    must(world.insert(e, og.Cloth, { columns: 8, rows: 14, spacing: 0.14, pins: 1, iterations: 10, damping: 0.985, wind: { x: 0.6, y: 0, z: 2.2 }, gust: 0.8, radius: 0.04, material: cloth }));
  });
}

function buildCrypt(world: World): void {
  // The chain from the roof, free at its end; the lamp hangs where its last link is (src/fx.ts).
  const c = CRYPT.chain;
  const chain = must(world.spawn({ name: "crypt-chain", position: { x: c.x, y: c.y, z: c.z } }));
  must(world.insert(chain, og.Rope, { segments: 14, length: c.length, iterations: 16, damping: 0.995, radius: 0.04, width: 0.05, color: { x: 0.12, y: 0.11, z: 0.1 }, wind: { x: 0.4, y: 0, z: 0.15 } }));
  const glass = must(og.render.standardMaterial(world, { color: { r: 1, g: 0.8, b: 0.4 }, emissive: { r: 1, g: 0.6, b: 0.25 }, emissiveIntensity: 4 }));
  const lamp = must(world.spawn({ name: "crypt-lamp", position: { x: c.x, y: c.y - c.length, z: c.z } }));
  must(world.insert(lamp, og.Mesh, { geometry: box(world, 0.22, 0.3, 0.22), material: glass }));
  must(world.insert(lamp, og.PointLight, { color: { r: 1, g: 0.62, b: 0.3 }, intensity: 7, distance: 9, castShadow: true }));
  // The drip, from the crypt's dark corner.
  const drip = must(world.spawn({ name: "crypt-drip", position: { x: -19, y: 5, z: -12.5 } }));
  must(world.insert(drip, og.AudioSource, { sound: must(og.load.sound(world, "audio/drip.wav")), bus: "sfx", loop: true, spatial: true, refDistance: 1, maxDistance: 14, occlude: true, volume: 0.7, priority: 50 }));
}

/** The gate's portcullis: solid, and an obstacle on the navmesh, until it rises. */
function buildGate(world: World, g: GameV): void {
  const iron = must(og.render.standardMaterial(world, { color: { r: 0.18, g: 0.17, b: 0.17 }, metalness: 0.6, roughness: 0.5 }));
  const [x0, y0, z0] = GATE.min;
  const [x1, y1, z1] = GATE.max;
  const gate = must(world.spawn({ name: "gate", position: { x: (x0 + x1) / 2, y: (y0 + y1) / 2, z: (z0 + z1) / 2 } }));
  must(world.insert(gate, og.Mesh, { geometry: box(world, x1 - x0, y1 - y0, z1 - z0), material: iron, castShadow: true, receiveShadow: true }));
  must(world.insert(gate, og.Collider, { shape: SHAPE_BOX, size: { x: x1 - x0, y: y1 - y0, z: z1 - z0 }, layers: LAYER.gate }));
  void g;
}

/** Shuts (or raises) the gate on the navmesh: an obstacle the foes plan round while it is down. */
export function gateObstacle(world: World, g: GameV, shut: boolean): void {
  const mesh = g.nav as unknown as Parameters<typeof og.nav.setObstacle>[1];
  if (shut) must(og.nav.setObstacle(world, mesh, 1, { min: { x: GATE.min[0], y: GATE.min[1] - 0.5, z: GATE.min[2] }, max: { x: GATE.max[0], y: GATE.max[1], z: GATE.max[2] } }));
  else must(og.nav.setObstacle(world, mesh, 1));
}

function buildBell(world: World): void {
  const bronze = must(og.render.standardMaterial(world, { color: { r: 0.72, g: 0.5, b: 0.22 }, metalness: 0.9, roughness: 0.35, clearcoat: 0.4 }));
  const wood = must(og.render.standardMaterial(world, { color: { r: 0.36, g: 0.22, b: 0.12 }, roughness: 0.85 }));
  const frame = must(world.spawn({ name: "bell-frame", position: { x: BELL.x, y: BELL.y, z: BELL.z } }));
  for (const dx of [-1.3, 1.3]) {
    const post = must(world.spawn({ parent: frame, position: { x: dx, y: 1.6, z: 0 } }));
    must(world.insert(post, og.Mesh, { geometry: box(world, 0.25, 3.2, 0.25), material: wood, castShadow: true }));
    must(world.insert(post, og.Collider, { shape: SHAPE_BOX, size: { x: 0.25, y: 3.2, z: 0.25 }, layers: LAYER.level }));
  }
  const beam = must(world.spawn({ parent: frame, position: { x: 0, y: 3.25, z: 0 } }));
  must(world.insert(beam, og.Mesh, { geometry: box(world, 3.0, 0.3, 0.3), material: wood, castShadow: true }));
  const bell = must(world.spawn({ name: "bell", parent: frame, position: { x: 0, y: 2.35, z: 0 } }));
  must(world.insert(bell, og.Mesh, { geometry: sphere(world, 0.7), material: bronze, castShadow: true }));
  must(world.insert(bell, og.AudioSource, { sound: must(og.load.sound(world, "audio/bell.wav")), bus: "sfx", playing: false, spatial: true, refDistance: 4, maxDistance: 120, rolloffFactor: 0.6, volume: 1.2, priority: 240 }));
}

function buildWaystones(world: World): void {
  const geo = sphere(world, 0.16);
  WAYSTONES.forEach((w, i) => {
    const e = must(world.spawn({ name: `waystone-${i}`, position: { x: w.x, y: w.y + 1.75, z: w.z } }));
    // Each its own material, so one glows while the others stay dim.
    const mat = must(og.render.standardMaterial(world, { color: { r: 0.6, g: 0.75, b: 0.9 }, emissive: { r: 0.45, g: 0.75, b: 1 }, emissiveIntensity: 0.4 }));
    must(world.insert(e, og.Mesh, { geometry: geo, material: mat }));
    must(world.insert(e, og.PointLight, { color: { r: 0.55, g: 0.75, b: 1 }, intensity: 0.5, distance: 6 }));
    must(world.insert(e, Waystone, { index: i, glow: 0 }));
  });
}

/** A waystone's glow (0 dim, 1 rested at). */
export function setWaystoneGlow(world: World, i: number, glow: number): void {
  const e = entity(world, `waystone-${i}`);
  must(world.set(e, og.PointLight, { intensity: 0.5 + glow * 5 }));
  const m = must(world.get(e, og.Mesh)).material;
  must(og.render.updateMaterial(world, m, { emissiveIntensity: 0.4 + glow * 3.5 }));
}

function buildPickups(world: World): void {
  const sigilMat: Material = must(og.render.standardMaterial(world, { color: { r: 0.85, g: 0.62, b: 0.25 }, metalness: 0.85, roughness: 0.3, emissive: { r: 1, g: 0.7, b: 0.3 }, emissiveIntensity: 1.2 }));
  const herbMat: Material = must(og.render.standardMaterial(world, { color: { r: 0.35, g: 0.62, b: 0.28 }, roughness: 0.8, emissive: { r: 0.3, g: 0.6, b: 0.2 }, emissiveIntensity: 0.25 }));
  const disc = box(world, 0.42, 0.42, 0.08);
  const leaf = box(world, 0.16, 0.32, 0.04);
  SIGILS.forEach((s, i) => {
    const e = must(world.spawn({ name: `sigil-${i}`, position: { x: s.x, y: s.y + 0.8, z: s.z } }));
    must(world.insert(e, og.Mesh, { geometry: disc, material: sigilMat, castShadow: true, visible: i !== 2 }));
    must(world.insert(e, og.PointLight, { color: { r: 1, g: 0.75, b: 0.35 }, intensity: i === 2 ? 0 : 3, distance: 4 }));
    must(world.insert(e, Pickup, { kind: Near.Sigil, index: i, taken: false, x: s.x, y: s.y + 0.8, z: s.z }));
  });
  const burst = must(world.spawn({ name: "sigil-burst" }));
  must(world.insert(burst, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/sigil.ogt")), seed: 3, visible: false }));
  HERBS.forEach((h, i) => {
    const e = must(world.spawn({ name: `herb-${i}`, position: { x: h.x, y: h.y, z: h.z } }));
    must(world.insert(e, Pickup, { kind: Near.Herb, index: i, taken: false, x: h.x, y: h.y, z: h.z }));
    for (let k = 0; k < 5; k++) {
      const a = (k / 5) * Math.PI * 2;
      const l = must(world.spawn({ parent: e, position: { x: Math.sin(a) * 0.12, y: 0.16, z: Math.cos(a) * 0.12 }, quaternion: og.transform.fromAxisAngle({ x: Math.cos(a), y: 0, z: -Math.sin(a) }, 0.5) }));
      must(world.insert(l, og.Mesh, { geometry: leaf, material: herbMat }));
    }
  });
}

/** Sets the time of day og.DayNight is held at (the game moves it, so a pause holds it). */
export function setHour(world: World, hour: number): void {
  const day = must(world.resource(og.DayNight));
  day.speed = 0;
  day.start = hour;
}
