// The headland on the far-left horizon and its lighthouse: a ridge of dark
// rock and dull grass cut by cliffs to the sea (a height field built from
// seeded value noise, the same every run), a white tower with a red band at
// its seaward tip, the keeper's house beside it, and a beam that sweeps the
// sea at night.

import { og } from "@origingame/engine";
import type { Entity, Material, World } from "@origingame/engine";
import { MeshBuilder, must, v3 } from "./mesh.ts";
import type { Rgb, V3 } from "./mesh.ts";

/** Where the headland lies: 30° left of the view, 1.8 km out. */
export const HEADLAND = v3(-900, 0, -1560);
const HALF_X = 600;
const HALF_Z = 210;

function hash(i: number, j: number): number {
  let h = (i * 374761393 + j * 668265263) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}

function value(x: number, z: number): number {
  const i = Math.floor(x);
  const j = Math.floor(z);
  const fx = x - i;
  const fz = z - j;
  const sx = fx * fx * (3 - 2 * fx);
  const sz = fz * fz * (3 - 2 * fz);
  const a = hash(i, j) + (hash(i + 1, j) - hash(i, j)) * sx;
  const b = hash(i, j + 1) + (hash(i + 1, j + 1) - hash(i, j + 1)) * sx;
  return a + (b - a) * sz;
}

function fbm(x: number, z: number, octaves = 5): number {
  let s = 0;
  let a = 0.5;
  let f = 1;
  for (let o = 0; o < octaves; o++) {
    s += a * value(x * f, z * f);
    f *= 2.03;
    a *= 0.5;
  }
  return s;
}

const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

/** The headland's height at a point in its own frame (x along the shore, z across). */
export function headlandHeight(lx: number, lz: number): number {
  const n = fbm(lx / 160 + 11.3, lz / 160 - 4.1);
  // An elongated island, its outline worn by the noise.
  const e = Math.pow(lx / HALF_X, 2) + Math.pow(lz / (HALF_Z * (0.75 + 0.5 * (lx < 0 ? 1 : 0.6))), 2) + (n - 0.5) * 0.6;
  // High to the west, falling to a low knoll at the seaward (east) tip.
  const ridge = 40 + 120 * smooth(520, -350, lx) + 28 * Math.exp(-Math.pow((lx - 430) / 70, 2));
  const land = Math.pow(smooth(1.0, 0.72, e), 0.45);
  const rough = 0.82 + 0.36 * fbm(lx / 45, lz / 45, 4);
  return land > 0 ? -12 + (ridge * rough + 12) * land : -12;
}

const ROCK: Rgb = [0.075, 0.07, 0.065];
const GRASS: Rgb = [0.085, 0.1, 0.05];
const SAND: Rgb = [0.22, 0.2, 0.16];

/** The lighthouse's place in the world: on the knoll of the seaward tip. */
export function lighthouseAt(): V3 {
  const lx = 430;
  const lz = -20;
  return v3(HEADLAND.x + lx, headlandHeight(lx, lz), HEADLAND.z + lz);
}

export interface Land {
  lamp: Material;
  beam: Entity;
  beamMaterial: Material;
  light: Entity;
}

export function buildLand(world: World): Land {
  const m = new MeshBuilder();
  const nx = 140;
  const nz = 52;
  const h = (i: number, j: number) => headlandHeight(-HALF_X * 1.15 + (i / (nx - 1)) * HALF_X * 2.3, -HALF_Z * 1.3 + (j / (nz - 1)) * HALF_Z * 2.6);
  const dx = (HALF_X * 2.3) / (nx - 1);
  const dz = (HALF_Z * 2.6) / (nz - 1);
  const first = m.positions.length / 3;
  for (let j = 0; j < nz; j++) {
    for (let i = 0; i < nx; i++) {
      const y = h(i, j);
      const gx = (h(Math.min(nx - 1, i + 1), j) - h(Math.max(0, i - 1), j)) / (2 * dx);
      const gz = (h(i, Math.min(nz - 1, j + 1)) - h(i, Math.max(0, j - 1))) / (2 * dz);
      const slope = Math.hypot(gx, gz);
      const grass = (1 - smooth(0.45, 0.8, slope)) * smooth(6, 18, y);
      const sand = 1 - smooth(-2, 3, y);
      const c: Rgb = [0, 1, 2].map((k) => ROCK[k]! + (GRASS[k]! - ROCK[k]!) * grass + (SAND[k]! - ROCK[k]!) * sand * (1 - grass)) as Rgb;
      m.vertex(v3(-HALF_X * 1.15 + i * dx, y, -HALF_Z * 1.3 + j * dz), c);
    }
  }
  m.grid(first, nz, nx, true);
  const rock = must(og.render.standardMaterial(world, { roughness: 0.95 }));
  const land = must(world.spawn({ name: "headland", position: HEADLAND }));
  must(world.insert(land, og.Mesh, { geometry: m.geometry(world), material: rock, castShadow: true, receiveShadow: true }));

  // The lighthouse: a tapering white tower with a red band, its gallery,
  // the lantern room (glass that glows) and a dark dome.
  const at = lighthouseAt();
  const t = new MeshBuilder();
  const WHITE: Rgb = [0.82, 0.81, 0.77];
  const RED: Rgb = [0.42, 0.05, 0.04];
  const DARK: Rgb = [0.05, 0.05, 0.05];
  const o = v3(0, -1, 0);
  const r = (y: number) => 2.7 - (0.9 * y) / 22;
  t.lathe(o, [[0, 0], [3.4, 0], [3.4, 1.4], [r(0), 1.4], [r(9), 9]], WHITE);
  t.lathe(o, [[r(9), 9], [r(12.5), 12.5]], RED);
  t.lathe(o, [[r(12.5), 12.5], [r(22), 22], [2.6, 22], [2.6, 22.5], [0, 22.5]], WHITE);
  t.lathe(o, [[1.15, 25.2], [1.25, 25.2], [0.9, 26.4], [0.12, 27], [0, 27.4]], DARK);
  t.lathe(o, [[1.3, 22.5], [1.3, 22.8], [0, 22.8]], DARK);
  // The keeper's house beside it and its low wall.
  t.box(v3(-9, 1.6, 3), v3(9, 4.2, 5.5), WHITE);
  t.box(v3(-9, 4.1, 3), v3(9.6, 0.8, 6.2), DARK);
  t.box(v3(-2, 0.4, 9), v3(22, 1.2, 0.6), [0.3, 0.29, 0.27]);
  const tower = must(world.spawn({ name: "lighthouse", position: at }));
  must(world.insert(tower, og.Mesh, { geometry: t.geometry(world), material: must(og.render.standardMaterial(world, { roughness: 0.7 })), castShadow: true, receiveShadow: true }));

  const glass = new MeshBuilder();
  glass.lathe(v3(0, 0, 0), [[0, 22.8], [1.2, 22.8], [1.2, 25.2], [0, 25.2]], [1, 1, 1], 20);
  const lamp = must(og.render.standardMaterial(world, { color: { r: 0.9, g: 0.85, b: 0.7 }, emissive: { r: 1, g: 0.8, b: 0.5 }, emissiveIntensity: 0.3, roughness: 0.2 }));
  const lantern = must(world.spawn({ name: "lighthouse-lamp", parent: tower }));
  must(world.insert(lantern, og.Mesh, { geometry: glass.geometry(world), material: lamp }));
  const light = must(world.spawn({ name: "lighthouse-light", parent: tower, position: { x: 0, y: 24, z: 0 } }));
  must(world.insert(light, og.PointLight, { intensity: 0, distance: 120, color: { r: 1, g: 0.8, b: 0.55 } }));

  // The beam: two long cones from the lamp, opposite each other, fading
  // along their length, turned about the tower by the loop.
  const b = new MeshBuilder();
  const cone = (dir: number) => {
    const sides = 14;
    const reach = 520;
    const first = b.positions.length / 3;
    for (let k = 0; k <= 10; k++) {
      const d = (k / 10) * reach;
      const rad = 0.9 + d * 0.035;
      for (let i = 0; i < sides; i++) {
        const a = (i / sides) * Math.PI * 2;
        b.vertex(v3(dir * d, Math.cos(a) * rad, Math.sin(a) * rad), [1, 1, 1]);
      }
    }
    b.grid(first, 11, sides);
    return first;
  };
  cone(1);
  cone(-1);
  // Alpha fades along the beam (vertex colour's fourth channel).
  const n = b.positions.length / 3;
  const rgba: number[] = [];
  for (let i = 0; i < n; i++) {
    const d = Math.abs(b.positions[i * 3]!) / 520;
    const a = 0.22 * Math.pow(1 - d, 2.2);
    rgba.push(1, 0.86, 0.62, a);
  }
  const beamGeometry = must(og.render.bufferGeometry(world, { positions: new Float32Array(b.positions), colors: new Float32Array(rgba), indices: new Uint32Array(b.indices) }));
  const beamMaterial = must(og.render.standardMaterial(world, { unlit: true, transparent: true, opacity: 0, side: "double", color: { r: 1, g: 0.9, b: 0.7 } }));
  const beam = must(world.spawn({ name: "lighthouse-beam", parent: tower, position: { x: 0, y: 24, z: 0 } }));
  must(world.insert(beam, og.Mesh, { geometry: beamGeometry, material: beamMaterial }));
  return { lamp, beam, beamMaterial, light };
}
