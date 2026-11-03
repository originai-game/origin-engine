// The sky: the authored islands, the ruined arch and the far tower placed in
// the shot, the hero island's waterfall (a water sheet), and the meadow's
// grass and flower meshes for og.TerrainScatter.

import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { MeshBuilder, must, v3 } from "./mesh.ts";
import type { Rgb, V3 } from "./mesh.ts";

function hash(n: number): number {
  const s = Math.sin(n * 127.1 + 311.7) * 43758.5453;
  return s - Math.floor(s);
}

/** Places an authored model (assets/models/*.glb, ~1.9 units tall, centred on its origin) at `at`, scaled and turned. */
function place(world: World, path: string, name: string, at: V3, scale: number, yaw: number): Entity {
  const e = must(world.instantiate(must(og.load.gltf(world, path)), { name, position: at }));
  must(world.set(e, og.Transform, { position: at, scale: { x: scale, y: scale, z: scale }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw) }));
  return e;
}

export interface Sky {
  falls: Entity;
  /** Where the hero island's waterfall leaves its lip (world). */
  lip: V3;
}

/**
 * The sky props, placed about `eye` in the shot's frame (the camera looks down -z):
 * one hero island with the ruined arch and the waterfall centre right, one
 * medium and two small islands lower and left of centre (never in the
 * headline's block), the far tower soft on the horizon at about 38 % across.
 * All authored (Meshy, assets/models/SOURCE.md).
 */
export function buildSky(world: World, eye: V3): Sky {
  const hero = v3(eye.x + 60, eye.y - 70, eye.z - 420);
  const heroScale = 55;
  place(world, "models/island.glb", "island-hero", hero, heroScale, 0.4);
  // The arch on the hero island's cap (its top about 0.6 of the model's half height above its centre).
  place(world, "models/arch.glb", "arch", v3(hero.x - 10, hero.y + heroScale * 0.62, hero.z - 4), 9, -0.5);
  place(world, "models/island.glb", "island-medium", v3(eye.x - 120, eye.y - 120, eye.z - 640), 30, 2.1);
  place(world, "models/islandSmall.glb", "island-small-a", v3(eye.x + 160, eye.y - 90, eye.z - 300), 9, 0.9);
  place(world, "models/islandSmall.glb", "island-small-b", v3(eye.x - 30, eye.y - 140, eye.z - 340), 7, 3.3);
  place(world, "models/tower.glb", "tower", v3(eye.x - 1100, eye.y - 140, eye.z - 3200), 160, 0.3);

  // The waterfall: a thin bright sheet from the hero island's lip down into the deck.
  const lip = v3(hero.x - heroScale * 0.55, hero.y + heroScale * 0.45, hero.z + heroScale * 0.4);
  const f = new MeshBuilder();
  const drop = 420;
  const rows = 24;
  const first = f.positions.length / 3;
  for (let k = 0; k <= rows; k++) {
    const t = k / rows;
    const w = 3.5 + t * 9;
    const y = lip.y - t * drop;
    const out = 4 + Math.sqrt(t) * 14;
    const c: Rgb = [0.92, 0.95, 0.98];
    f.vertex(v3(lip.x - w * 0.5, y, lip.z + out), c);
    f.vertex(v3(lip.x + w * 0.5, y, lip.z + out), c);
  }
  f.grid(first, rows + 1, 2, false);
  const water = must(og.render.standardMaterial(world, { color: { r: 0.85, g: 0.9, b: 0.95 }, emissive: { r: 0.55, g: 0.6, b: 0.65 }, emissiveIntensity: 0.6, roughness: 0.3, transparent: true, opacity: 0.82, side: "double" }));
  const falls = must(world.spawn({ name: "falls" }));
  must(world.insert(falls, og.Mesh, { geometry: f.geometry(world), material: water }));
  return { falls, lip };
}

/** A grass clump: seven blades rooted at the origin, dark at the root to sage-gold at the tips. */
export function grassClump(world: World): ReturnType<MeshBuilder["geometry"]> {
  const b = new MeshBuilder();
  const blades = 7;
  for (let k = 0; k < blades; k++) {
    const a = (k / blades) * Math.PI * 2 + hash(k) * 0.8;
    const lean = 0.12 + hash(k + 9) * 0.18;
    const h = 0.38 + hash(k + 3) * 0.32;
    const w = 0.035 + hash(k + 5) * 0.02;
    const base = v3(Math.cos(a) * 0.06, 0, Math.sin(a) * 0.06);
    const dir = v3(Math.cos(a), 0, Math.sin(a));
    const side = v3(-Math.sin(a), 0, Math.cos(a));
    const segs = 3;
    const first = b.positions.length / 3;
    for (let s = 0; s <= segs; s++) {
      const t = s / segs;
      const c: Rgb = [0.12 + 0.42 * t, 0.2 + 0.4 * t, 0.1 + 0.12 * t];
      const p = v3(base.x + dir.x * lean * t * t * h * 2, t * h, base.z + dir.z * lean * t * t * h * 2);
      const half = w * (1 - t * 0.92);
      b.vertex(v3(p.x - side.x * half, p.y, p.z - side.z * half), c);
      b.vertex(v3(p.x + side.x * half, p.y, p.z + side.z * half), c);
    }
    b.grid(first, segs + 1, 2, false);
  }
  return b.geometry(world);
}

/** A flower: a short stem and a small cross of petals in `petal`. */
export function flower(world: World, petal: Rgb): ReturnType<MeshBuilder["geometry"]> {
  const b = new MeshBuilder();
  const stem: Rgb = [0.18, 0.3, 0.14];
  b.rod(v3(0, 0, 0), v3(0, 0.3, 0), 0.008, 0.006, stem, 4);
  const top = v3(0, 0.3, 0);
  for (let k = 0; k < 5; k++) {
    const a = (k / 5) * Math.PI * 2;
    const tip = v3(Math.cos(a) * 0.045, 0.31, Math.sin(a) * 0.045);
    const l = v3(Math.cos(a + 0.6) * 0.02, 0.305, Math.sin(a + 0.6) * 0.02);
    const r = v3(Math.cos(a - 0.6) * 0.02, 0.305, Math.sin(a - 0.6) * 0.02);
    const i = b.vertex(top, [0.95, 0.85, 0.4]);
    b.vertex(l, petal);
    b.vertex(tip, petal);
    b.vertex(r, petal);
    b.quad(i, i + 1, i + 2, i + 3);
  }
  return b.geometry(world);
}
