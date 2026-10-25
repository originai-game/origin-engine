// Every asset of the complex scene, made by code (`bun tools/bench/complex/tools/make-assets.ts`),
// the same bytes on every run (seeded, no clock, nothing downloaded):
//
// - models/ground.glb: the 320 m ground, 256 x 256 quads (131,072
//   triangles) over src/layout.js's height field, in four textured
//   materials (grass, dirt, rock, sand) with a shared normal map;
// - the prop kit: pine, oak, three rocks, bush, crate, barrel, fence,
//   lamp post, house (with blended window glass), market stall, each with
//   its own materials and 256 x 256 textures embedded in the .glb;
// - models/villager.glb: a box-bodied humanoid on examples/kestrel-stair's
//   clip library skeleton, with the library's nine clips in the file (so
//   three.js's GLTFLoader and og.load.animation read the same clips);
// - the engine's own CC0 models copied in: walker.glb (crates/og-tests/
//   reference-scene), palm, berry and islet (examples/rumpus-isle), car and
//   ghost (examples/meridian-loop), stair (examples/kestrel-stair), and
//   dropship.glb (templates/game, MIT);
// - the particle textures and the two emitters (fx/fire.ogt, fx/smoke.ogt)
//   written from src/layout.js's FX table, which three-complex.js simulates.
//
// tools/bench/threejs-compare.ts runs it when this script or src/layout.js
// changed since the stamp it keeps in assets/.made; assets/ is not committed.

import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Glb, MeshBuilder, quat, v3 } from "../../../../examples/kestrel-stair/tools/gltf.ts";
import type { Q, V3 } from "../../../../examples/kestrel-stair/tools/gltf.ts";
import { draw, fbm, hash2, png, rng } from "../../../../examples/kestrel-stair/tools/media.ts";
import { BODIES, character, library } from "../../../../examples/kestrel-stair/tools/rig.ts";
import { FX, GRID, SIZE, height, slope, LAKE, VILLAGE, FIELD, PARK } from "../src/layout.js";

const here = dirname(fileURLToPath(import.meta.url));
const project = join(here, "..");
const repo = join(project, "..", "..", "..");
const assets = join(project, "assets");

function write(rel: string, bytes: Uint8Array | string): void {
  const path = join(assets, rel);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, bytes);
}

// ── textures ────────────────────────────────────────────────────────────
const T = 256;
const srgb = (c: number) => Math.round(Math.max(0, Math.min(1, c)) * 255);
/** A tileable colour texture: base colour varied by tileable fbm. */
function tex(seed: number, base: [number, number, number], vary: number, extra?: (u: number, v: number, n: number) => [number, number, number]): Uint8Array {
  return draw(T, (u, v) => {
    const n = fbm(u * 8, v * 8, seed, 4, 8);
    const c = extra ? extra(u, v, n) : base;
    const k = 1 + (n - 0.5) * vary;
    return [srgb(c[0] * k), srgb(c[1] * k), srgb(c[2] * k), 255];
  });
}
/** A tileable normal map from fbm heights. */
function normalTex(seed: number, strength: number, freq = 8): Uint8Array {
  const h = (x: number, y: number) => fbm((x / T) * freq, (y / T) * freq, seed, 5, freq);
  const px = new Float32Array(T * T * 4);
  for (let y = 0; y < T; y++)
    for (let x = 0; x < T; x++) {
      const dx = (h((x + 1) % T, y) - h((x + T - 1) % T, y)) * strength;
      const dy = (h(x, (y + 1) % T) - h(x, (y + T - 1) % T)) * strength;
      const n = v3.norm([-dx, dy, 1]);
      px.set([(n[0] * 0.5 + 0.5) * 255, (n[1] * 0.5 + 0.5) * 255, (n[2] * 0.5 + 0.5) * 255, 255], (y * T + x) * 4);
    }
  return png(T, T, px, 4);
}
const planks = (u: number, v: number, n: number): [number, number, number] => {
  const board = Math.floor(v * 6);
  const seam = (v * 6) % 1 < 0.06 ? 0.45 : 1;
  const grain = 0.85 + 0.15 * Math.sin((u * 40 + hash2(board, 0, 5) * 20) + n * 6);
  const tone = 0.85 + 0.3 * hash2(board, 1, 9);
  return [0.48 * grain * seam * tone, 0.32 * grain * seam * tone, 0.18 * grain * seam * tone];
};

// ── geometry ────────────────────────────────────────────────────────────
interface Prim { pos: number[]; nor: number[]; uv: number[]; col: number[]; idx: number[] }
const prim = (): Prim => ({ pos: [], nor: [], uv: [], col: [], idx: [] });
/** Moves the vertices added since `from` by a rotation, a scale and a translation. */
function place(p: Prim, from: number, at: V3, rot: Q = [0, 0, 0, 1], scale: V3 = [1, 1, 1]): void {
  for (let i = from; i < p.pos.length / 3; i++) {
    const s: V3 = [p.pos[i * 3]! * scale[0], p.pos[i * 3 + 1]! * scale[1], p.pos[i * 3 + 2]! * scale[2]];
    const w = v3.add(quat.rotate(rot, s), at);
    p.pos.splice(i * 3, 3, w[0], w[1], w[2]);
    const n = v3.norm(quat.rotate(rot, v3.norm([p.nor[i * 3]! / scale[0], p.nor[i * 3 + 1]! / scale[1], p.nor[i * 3 + 2]! / scale[2]])));
    p.nor.splice(i * 3, 3, n[0], n[1], n[2]);
  }
}
const start = (p: Prim) => p.pos.length / 3;
function vert(p: Prim, pos: V3, nor: V3, uv: [number, number], col: [number, number, number] = [1, 1, 1]): void {
  p.pos.push(...pos);
  p.nor.push(...nor);
  p.uv.push(...uv);
  p.col.push(col[0], col[1], col[2], 1);
}
/** A cylinder (or a cone, r1 = 0) along +y from y0 to y1, `seg` sides, `rings` rings, with caps. */
function cylinder(p: Prim, r0: number, r1: number, y0: number, y1: number, seg: number, tile = 1, rings = 1, col?: [number, number, number]): void {
  const base = start(p);
  const slopeN = (r0 - r1) / (y1 - y0);
  for (let j = 0; j <= rings; j++) {
    const t = j / rings, y = y0 + (y1 - y0) * t, r = r0 + (r1 - r0) * t;
    for (let i = 0; i <= seg; i++) {
      const a = (i / seg) * Math.PI * 2;
      vert(p, [Math.cos(a) * r, y, Math.sin(a) * r], v3.norm([Math.cos(a), slopeN, Math.sin(a)]), [(i / seg) * Math.max(1, Math.round((2 * Math.PI * r0) / tile)), y / tile], col);
    }
  }
  for (let j = 0; j < rings; j++)
    for (let i = 0; i < seg; i++) {
      const a = base + j * (seg + 1) + i, b = a + seg + 1;
      p.idx.push(a, b, a + 1, a + 1, b, b + 1);
    }
  for (const [y, r, up] of [[y0, r0, -1], [y1, r1, 1]] as const) {
    if (r <= 0) continue;
    const c = start(p);
    vert(p, [0, y, 0], [0, up, 0], [0.5, 0.5], col);
    for (let i = 0; i <= seg; i++) {
      const a = (i / seg) * Math.PI * 2;
      vert(p, [Math.cos(a) * r, y, Math.sin(a) * r], [0, up, 0], [0.5 + Math.cos(a) * 0.5, 0.5 + Math.sin(a) * 0.5], col);
    }
    for (let i = 0; i < seg; i++) up > 0 ? p.idx.push(c, c + 2 + i, c + 1 + i) : p.idx.push(c, c + 1 + i, c + 2 + i);
  }
}
/** A lumpy ellipsoid: a UV sphere displaced by noise (rocks, canopies, bushes). */
function blob(p: Prim, radius: V3, seg: number, lump: number, seed: number, tile = 1, col?: [number, number, number]): void {
  const base = start(p), rows = Math.max(4, Math.round(seg * 0.6));
  const at = (i: number, j: number): V3 => {
    const th = (j / rows) * Math.PI, ph = ((i % seg) / seg) * Math.PI * 2;
    const d: V3 = [Math.sin(th) * Math.cos(ph), Math.cos(th), Math.sin(th) * Math.sin(ph)];
    const k = 1 + (fbm(d[0] * 2 + 5, d[2] * 2 + d[1] * 1.7 + 5, seed, 3) - 0.5) * 2 * lump;
    return [d[0] * radius[0] * k, d[1] * radius[1] * k, d[2] * radius[2] * k];
  };
  for (let j = 0; j <= rows; j++)
    for (let i = 0; i <= seg; i++) {
      const c = at(i, j);
      const n = v3.norm(v3.cross(v3.sub(at(i, Math.min(rows, j + 1)), at(i, Math.max(0, j - 1))), v3.sub(at(i + 1, j), at(i + seg - 1, j))));
      const fallback = v3.norm(c);
      vert(p, c, Number.isFinite(n[0]) && v3.dot(n, fallback) > 0 ? n : fallback, [(i / seg) * 3 * tile, (j / rows) * 2 * tile], col);
    }
  for (let j = 0; j < rows; j++)
    for (let i = 0; i < seg; i++) {
      const a = base + j * (seg + 1) + i, b = a + seg + 1;
      p.idx.push(a, a + 1, b, a + 1, b + 1, b);
    }
}
/** An axis-aligned box from min to max, its UVs the faces' world positions over `tile` metres. */
function box(p: Prim, min: V3, max: V3, tile = 1, col?: [number, number, number]): void {
  const mb = new MeshBuilder();
  mb.box(0, min, max, { tile, color: col });
  const g = mb.groups.get(0)!;
  const base = start(p);
  p.pos.push(...g.pos);
  p.nor.push(...g.nor);
  p.uv.push(...g.uv);
  p.col.push(...g.col);
  p.idx.push(...g.idx.map((i) => i + base));
}
const yawQ = (a: number): Q => quat.axisAngle([0, 1, 0], a);

/** A model: named primitives, each with a material, written as one mesh on one node. */
function model(name: string, parts: { prim: Prim; material: Record<string, unknown> }[], textures: Record<string, Uint8Array> = {}): Uint8Array {
  const g = new Glb();
  g.json.asset.generator = "tools/bench/complex/tools/make-assets.ts";
  const texIndex: Record<string, number> = {};
  for (const [k, bytes] of Object.entries(textures)) texIndex[k] = g.texture(bytes);
  const resolve = (m: Record<string, unknown>): Record<string, unknown> =>
    JSON.parse(JSON.stringify(m), (_k, v) => (typeof v === "string" && v.startsWith("tex:") ? texIndex[v.slice(4)] : v));
  const prims = parts.map((p) => ({ pos: p.prim.pos, nor: p.prim.nor, uv: p.prim.uv, col: p.prim.col, idx: p.prim.idx, material: g.material(resolve(p.material)) }));
  if (parts.some((p) => JSON.stringify(p.material).includes("emissive_strength"))) g.json.extensionsUsed.push("KHR_materials_emissive_strength");
  g.node({ name, mesh: g.mesh(name, prims) }, true);
  return g.bytes();
}
const mat = (name: string, base: [number, number, number, number], rough: number, metal = 0, extra: Record<string, unknown> = {}, tex?: string, nrm?: string) => ({
  name,
  pbrMetallicRoughness: { baseColorFactor: base, metallicFactor: metal, roughnessFactor: rough, ...(tex ? { baseColorTexture: { index: `tex:${tex}` } } : {}) },
  ...(nrm ? { normalTexture: { index: `tex:${nrm}` } } : {}),
  ...extra,
});

// ── the ground ──────────────────────────────────────────────────────────
function ground(): Uint8Array {
  const g = new Glb();
  g.json.asset.generator = "tools/bench/complex/tools/make-assets.ts";
  const N = GRID + 1, step = SIZE / GRID, half = SIZE / 2;
  const pos = new Float32Array(N * N * 3), nor = new Float32Array(N * N * 3), uv = new Float32Array(N * N * 2);
  const h = new Float64Array(N * N);
  for (let j = 0; j < N; j++) for (let i = 0; i < N; i++) h[j * N + i] = height(-half + i * step, -half + j * step);
  for (let j = 0; j < N; j++)
    for (let i = 0; i < N; i++) {
      const k = j * N + i, x = -half + i * step, z = -half + j * step;
      pos.set([x, h[k]!, z], k * 3);
      const hx = h[j * N + Math.min(N - 1, i + 1)]! - h[j * N + Math.max(0, i - 1)]!;
      const hz = h[Math.min(N - 1, j + 1) * N + i]! - h[Math.max(0, j - 1) * N + i]!;
      nor.set(v3.norm([-hx, 2 * step, -hz]), k * 3);
      uv.set([x / 4, z / 4], k * 2);
    }
  // Each quad takes one material: sand by the water, rock on steep ground, dirt on the village and the pads, grass elsewhere.
  const lists: number[][] = [[], [], [], []];
  for (let j = 0; j < GRID; j++)
    for (let i = 0; i < GRID; i++) {
      const x = -half + (i + 0.5) * step, z = -half + (j + 0.5) * step, y = height(x, z);
      const near = (c: { x: number; z: number }, r: number) => Math.hypot(x - c.x, z - c.z) < r;
      const m = y < 1.2 && near(LAKE, LAKE.r + 14) ? 3 : slope(x, z) > 0.75 ? 2 : near(VILLAGE, VILLAGE.r * 0.85) || near(FIELD, 20) || near(PARK, 20) ? 1 : fbm(x / 9, z / 9, 41, 3) > 0.68 ? 1 : 0;
      const a = j * N + i, b = a + 1, c = a + N, d = c + 1;
      lists[m]!.push(a, c, b, b, c, d);
    }
  const P = g.accessor(pos, "VEC3", 34962, true), Nn = g.accessor(nor, "VEC3", 34962), U = g.accessor(uv, "VEC2", 34962);
  const nrm = g.texture(normalTex(51, 6, 16));
  const albedo = [
    tex(11, [0.24, 0.36, 0.12], 0.6, (u, v, n) => [0.2 + 0.08 * n, 0.33 + 0.1 * fbm(u * 32, v * 32, 12, 2, 32), 0.1]),
    tex(13, [0.38, 0.28, 0.18], 0.7),
    tex(17, [0.42, 0.41, 0.39], 0.9),
    tex(19, [0.76, 0.68, 0.5], 0.35),
  ].map((b) => g.texture(b));
  const names = ["grass", "dirt", "rock", "sand"], rough = [0.95, 0.9, 0.8, 0.92];
  const primitives = lists.map((l, m) => ({
    attributes: { POSITION: P, NORMAL: Nn, TEXCOORD_0: U },
    indices: g.accessor(new Uint32Array(l), "SCALAR", 34963),
    material: g.material({ name: names[m], pbrMetallicRoughness: { baseColorTexture: { index: albedo[m] }, metallicFactor: 0, roughnessFactor: rough[m] }, normalTexture: { index: nrm } }),
  }));
  g.json.meshes.push({ name: "ground", primitives });
  g.node({ name: "ground", mesh: 0 }, true);
  return g.bytes();
}

// ── the prop kit ────────────────────────────────────────────────────────
const BARK = tex(21, [0.3, 0.22, 0.15], 0.8, (u, _v, n) => {
  const s = 0.75 + 0.25 * Math.abs(Math.sin(u * Math.PI * 14 + n * 5));
  return [0.3 * s, 0.22 * s, 0.15 * s];
});
const LEAVES = tex(23, [0.2, 0.4, 0.12], 0.9);
const ROCK = tex(29, [0.5, 0.49, 0.46], 0.8);
const ROCK_N = normalTex(31, 5);
const WOOD = tex(37, [0.48, 0.32, 0.18], 0.3, planks);
const PLASTER = tex(41, [0.86, 0.82, 0.72], 0.25);
const ROOF = draw(T, (u, v) => {
  const row = Math.floor(v * 8), off = row % 2 ? 0.5 : 0;
  const tu = (u * 6 + off) % 1, tv = (v * 8) % 1;
  const edge = tu < 0.06 || tv > 0.9 ? 0.55 : 1;
  const tone = 0.8 + 0.3 * hash2(Math.floor(u * 6 + off), row, 3);
  return [srgb(0.55 * edge * tone), srgb(0.22 * edge * tone), srgb(0.14 * edge * tone), 255];
});
const CLOTH = draw(T, (u) => (Math.floor(u * 8) % 2 ? [srgb(0.85), srgb(0.82), srgb(0.75), 255] : [srgb(0.72), srgb(0.12), srgb(0.1), 255]));

function pine(): Uint8Array {
  const trunk = prim(), needles = prim();
  cylinder(trunk, 0.28, 0.12, 0, 3.2, 12, 1, 3);
  for (const [r, y0, y1] of [[2.4, 1.4, 5.0], [1.9, 2.9, 6.6], [1.3, 4.5, 8.2], [0.7, 6.0, 9.2]] as const) cylinder(needles, r, 0, y0, y1, 16, 1, 3);
  return model("pine", [{ prim: trunk, material: mat("pine-bark", [1, 1, 1, 1], 0.9, 0, {}, "bark") }, { prim: needles, material: mat("pine-needles", [0.55, 0.75, 0.5, 1], 0.85, 0, {}, "leaves") }], { bark: BARK, leaves: LEAVES });
}
function oak(): Uint8Array {
  const trunk = prim(), leaves = prim();
  cylinder(trunk, 0.4, 0.25, 0, 3.4, 12, 1, 3);
  for (const [ax, az, l] of [[0.6, 0.2, 2.2], [-0.5, 0.5, 2], [0.1, -0.7, 1.8]] as const) {
    const s = start(trunk);
    cylinder(trunk, 0.18, 0.08, 0, l, 8, 1, 1);
    place(trunk, s, [0, 2.8, 0], quat.between([0, 1, 0], v3.norm([ax, 1, az])));
  }
  for (const [x, y, z, r, seed] of [[0, 5.2, 0, 2.6, 1], [1.5, 4.6, 0.6, 1.9, 2], [-1.3, 4.7, 0.8, 1.8, 3], [0.2, 4.4, -1.5, 1.8, 4]] as const) {
    const s = start(leaves);
    blob(leaves, [r, r * 0.8, r], 16, 0.25, seed);
    place(leaves, s, [x, y, z]);
  }
  return model("oak", [{ prim: trunk, material: mat("oak-bark", [0.9, 0.85, 0.8, 1], 0.9, 0, {}, "bark") }, { prim: leaves, material: mat("oak-leaves", [0.75, 0.9, 0.6, 1], 0.8, 0, {}, "leaves") }], { bark: BARK, leaves: LEAVES });
}
function rock(seed: number, r: V3, moss: boolean): Uint8Array {
  const stone = prim(), green = prim();
  blob(stone, r, 18, 0.35, seed, 0.6);
  place(stone, 0, [0, r[1] * 0.45, 0]);
  const parts: { prim: Prim; material: Record<string, unknown> }[] = [{ prim: stone, material: mat(`rock-${seed}`, [1, 1, 1, 1], 0.85, 0, {}, "rock", "rockn") }];
  if (moss) {
    blob(green, [r[0] * 0.8, r[1] * 0.3, r[2] * 0.8], 12, 0.3, seed + 7);
    place(green, 0, [0, r[1] * 1.15, 0]);
    parts.push({ prim: green, material: mat("moss", [0.32, 0.45, 0.18, 1], 0.95, 0, {}, "leaves") });
  }
  return model(`rock-${seed}`, parts, { rock: ROCK, rockn: ROCK_N, ...(moss ? { leaves: LEAVES } : {}) });
}
function bush(): Uint8Array {
  const leaves = prim();
  for (const [x, z, r, seed] of [[0, 0, 0.9, 5], [0.7, 0.3, 0.7, 6], [-0.6, 0.4, 0.65, 7], [0.1, -0.6, 0.6, 8]] as const) {
    const s = start(leaves);
    blob(leaves, [r, r * 0.8, r], 12, 0.3, seed);
    place(leaves, s, [x, r * 0.6, z]);
  }
  return model("bush", [{ prim: leaves, material: mat("bush-leaves", [0.6, 0.8, 0.5, 1], 0.85, 0, {}, "leaves") }], { leaves: LEAVES });
}
function crate(): Uint8Array {
  const wood = prim(), frame = prim();
  box(wood, [-0.45, 0, -0.45], [0.45, 0.9, 0.45], 0.9);
  for (const [x, z] of [[-1, -1], [1, -1], [-1, 1], [1, 1]] as const) box(frame, [x * 0.45 - 0.05, 0, z * 0.45 - 0.05], [x * 0.45 + 0.05, 0.92, z * 0.45 + 0.05], 0.5);
  for (const y of [0, 0.84]) box(frame, [-0.48, y, -0.48], [0.48, y + 0.08, 0.48], 0.5);
  return model("crate", [{ prim: wood, material: mat("crate-wood", [1, 1, 1, 1], 0.8, 0, {}, "wood") }, { prim: frame, material: mat("crate-frame", [0.55, 0.45, 0.35, 1], 0.7, 0, {}, "wood") }], { wood: WOOD });
}
function barrel(): Uint8Array {
  const staves = prim(), iron = prim();
  cylinder(staves, 0.36, 0.42, 0, 0.55, 20, 0.5, 2);
  const s = start(staves);
  cylinder(staves, 0.42, 0.36, 0, 0.55, 20, 0.5, 2);
  place(staves, s, [0, 0.55, 0]);
  for (const y of [0.12, 0.5, 0.96]) {
    const t = start(iron);
    cylinder(iron, 0.43, 0.43, 0, 0.05, 20);
    place(iron, t, [0, y, 0]);
  }
  return model("barrel", [{ prim: staves, material: mat("barrel-wood", [1, 1, 1, 1], 0.75, 0, {}, "wood") }, { prim: iron, material: mat("iron", [0.3, 0.3, 0.32, 1], 0.45, 1) }], { wood: WOOD });
}
function fence(): Uint8Array {
  const wood = prim();
  for (const x of [-1.5, 1.5]) box(wood, [x - 0.07, 0, -0.07], [x + 0.07, 1.2, 0.07], 0.5);
  for (const y of [0.45, 0.95]) box(wood, [-1.55, y, -0.04], [1.55, y + 0.12, 0.04], 1);
  return model("fence", [{ prim: wood, material: mat("fence-wood", [0.85, 0.8, 0.75, 1], 0.85, 0, {}, "wood") }], { wood: WOOD });
}
function lamppost(): Uint8Array {
  const iron = prim(), glass = prim();
  cylinder(iron, 0.1, 0.07, 0, 4.2, 10, 1, 2);
  cylinder(iron, 0.22, 0.22, 0, 0.3, 10);
  box(iron, [-0.04, 4.0, -0.04], [0.8, 4.1, 0.04]);
  box(iron, [0.62, 3.95, -0.18], [0.98, 4.02, 0.18]);
  box(glass, [0.66, 3.55, -0.14], [0.94, 3.95, 0.14]);
  return model("lamppost", [{ prim: iron, material: mat("lamp-iron", [0.12, 0.12, 0.13, 1], 0.5, 1) }, { prim: glass, material: mat("lamp-glass", [1, 0.9, 0.7, 1], 0.3, 0, { emissiveFactor: [1, 0.78, 0.45], extensions: { KHR_materials_emissive_strength: { emissiveStrength: 4 } } }) }]);
}
function house(): Uint8Array {
  const walls = prim(), roof = prim(), wood = prim(), glass = prim(), stone = prim();
  box(walls, [-3, 0, -2.5], [3, 4, 2.5], 2);
  // Gables: two triangles under the roof, as thin boxes stepped up.
  for (let k = 0; k < 6; k++) box(walls, [-3 + k * 0.5, 4 + k * 0.33, -2.45], [3 - k * 0.5, 4.33 + k * 0.33, 2.45], 2);
  for (const side of [-1, 1]) {
    const s = start(roof);
    box(roof, [-0.15, -0.12, -3.3], [3.6, 0.12, 3.3], 2);
    place(roof, s, [side * 0.05, 6.05, 0], quat.axisAngle([0, 0, 1], side > 0 ? -0.58 : Math.PI + 0.58), [1, 1, 1]);
  }
  box(wood, [-0.6, 0, 2.5], [0.6, 2.2, 2.62], 1);
  for (const x of [-3.05, 3.05]) box(wood, [x - 0.1, 0, -2.6], [x + 0.1, 4, -2.4], 1), box(wood, [x - 0.1, 0, 2.4], [x + 0.1, 4, 2.6], 1);
  box(wood, [-3.1, 3.9, -2.6], [3.1, 4.05, 2.6], 1);
  for (const [x, z, w, d] of [[-1.9, 2.5, 1, 0.08], [1.9, 2.5, 1, 0.08], [-1.9, -2.58, 1, 0.08], [1.9, -2.58, 1, 0.08], [-3.08, 0, 0.08, 1.1], [3.0, 0, 0.08, 1.1]] as const) {
    box(glass, [x - w / 2, 1.3, z - d / 2 + (z > 0 ? 0.04 : 0)], [x + w / 2, 2.5, z + d / 2 + (z > 0 ? 0.04 : 0)], 1);
  }
  box(stone, [1.8, 4.5, -1.8], [2.6, 7.4, -1.0], 1);
  return model(
    "house",
    [
      { prim: walls, material: mat("plaster", [1, 1, 1, 1], 0.9, 0, {}, "plaster") },
      { prim: roof, material: mat("roof-tiles", [1, 1, 1, 1], 0.7, 0, {}, "roof") },
      { prim: wood, material: mat("house-wood", [0.7, 0.6, 0.55, 1], 0.8, 0, {}, "wood") },
      { prim: stone, material: mat("chimney", [0.8, 0.78, 0.75, 1], 0.9, 0, {}, "rock", "rockn") },
      { prim: glass, material: mat("window-glass", [0.55, 0.7, 0.85, 0.45], 0.05, 0, { alphaMode: "BLEND" }) },
    ],
    { plaster: PLASTER, roof: ROOF, wood: WOOD, rock: ROCK, rockn: ROCK_N },
  );
}
function stall(): Uint8Array {
  const wood = prim(), cloth = prim(), side = prim(), goods = prim();
  for (const [x, z] of [[-1.4, -0.9], [1.4, -0.9], [-1.4, 0.9], [1.4, 0.9]] as const) box(wood, [x - 0.06, 0, z - 0.06], [x + 0.06, z < 0 ? 2.6 : 2.2, z + 0.06], 0.5);
  box(wood, [-1.5, 0.85, -0.95], [1.5, 0.95, 0.95], 1);
  const s = start(cloth);
  box(cloth, [-1.6, -0.03, -1.15], [1.6, 0.03, 1.15], 3.2);
  place(cloth, s, [0, 2.42, 0], quat.axisAngle([1, 0, 0], -0.18));
  box(side, [-1.55, 1.6, -1.0], [-1.45, 2.4, 1.0], 2);
  box(side, [1.45, 1.6, -1.0], [1.55, 2.4, 1.0], 2);
  for (let i = 0; i < 6; i++) {
    const t = start(goods);
    blob(goods, [0.22, 0.15, 0.22], 8, 0.2, 60 + i);
    place(goods, t, [-1.1 + i * 0.44, 1.08, 0.2 * ((i % 2) * 2 - 1)]);
  }
  return model(
    "stall",
    [
      { prim: wood, material: mat("stall-wood", [0.9, 0.8, 0.7, 1], 0.8, 0, {}, "wood") },
      { prim: cloth, material: mat("stall-cloth", [1, 1, 1, 1], 0.95, 0, { doubleSided: true }, "cloth") },
      { prim: side, material: mat("stall-side", [0.2, 0.3, 0.7, 1], 0.95) },
      { prim: goods, material: mat("stall-goods", [0.9, 0.5, 0.15, 1], 0.6) },
    ],
    { wood: WOOD, cloth: CLOTH },
  );
}

// ── the villager: kestrel-stair's library skeleton, a box body, the library's clips ──
(BODIES as Record<string, unknown>)["villager"] = {
  scheme: "blender",
  // examples/kestrel-stair/tools/rig.ts's LIBRARY proportions, so the body's rest pose is the clips'.
  prop: {
    hips: 1.0, spine: 0.12, chest: 0.2, neck: 0.2, head: 0.1, headSize: 0.22,
    shoulderX: 0.04, shoulderY: 0.06, clavicle: 0.14, upperArm: 0.28, lowerArm: 0.26, hand: 0.14, armDrop: 0,
    legX: 0.1, hipDrop: 0.02, upperLeg: 0.44, lowerLeg: 0.44, footHeight: 0.08, foot: 0.14, toes: 0.08,
  },
  build: (mb: MeshBuilder, h: Record<string, V3>, t: Record<string, V3>, j: (b: string) => number, m: { body: number; glow: number }) => {
    const limb = (joint: string, a: V3, b: V3, w: number, d: number, color: [number, number, number]) => {
      const y = v3.sub(b, a), len = v3.len(y), yn = v3.norm(y);
      const ref: V3 = Math.abs(yn[2]) > 0.9 ? [1, 0, 0] : [0, 0, 1];
      const x = v3.norm(v3.cross(yn, ref)), z = v3.cross(x, yn);
      mb.box(m.body, [-w / 2, 0, -d / 2], [w / 2, len, d / 2], { frame: { o: a, x, y: yn, z }, joint: j(joint), color });
    };
    const TUNIC: [number, number, number] = [0.25, 0.45, 0.3], SKIN: [number, number, number] = [0.85, 0.64, 0.5], TROUSERS: [number, number, number] = [0.35, 0.28, 0.2];
    limb("hips", v3.add(h.hips!, [0, -0.1, 0]), h.spine!, 0.32, 0.2, TROUSERS);
    limb("spine", h.spine!, h.chest!, 0.34, 0.22, TUNIC);
    limb("chest", h.chest!, h.neck!, 0.4, 0.24, TUNIC);
    limb("neck", h.neck!, h.head!, 0.1, 0.1, SKIN);
    limb("head", h.head!, t.head!, 0.22, 0.24, SKIN);
    for (const s of ["L", "R"]) {
      limb(`shoulder${s}`, h[`shoulder${s}`]!, h[`upperArm${s}`]!, 0.1, 0.12, TUNIC);
      limb(`upperArm${s}`, h[`upperArm${s}`]!, h[`lowerArm${s}`]!, 0.1, 0.1, TUNIC);
      limb(`lowerArm${s}`, h[`lowerArm${s}`]!, h[`hand${s}`]!, 0.09, 0.09, SKIN);
      limb(`hand${s}`, h[`hand${s}`]!, t[`hand${s}`]!, 0.08, 0.06, SKIN);
      limb(`upperLeg${s}`, h[`upperLeg${s}`]!, h[`lowerLeg${s}`]!, 0.14, 0.15, TROUSERS);
      limb(`lowerLeg${s}`, h[`lowerLeg${s}`]!, h[`foot${s}`]!, 0.12, 0.13, TROUSERS);
      limb(`foot${s}`, h[`foot${s}`]!, h[`toes${s}`]!, 0.11, 0.1, [0.2, 0.15, 0.1]);
      limb(`toes${s}`, h[`toes${s}`]!, t[`toes${s}`]!, 0.11, 0.06, [0.2, 0.15, 0.1]);
    }
  },
};

/** A .glb's JSON and binary chunk. */
function split(glb: Uint8Array): { json: Record<string, any>; bin: Uint8Array } {
  const dv = new DataView(glb.buffer, glb.byteOffset, glb.byteLength);
  const jl = dv.getUint32(12, true);
  const json = JSON.parse(new TextDecoder().decode(glb.subarray(20, 20 + jl)));
  const bl = dv.getUint32(20 + jl, true);
  return { json, bin: glb.subarray(28 + jl, 28 + jl + bl) };
}
function join2(json: Record<string, unknown>, bin: Uint8Array): Uint8Array {
  const text = new TextEncoder().encode(JSON.stringify(json));
  const jpad = (4 - (text.length % 4)) % 4, bpad = (4 - (bin.length % 4)) % 4;
  const total = 28 + text.length + jpad + bin.length + bpad;
  const out = new Uint8Array(total);
  const dv = new DataView(out.buffer);
  dv.setUint32(0, 0x46546c67, true);
  dv.setUint32(4, 2, true);
  dv.setUint32(8, total, true);
  dv.setUint32(12, text.length + jpad, true);
  dv.setUint32(16, 0x4e4f534a, true);
  out.set(text, 20);
  out.fill(0x20, 20 + text.length, 20 + text.length + jpad);
  const at = 20 + text.length + jpad;
  dv.setUint32(at, bin.length + bpad, true);
  dv.setUint32(at + 4, 0x004e4942, true);
  out.set(bin, at + 8);
  return out;
}
/** The villager with the library's clips moved into its file: the same joint names and rest offsets, so the channels retarget by name. */
function villager(): Uint8Array {
  const body = split(character("villager" as "courier"));
  const lib = split(library());
  const at = Math.ceil(body.bin.length / 4) * 4;
  const bin = new Uint8Array(at + lib.bin.length);
  bin.set(body.bin);
  bin.set(lib.bin, at);
  const views = body.json.bufferViews.length, accessors = body.json.accessors.length;
  body.json.bufferViews.push(...lib.json.bufferViews.map((v: any) => ({ ...v, byteOffset: (v.byteOffset ?? 0) + at })));
  body.json.accessors.push(...lib.json.accessors.map((a: any) => ({ ...a, bufferView: a.bufferView + views })));
  const byName = new Map(body.json.nodes.map((n: any, i: number) => [n.name, i]));
  body.json.animations = lib.json.animations.map((a: any) => ({
    ...a,
    samplers: a.samplers.map((s: any) => ({ ...s, input: s.input + accessors, output: s.output + accessors })),
    channels: a.channels.map((c: any) => {
      const node = byName.get(lib.json.nodes[c.target.node].name);
      if (node === undefined) throw new Error(`villager: no joint ${lib.json.nodes[c.target.node].name}`);
      return { ...c, target: { ...c.target, node } };
    }),
  }));
  body.json.buffers = [{ byteLength: bin.length }];
  body.json.asset.generator = "tools/bench/complex/tools/make-assets.ts";
  return join2(body.json, bin);
}

// ── particles ───────────────────────────────────────────────────────────
const puff = (seed: number, hard: number) =>
  draw(64, (u, v, x, y) => {
    const d = Math.hypot(u - 0.5, v - 0.5) * 2;
    const n = 0.7 + 0.3 * hash2(x >> 2, y >> 2, seed);
    return [255, 255, 255, Math.max(0, 1 - d) ** hard * n * 255];
  });
function emitter(name: keyof typeof FX): string {
  const f = FX[name];
  return `type particles
version 1
---
# ${name === "fire" ? "A campfire's flames, added" : "Smoke, alpha blended"}: written by tools/make-assets.ts from src/layout.js's FX table.
maxParticles ${f.max}
shape cone ${f.cone} ${f.radius}
rate ${f.rate}
lifetime ${f.life[0]} ${f.life[1]}
speed ${f.speed[0]} ${f.speed[1]}
size ${f.size[0]} ${f.size[1]}
sizeOverLife 0 ${f.sizeOverLife[0]} 1 ${f.sizeOverLife[1]}
colorOverLife {
${f.color.map((k) => `  key ${k.join(" ")}`).join("\n")}
}
gravity 0 ${f.gravity} 0
texture "${f.texture}"
blend ${f.blend}
`;
}

// ── write ───────────────────────────────────────────────────────────────
const t0 = performance.now();
write("models/ground.glb", ground());
write("models/pine.glb", pine());
write("models/oak.glb", oak());
write("models/rock-a.glb", rock(1, [1, 0.7, 0.9], false));
write("models/rock-b.glb", rock(2, [1.2, 0.8, 1], true));
write("models/rock-c.glb", rock(3, [0.7, 1.1, 0.7], false));
write("models/bush.glb", bush());
write("models/crate.glb", crate());
write("models/barrel.glb", barrel());
write("models/fence.glb", fence());
write("models/lamppost.glb", lamppost());
write("models/house.glb", house());
write("models/stall.glb", stall());
write("models/villager.glb", villager());
const copies: [string, string][] = [
  ["crates/og-tests/reference-scene/assets/models/walker.glb", "walker.glb"],
  ["examples/rumpus-isle/assets/models/palm.glb", "palm.glb"],
  ["examples/rumpus-isle/assets/models/berry.glb", "berry.glb"],
  ["examples/rumpus-isle/assets/models/islet.glb", "islet.glb"],
  ["examples/meridian-loop/assets/models/car.glb", "car.glb"],
  ["examples/meridian-loop/assets/models/ghost.glb", "ghost.glb"],
  ["examples/kestrel-stair/assets/models/stair.glb", "stair.glb"],
  ["templates/game/assets/models/dropship.glb", "dropship.glb"],
];
for (const [from, to] of copies) copyFileSync(join(repo, from), join(assets, "models", to));
write("textures/flame.png", puff(3, 1.4));
write("textures/smoke.png", puff(5, 1.8));
write("fx/fire.ogt", emitter("fire"));
write("fx/smoke.ogt", emitter("smoke"));
void rng;
process.stderr.write(`complex scene assets written in ${Math.round(performance.now() - t0)} ms\n`);
