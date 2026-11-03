// Writes the cliff-top meadow's height field and control map
// (assets/terrain/meadow.png, meadow-control.png):
//
//   bun tools/make-terrain.ts
//
// 160 m square centred on the terrain's entity, in the shot's frame: the
// camera stands near (0, 5) looking down -z. A gently rolling meadow at
// y 900 ends at an edge running from just ahead of the camera on the left
// (so the left of the frame opens onto the sky and the cloud sea) to the
// outcrop on the right where the traveller stands, then falls 300 m in a
// broken cliff into the clouds. The control map: R meadow grass, G the
// flowers along the edge, B the cliff's rock, A the worn
// earth of the outcrop.

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { png } from "./png.ts";

const ROOT = resolve(import.meta.dir, "..", "assets", "terrain");
const N = 321; // samples per side: 0.5 m
const SIZE = 160;
const LOW = 580;
const HIGH = 910;
const TOP = 900;

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
const smooth = (a: number, b: number, x: number) => {
  const t = clamp((x - a) / (b - a), 0, 1);
  return t * t * (3 - 2 * t);
};

// Seeded value noise (deterministic: the same field every run).
function hash(x: number, y: number): number {
  let h = (x * 374761393 + y * 668265263) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
function noise(x: number, y: number): number {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const fx = x - xi;
  const fy = y - yi;
  const u = fx * fx * (3 - 2 * fx);
  const v = fy * fy * (3 - 2 * fy);
  const a = hash(xi, yi);
  const b = hash(xi + 1, yi);
  const c = hash(xi, yi + 1);
  const d = hash(xi + 1, yi + 1);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}
function fbm(x: number, y: number, octaves: number): number {
  let s = 0;
  let amp = 0.5;
  let f = 1;
  for (let i = 0; i < octaves; i++) {
    s += amp * noise(x * f, y * f);
    amp *= 0.5;
    f *= 2;
  }
  return s;
}

/** Where the meadow ends along z at x (the edge, broken by noise): beyond it (smaller z) the cliff. */
export function edgeZ(x: number): number {
  return -1.5 - 0.75 * x + (fbm(x * 0.12 + 7, 3.1, 3) - 0.5) * 3;
}

/** How far a point is past the edge, in metres (negative: on the meadow). */
function past(x: number, z: number): number {
  // Distance across the edge line (slope -0.75), so the cliff falls square to it.
  return (edgeZ(x) - z) / Math.hypot(1, 0.75);
}

const heights = new Float64Array(N * N);
const control = new Float64Array(N * N * 4);
for (let j = 0; j < N; j++) {
  for (let i = 0; i < N; i++) {
    const x = (i / (N - 1) - 0.5) * SIZE;
    const z = (j / (N - 1) - 0.5) * SIZE;
    const d = past(x, z);
    // The meadow: slow rolls of a metre and a half, rising a little behind the camera.
    const meadow = TOP + (fbm(x * 0.03, z * 0.03, 4) - 0.5) * 3 + smooth(10, 60, z) * 6;
    // The lip rounds over in the first 1.5 m, then the face falls in broken steps.
    const lip = smooth(-0.5, 1.5, d) * 1.2;
    const steps = Math.floor(d / 9) * 9 + smooth(0, 9, d % 9) * 9;
    const face = d <= 0 ? 0 : Math.min(320, steps * 6 + (fbm(x * 0.15, d * 0.1, 4) - 0.5) * 40 * smooth(2, 20, d));
    const h = meadow - lip - face;
    heights[j * N + i] = clamp(h, LOW, HIGH);
    // Control: flowers in a band along the edge and in drifts; earth on the outcrop.
    const k = (j * N + i) * 4;
    const flowers = clamp(smooth(-7, -1.5, d) * smooth(0.35, 0.6, fbm(x * 0.2 + 3, z * 0.2, 3)) + smooth(0.62, 0.72, fbm(x * 0.08, z * 0.08 + 9, 3)), 0, 1);
    const earth = smooth(3.5, 1.5, Math.hypot(x - 4, z + 4)) * smooth(0.3, 0.5, fbm(x * 0.6, z * 0.6, 3)) * 0.8;
    // Rock past the lip (its slope rule keeps it to the steep faces); the meadow's layers before it.
    const rock = smooth(0.5, 2.5, d);
    const grass = Math.max(0, 1 - flowers - earth) * (1 - rock);
    control[k] = grass * 255;
    control[k + 1] = flowers * (1 - rock) * 255;
    control[k + 2] = rock * 255;
    control[k + 3] = earth * (1 - rock) * 255;
  }
}

const field = new Float64Array(N * N);
for (let p = 0; p < N * N; p++) field[p] = ((heights[p]! - LOW) / (HIGH - LOW)) * 65535;

function write(rel: string, bytes: Uint8Array): void {
  const out = join(ROOT, rel);
  mkdirSync(dirname(out), { recursive: true });
  writeFileSync(out, bytes);
  console.log(`wrote ${rel} (${bytes.length} bytes)`);
}
write("meadow.png", png(N, N, field, 1));
write("meadow-control.png", png(N, N, control, 4));
