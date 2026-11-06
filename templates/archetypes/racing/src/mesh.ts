// The circuit's and the cars' meshes, built from numbers: flat-shaded
// faces with a colour each (og.render.bufferGeometry's vertex colours,
// multiplied into the material's), so a whole layer of the circuit is one
// draw. The engine's math for every angle: the same triangles on every host.

import { math } from "@origingame/engine";
import { COUNT, SAMPLES, SECTORS, LENGTH, WALL, WIDTH, type Sample } from "./track.ts";

export type V3 = readonly [number, number, number];
export type RGB = readonly [number, number, number];

/** Faces with a colour each; every face has its own vertices, so its normal is its own (flat shading). */
export class Builder {
  readonly positions: number[] = [];
  readonly normals: number[] = [];
  readonly colors: number[] = [];
  readonly indices: number[] = [];

  /** A triangle, wound to face `facing` (a direction it should face). */
  tri(a: V3, b: V3, c: V3, color: RGB, facing?: V3): void {
    let n = normal(a, b, c);
    if (facing && n[0] * facing[0] + n[1] * facing[1] + n[2] * facing[2] < 0) {
      [b, c] = [c, b];
      n = [-n[0], -n[1], -n[2]];
    }
    const base = this.positions.length / 3;
    for (const p of [a, b, c]) {
      this.positions.push(p[0], p[1], p[2]);
      this.normals.push(n[0], n[1], n[2]);
      this.colors.push(color[0], color[1], color[2]);
    }
    this.indices.push(base, base + 1, base + 2);
  }

  /** A quad a b c d (in order round its edge), wound to face `facing`. */
  quad(a: V3, b: V3, c: V3, d: V3, color: RGB, facing?: V3): void {
    let n = normal(a, b, c);
    if (n[0] === 0 && n[1] === 0 && n[2] === 0) n = normal(a, c, d);
    if (facing && n[0] * facing[0] + n[1] * facing[1] + n[2] * facing[2] < 0) {
      [b, d] = [d, b];
      n = [-n[0], -n[1], -n[2]];
    }
    const base = this.positions.length / 3;
    for (const p of [a, b, c, d]) {
      this.positions.push(p[0], p[1], p[2]);
      this.normals.push(n[0], n[1], n[2]);
      this.colors.push(color[0], color[1], color[2]);
    }
    this.indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
  }

  /** An axis-aligned box from its centre and half-sizes. */
  box(c: V3, h: V3, color: RGB, top?: RGB): void {
    const [x, y, z] = c;
    const [a, b, d] = h;
    const p = (i: number, j: number, k: number): V3 => [x + i * a, y + j * b, z + k * d];
    this.quad(p(-1, 1, -1), p(1, 1, -1), p(1, 1, 1), p(-1, 1, 1), top ?? color, [0, 1, 0]);
    this.quad(p(-1, -1, -1), p(1, -1, -1), p(1, -1, 1), p(-1, -1, 1), color, [0, -1, 0]);
    this.quad(p(-1, -1, 1), p(1, -1, 1), p(1, 1, 1), p(-1, 1, 1), color, [0, 0, 1]);
    this.quad(p(-1, -1, -1), p(1, -1, -1), p(1, 1, -1), p(-1, 1, -1), color, [0, 0, -1]);
    this.quad(p(1, -1, -1), p(1, -1, 1), p(1, 1, 1), p(1, 1, -1), color, [1, 0, 0]);
    this.quad(p(-1, -1, -1), p(-1, -1, 1), p(-1, 1, 1), p(-1, 1, -1), color, [-1, 0, 0]);
  }

  /** The geometry's arrays, as og.render.bufferGeometry takes them. */
  arrays(): { positions: Float32Array; normals: Float32Array; colors: Float32Array; indices: Uint32Array } {
    return { positions: new Float32Array(this.positions), normals: new Float32Array(this.normals), colors: new Float32Array(this.colors), indices: new Uint32Array(this.indices) };
  }
}

function normal(a: V3, b: V3, c: V3): V3 {
  const ux = b[0] - a[0], uy = b[1] - a[1], uz = b[2] - a[2];
  const vx = c[0] - a[0], vy = c[1] - a[1], vz = c[2] - a[2];
  const nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
  const l = Math.sqrt(nx * nx + ny * ny + nz * nz);
  return l > 0 ? [nx / l, ny / l, nz / l] : [0, 0, 0];
}

/** A hash of two integers, 0 to 1: the circuit's patchwork, the same everywhere. */
export function hash(a: number, b: number): number {
  let h = Math.imul(a | 0, 374761393) ^ Math.imul(b | 0, 668265263);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

const at = (p: Sample, across: number, y: number): V3 => [p.x + p.rx * across, y, p.z + p.rz * across];
const shade = (c: RGB, k: number): RGB => [c[0] * k, c[1] * k, c[2] * k];

const ASPHALT: RGB = [0.12, 0.125, 0.135];
const RUBBER: RGB = [0.075, 0.075, 0.08];
const PAINT: RGB = [0.92, 0.92, 0.9];
const KERB_RED: RGB = [0.82, 0.12, 0.1];
const GRASS: RGB = [0.22, 0.48, 0.14];
const VERGE: RGB = [0.46, 0.43, 0.35];

/** Whether the road bends enough at a sample for kerbs (over a few samples, so a kerb is a run). */
export function kerbAt(i: number): boolean {
  let k = 0;
  for (let j = -3; j <= 3; j++) k = Math.max(k, Math.abs(SAMPLES[(i + j + COUNT) % COUNT]!.k));
  return k > 1 / 55;
}

/**
 * The road layer: the asphalt (darker where the racing line has laid
 * rubber), its edge lines, the kerbs on the bends, a gravel verge to the
 * barriers, the start line's chequer and the grid's boxes.
 */
export function roadMesh(line: Float64Array, grid: readonly { along: number; across: number }[]): Builder {
  const b = new Builder();
  const up: V3 = [0, 1, 0];
  const lanes = 8;
  const half = WIDTH / 2;
  for (let i = 0; i < COUNT; i++) {
    const p = SAMPLES[i]!;
    const q = SAMPLES[(i + 1) % COUNT]!;
    const o = (line[i]! + line[(i + 1) % COUNT]!) / 2;
    for (let l = 0; l < lanes; l++) {
      const a0 = -half + (WIDTH * l) / lanes;
      const a1 = -half + (WIDTH * (l + 1)) / lanes;
      const mid = (a0 + a1) / 2;
      const rubber = Math.max(0, 1 - Math.abs(mid - o) / 2.4);
      const base: RGB = [ASPHALT[0] + (RUBBER[0] - ASPHALT[0]) * rubber, ASPHALT[1] + (RUBBER[1] - ASPHALT[1]) * rubber, ASPHALT[2] + (RUBBER[2] - ASPHALT[2]) * rubber];
      b.quad(at(p, a0, 0.02), at(p, a1, 0.02), at(q, a1, 0.02), at(q, a0, 0.02), shade(base, 0.93 + 0.1 * hash(i, l)), up);
    }
    // The edge lines, 0.3 m wide, 0.4 m inside the edge.
    for (const s of [-1, 1]) {
      const a0 = s * (half - 0.7);
      const a1 = s * (half - 0.4);
      b.quad(at(p, a0, 0.03), at(p, a1, 0.03), at(q, a1, 0.03), at(q, a0, 0.03), PAINT, up);
    }
    // Kerbs on the bends (red and white, a sample each), a gravel verge to the barrier elsewhere.
    const kerb = kerbAt(i);
    for (const s of [-1, 1]) {
      const a0 = s * half;
      const a1 = s * (half + 1.1);
      if (kerb) b.quad(at(p, a0, 0.035), at(p, a1, 0.06), at(q, a1, 0.06), at(q, a0, 0.035), i % 2 === 0 ? KERB_RED : PAINT, up);
      else b.quad(at(p, a0, 0.02), at(p, a1, 0.02), at(q, a1, 0.02), at(q, a0, 0.02), shade(VERGE, 0.95 + 0.08 * hash(i, 9 + s)), up);
      b.quad(at(p, s * (half + 1.1), 0.02), at(p, s * (WALL + 0.2), 0.02), at(q, s * (WALL + 0.2), 0.02), at(q, s * (half + 1.1), 0.02), shade(VERGE, 0.9 + 0.08 * hash(i, 19 + s)), up);
    }
  }
  // The start line: a chequer two squares deep across the road at s = 0.
  const p0 = SAMPLES[0]!;
  const sq = WIDTH / 13;
  for (let r = 0; r < 2; r++) {
    for (let c = 0; c < 13; c++) {
      const a0 = -half + c * sq;
      const a1 = a0 + sq;
      const s0 = (r - 1) * sq;
      const s1 = s0 + sq;
      const pt = (across: number, s: number): V3 => [p0.x + p0.rx * across + p0.tx * s, 0.04, p0.z + p0.rz * across + p0.tz * s];
      b.quad(pt(a0, s0), pt(a1, s0), pt(a1, s1), pt(a0, s1), (r + c) % 2 === 0 ? PAINT : [0.05, 0.05, 0.06], up);
    }
  }
  // The grid's boxes: a white bracket in front of each slot.
  for (const g of grid) {
    const i = Math.floor((g.along / LENGTH) * COUNT) % COUNT;
    const p = SAMPLES[i]!;
    const ds = g.along - p.s + 2.4;
    const pt = (across: number, s: number): V3 => [p.x + p.rx * across + p.tx * s, 0.035, p.z + p.rz * across + p.tz * s];
    b.quad(pt(g.across - 1.3, ds), pt(g.across + 1.3, ds), pt(g.across + 1.3, ds + 0.25), pt(g.across - 1.3, ds + 0.25), PAINT, up);
    for (const e of [-1.3, 1.05]) b.quad(pt(g.across + e, ds - 1.2), pt(g.across + e + 0.25, ds - 1.2), pt(g.across + e + 0.25, ds + 0.25), pt(g.across + e, ds + 0.25), PAINT, up);
  }
  return b;
}

/** The barriers both sides: an armco rail on a low wall, red and white blocks on the faces the cars meet. */
export function barrierMesh(): Builder {
  const b = new Builder();
  const h = 0.95;
  const t = 0.35;
  for (let i = 0; i < COUNT; i++) {
    const p = SAMPLES[i]!;
    const q = SAMPLES[(i + 1) % COUNT]!;
    const block = Math.floor(i / 2) % 2 === 0;
    for (const s of [-1, 1]) {
      const inner = s * WALL;
      const outer = s * (WALL + t);
      const face: RGB = block ? [0.86, 0.1, 0.08] : [0.94, 0.94, 0.92];
      const facing: V3 = [-s * p.rx, 0, -s * p.rz];
      b.quad(at(p, inner, 0), at(q, inner, 0), at(q, inner, h), at(p, inner, h), face, facing);
      b.quad(at(p, inner, h), at(q, inner, h), at(q, outer, h), at(p, outer, h), [0.62, 0.64, 0.66], [0, 1, 0]);
      b.quad(at(p, outer, 0), at(q, outer, 0), at(q, outer, h), at(p, outer, h), [0.45, 0.46, 0.47], [s * p.rx, 0, s * p.rz]);
      // The rail along the top of the face.
      b.quad(at(p, inner - s * 0.06, h - 0.32), at(q, inner - s * 0.06, h - 0.32), at(q, inner - s * 0.06, h - 0.12), at(p, inner - s * 0.06, h - 0.12), [0.78, 0.8, 0.82], facing);
    }
  }
  return b;
}

/** The ground: a field of grass in a patchwork of mown stripes, 1.2 km square. */
export function groundMesh(): Builder {
  const b = new Builder();
  const cell = 24;
  const n = 50;
  const up: V3 = [0, 1, 0];
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      const x0 = (i - n / 2) * cell + 60;
      const z0 = (j - n / 2) * cell;
      const stripe = (i + j) % 2 === 0 ? 1.04 : 0.95;
      const k = stripe * (0.94 + 0.1 * hash(i, j));
      b.quad([x0, 0, z0], [x0 + cell, 0, z0], [x0 + cell, 0, z0 + cell], [x0, 0, z0 + cell], shade(GRASS, k), up);
    }
  }
  return b;
}

/** Whether a point is clear of the circuit by `room` metres beyond the barriers. */
export function clearOfTrack(x: number, z: number, room: number): boolean {
  const r = (WALL + room) ** 2;
  for (const p of SAMPLES) if ((p.x - x) ** 2 + (p.z - z) ** 2 < r) return false;
  return true;
}

/** A low-poly tree: a trunk and two stacked cones, `k` its size. */
function tree(b: Builder, x: number, z: number, k: number, tint: number): void {
  const leaf: RGB = [0.16 + 0.06 * tint, 0.4 + 0.08 * tint, 0.15];
  const dark = shade(leaf, 0.8);
  const sides = 7;
  const ring = (r: number, y: number, a: number): V3 => [x + r * math.cos(a), y, z + r * math.sin(a)];
  b.box([x, 0.9 * k, z], [0.22 * k, 0.9 * k, 0.22 * k], [0.4, 0.27, 0.16]);
  for (const [r, y0, y1, c] of [[1.7, 1.4, 4.2, dark], [1.25, 3, 5.6, leaf]] as const) {
    for (let s = 0; s < sides; s++) {
      const a0 = (s / sides) * Math.PI * 2;
      const a1 = ((s + 1) / sides) * Math.PI * 2;
      b.tri(ring(r * k, y0 * k, a0), ring(r * k, y0 * k, a1), [x, y1 * k, z], shade(c, 0.9 + 0.15 * hash(s, Math.round(x))), [math.cos((a0 + a1) / 2), 0.5, math.sin((a0 + a1) / 2)]);
    }
  }
}

/** Trees round the circuit (clear of the barriers), hills on the horizon. */
export function sceneryMesh(): Builder {
  const b = new Builder();
  let planted = 0;
  for (let n = 0; n < 900 && planted < 170; n++) {
    const x = -140 + hash(n, 1) * 400;
    const z = -230 + hash(n, 2) * 460;
    if (!clearOfTrack(x, z, 5)) continue;
    // Not on the grandstand's ground by the start straight.
    if (x < -8 && x > -40 && z > -40 && z < 80) continue;
    tree(b, x, z, 0.8 + 0.6 * hash(n, 3), hash(n, 4));
    planted++;
  }
  // Hills: a ring of low faceted mounds far out, darker and bluer with distance.
  for (let h = 0; h < 22; h++) {
    const a = (h / 22) * Math.PI * 2;
    const r = 520 + 80 * hash(h, 5);
    const cx = 60 + r * math.cos(a);
    const cz = r * math.sin(a);
    const height = 50 + 70 * hash(h, 6);
    const base = 120 + 60 * hash(h, 7);
    const col: RGB = [0.32, 0.45, 0.36];
    const sides = 6;
    for (let s = 0; s < sides; s++) {
      const a0 = (s / sides) * Math.PI * 2 + h;
      const a1 = ((s + 1) / sides) * Math.PI * 2 + h;
      const p0: V3 = [cx + base * math.cos(a0), -2, cz + base * math.sin(a0)];
      const p1: V3 = [cx + base * math.cos(a1), -2, cz + base * math.sin(a1)];
      b.tri(p0, p1, [cx, height, cz], shade(col, 0.8 + 0.3 * hash(h, s)), [math.cos((a0 + a1) / 2), 0.6, math.sin((a0 + a1) / 2)]);
    }
  }
  return b;
}

/**
 * The grandstand along the start straight's left and its crowd: stepped
 * terraces, a roof on posts, the people as dabs of colour.
 */
export function standMesh(): Builder {
  const b = new Builder();
  const x0 = -WALL - 5;
  const z0 = -30;
  const z1 = 60;
  const rows = 7;
  const crowd: RGB[] = [[0.85, 0.2, 0.15], [0.95, 0.8, 0.2], [0.2, 0.45, 0.85], [0.95, 0.95, 0.95], [0.15, 0.6, 0.35], [0.9, 0.5, 0.15]];
  for (let r = 0; r < rows; r++) {
    const x = x0 - r * 1.4;
    const y = 0.6 + r * 0.7;
    b.box([x - 0.7, y / 2, (z0 + z1) / 2], [0.7, y / 2, (z1 - z0) / 2], [0.58, 0.6, 0.64], [0.7, 0.72, 0.75]);
    for (let s = 0; s < (z1 - z0) / 0.9; s++) {
      if (hash(r, s) < 0.18) continue;
      const z = z0 + 0.45 + s * 0.9;
      b.box([x - 0.7, y + 0.35, z], [0.22, 0.35, 0.25], crowd[Math.floor(hash(s, r + 40) * crowd.length)]!);
    }
  }
  const back = x0 - rows * 1.4;
  b.box([back - 0.3, 3.6, (z0 + z1) / 2], [0.3, 3.6, (z1 - z0) / 2], [0.36, 0.38, 0.42]);
  for (let z = z0; z <= z1; z += 15) b.box([x0 + 0.3, 3.4, z], [0.15, 3.4, 0.15], [0.3, 0.3, 0.32]);
  b.box([(x0 + back) / 2, 7.1, (z0 + z1) / 2], [(x0 - back) / 2 + 1.2, 0.15, (z1 - z0) / 2 + 1], [0.9, 0.92, 0.95], [0.85, 0.2, 0.15]);
  return b;
}

/** The start gantry over the line: two towers and a beam with the race's banner (its lights are their own meshes). */
export function gantryMesh(): Builder {
  const b = new Builder();
  const p = SAMPLES[0]!;
  const post = (s: number) => [p.x + p.rx * s * (WALL + 0.9), p.z + p.rz * s * (WALL + 0.9)] as const;
  for (const s of [-1, 1]) {
    const [x, z] = post(s);
    b.box([x, 3.6, z], [0.45, 3.6, 0.45], [0.2, 0.21, 0.24]);
  }
  // The beam runs across the road (the road runs along z here).
  b.box([p.x, 7.6, p.z], [WALL + 1.4, 0.55, 0.4], [0.14, 0.15, 0.18], [0.85, 0.2, 0.15]);
  return b;
}

/** A checkpoint's arch at a fraction of the lap: two posts and a banner across the road. */
export function gateMesh(): Builder {
  const b = new Builder();
  for (const f of SECTORS) {
    const i = Math.round(f * COUNT) % COUNT;
    const p = SAMPLES[i]!;
    for (const s of [-1, 1]) {
      const x = p.x + p.rx * s * (WALL + 0.6);
      const z = p.z + p.rz * s * (WALL + 0.6);
      b.box([x, 3, z], [0.25, 3, 0.25], [0.9, 0.9, 0.92]);
    }
    // The banner: a thin slab across the road, square to the tangent.
    const w = WALL + 0.6;
    const y0 = 5.4, y1 = 6.4;
    const l = (s: number, y: number, d: number): V3 => [p.x + p.rx * s * w + p.tx * d, y, p.z + p.rz * s * w + p.tz * d];
    const yellow: RGB = [0.98, 0.78, 0.12];
    b.quad(l(-1, y0, -0.1), l(1, y0, -0.1), l(1, y1, -0.1), l(-1, y1, -0.1), yellow, [-p.tx, 0, -p.tz]);
    b.quad(l(-1, y0, 0.1), l(1, y0, 0.1), l(1, y1, 0.1), l(-1, y1, 0.1), yellow, [p.tx, 0, p.tz]);
    b.quad(l(-1, y1, -0.1), l(1, y1, -0.1), l(1, y1, 0.1), l(-1, y1, 0.1), [0.2, 0.2, 0.22], [0, 1, 0]);
    // Its chequered band.
    for (let c = 0; c < 16; c++) {
      const s0 = -1 + (2 * c) / 16;
      const s1 = s0 + 2 / 16;
      const k: RGB = c % 2 === 0 ? [0.05, 0.05, 0.06] : [0.95, 0.95, 0.95];
      for (const d of [-0.12, 0.12]) b.quad(l(s0, y0 + 0.05, d), l(s1, y0 + 0.05, d), l(s1, y0 + 0.3, d), l(s0, y0 + 0.3, d), k, [p.tx * d, 0, p.tz * d]);
    }
  }
  return b;
}

// ---- The car -------------------------------------------------------------
//
// Car frame: forward -z, right +x, up +y, the origin on the ground under its
// middle. A part is a prism of a side profile (forward, up, half-width at
// that point), so the flanks lean in towards the roof.

type Profile = readonly (readonly [number, number, number])[];

function prism(b: Builder, profile: Profile, color: RGB): void {
  const n = profile.length;
  const L = (i: number): V3 => [-profile[i]![2], profile[i]![1], -profile[i]![0]];
  const R = (i: number): V3 => [profile[i]![2], profile[i]![1], -profile[i]![0]];
  // The flanks: a fan of each side's convex profile.
  for (let i = 1; i < n - 1; i++) {
    b.tri(L(0), L(i), L(i + 1), shade(color, 0.86), [-1, 0, 0]);
    b.tri(R(0), R(i), R(i + 1), shade(color, 0.86), [1, 0, 0]);
  }
  // The skin round the profile, each face lit by its own slope.
  let cx = 0, cy = 0;
  for (const p of profile) {
    cx += p[0] / n;
    cy += p[1] / n;
  }
  for (let i = 0; i < n; i++) {
    const j = (i + 1) % n;
    const mu = (profile[i]![0] + profile[j]![0]) / 2 - cx;
    const mv = (profile[i]![1] + profile[j]![1]) / 2 - cy;
    b.quad(L(i), R(i), R(j), L(j), color, [0, mv, -mu]);
  }
}

/** The paint: the lower body, the nose and the tail (the livery's colour). */
export function carPaint(): Builder {
  const b = new Builder();
  prism(b, [
    [2.15, 0.3, 0.86],
    [2.2, 0.52, 0.86],
    [1.95, 0.72, 0.82],
    [0.8, 0.82, 0.84],
    [-1.6, 0.86, 0.86],
    [-2.12, 0.82, 0.84],
    [-2.18, 0.42, 0.86],
    [-2.0, 0.28, 0.86],
  ], [1, 1, 1]);
  return b;
}

/** The racing stripes: two bands raised a hair over the bonnet and the boot (white). */
export function carStripe(): Builder {
  const b = new Builder();
  for (const [u0, u1, v0, v1] of [[0.82, 1.93, 0.826, 0.74], [-1.63, -2.08, 0.862, 0.83]] as const) {
    for (const x of [-0.34, 0.14]) b.quad([x, v0 + 0.015, -u0], [x + 0.2, v0 + 0.015, -u0], [x + 0.2, v1 + 0.015, -u1], [x, v1 + 0.015, -u1], [1, 1, 1], [0, 1, 0]);
  }
  return b;
}

/** The glasshouse: the cabin, its roof and pillars dark glass. */
export function carGlass(): Builder {
  const b = new Builder();
  prism(b, [
    [0.75, 0.82, 0.76],
    [0.05, 1.24, 0.6],
    [-0.95, 1.28, 0.6],
    [-1.55, 0.86, 0.78],
  ], [1, 1, 1]);
  return b;
}

/** The trim: the splitter, the diffuser, the side skirts, the wing and its struts. */
export function carTrim(): Builder {
  const b = new Builder();
  const dark: RGB = [1, 1, 1];
  b.box([0, 0.25, -2.18], [0.9, 0.06, 0.12], dark);
  b.box([0, 0.3, 2.08], [0.82, 0.1, 0.12], dark);
  for (const s of [-1, 1]) b.box([s * 0.88, 0.32, 0], [0.04, 0.06, 1.3], dark);
  for (const s of [-1, 1]) b.box([s * 0.5, 1.0, 1.88], [0.04, 0.15, 0.08], dark);
  b.box([0, 1.17, 1.92], [0.92, 0.035, 0.2], dark);
  for (const s of [-1, 1]) b.box([s * 0.92, 1.12, 1.92], [0.02, 0.12, 0.24], dark);
  return b;
}

/** The lamps: headlamps at the nose, tail lamps at the tail (two meshes, each its own glow). */
export function carLamps(front: boolean): Builder {
  const b = new Builder();
  for (const s of [-1, 1]) {
    if (front) b.box([s * 0.6, 0.6, -2.17], [0.2, 0.06, 0.04], [1, 1, 1]);
    else b.box([s * 0.62, 0.66, 2.17], [0.18, 0.05, 0.03], [1, 1, 1]);
  }
  return b;
}

/** A wheel about its axle (+x): the tread dark, the rim's spokes lighter, so a turning wheel reads. */
export function wheelMesh(): Builder {
  const b = new Builder();
  const r = 0.36;
  const w = 0.14;
  const sides = 12;
  const tread: RGB = [0.07, 0.07, 0.08];
  const rim: RGB = [0.7, 0.72, 0.75];
  const hub: RGB = [0.22, 0.22, 0.24];
  const pt = (x: number, rad: number, a: number): V3 => [x, rad * math.cos(a), rad * math.sin(a)];
  for (let s = 0; s < sides; s++) {
    const a0 = (s / sides) * Math.PI * 2;
    const a1 = ((s + 1) / sides) * Math.PI * 2;
    const am = (a0 + a1) / 2;
    b.quad(pt(-w, r, a0), pt(w, r, a0), pt(w, r, a1), pt(-w, r, a1), tread, [0, math.cos(am), math.sin(am)]);
    for (const x of [-w, w]) {
      b.quad(pt(x, r, a0), pt(x, r * 0.68, a0), pt(x, r * 0.68, a1), pt(x, r, a1), tread, [x, 0, 0]);
      b.tri(pt(x * 1.05, 0, 0), pt(x * 1.05, r * 0.68, a0), pt(x * 1.05, r * 0.68, a1), s % 3 === 0 ? hub : rim, [x, 0, 0]);
    }
  }
  return b;
}

/** Where the wheels sit in the car frame: front left, front right, rear left, rear right. */
export const WHEELS: readonly V3[] = [
  [-0.84, 0.36, -1.35],
  [0.84, 0.36, -1.35],
  [-0.84, 0.36, 1.32],
  [0.84, 0.36, 1.32],
];
