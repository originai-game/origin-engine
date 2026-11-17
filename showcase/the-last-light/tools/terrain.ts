// Generates the coast's height field and control map:
//
//   bun tools/terrain.ts
//
// writes assets/terrain/coast-height.png (16-bit grey) and
// assets/terrain/coast-control.png (RGBA: grass, cliff rock, path, shore),
// which assets/terrain/coast.ogt names. Needs ImageMagick (`magick`).
// The layout constants are shared with the game through src/layout.ts, so
// the camera, the path and the buildings stand where the ground was made
// for them.
import { spawnSync } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { HEADLAND, HEIGHT_RANGE, PATH, SAMPLES, SIZE, coastX } from "../src/layout.ts";

const out = join(import.meta.dir, "..", "assets", "terrain");
mkdirSync(out, { recursive: true });

// ── noise: value noise on an integer lattice, hashed with 32-bit integer
// arithmetic so every machine makes the same field ──────────────────────────
function hash(x: number, y: number, seed: number): number {
  let h = (Math.imul(x, 0x27d4eb2d) ^ Math.imul(y, 0x165667b1) ^ Math.imul(seed, 0x9e3779b1)) >>> 0;
  h = Math.imul(h ^ (h >>> 15), 0x85ebca6b) >>> 0;
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35) >>> 0;
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
function noise(x: number, y: number, seed: number): number {
  const xi = Math.floor(x), yi = Math.floor(y);
  const fx = x - xi, fy = y - yi;
  const u = fx * fx * (3 - 2 * fx), v = fy * fy * (3 - 2 * fy);
  const a = hash(xi, yi, seed), b = hash(xi + 1, yi, seed), c = hash(xi, yi + 1, seed), d = hash(xi + 1, yi + 1, seed);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}
/** Fractal noise in -1..1. */
function fbm(x: number, y: number, octaves: number, seed: number): number {
  let sum = 0, amp = 0.5, norm = 0;
  for (let i = 0; i < octaves; i++) {
    sum += (noise(x, y, seed + i * 17) * 2 - 1) * amp;
    norm += amp;
    x *= 2.03; y *= 2.03; amp *= 0.5;
  }
  return sum / norm;
}
/** Ridged noise in 0..1: sharp crests, for broken rock. */
function ridged(x: number, y: number, octaves: number, seed: number): number {
  let sum = 0, amp = 0.5, norm = 0;
  for (let i = 0; i < octaves; i++) {
    const n = 1 - Math.abs(noise(x, y, seed + i * 31) * 2 - 1);
    sum += n * n * amp;
    norm += amp;
    x *= 2.1; y *= 2.1; amp *= 0.5;
  }
  return sum / norm;
}
/** Cellular noise: the distance to the nearest of one jittered point a cell, 0..~1. */
function cellular(x: number, y: number, seed: number): number {
  const xi = Math.floor(x), yi = Math.floor(y);
  let best = 9;
  for (let j = -1; j <= 1; j++) {
    for (let i = -1; i <= 1; i++) {
      const px = xi + i + hash(xi + i, yi + j, seed), py = yi + j + hash(xi + i, yi + j, seed + 7);
      best = Math.min(best, Math.hypot(px - x, py - y));
    }
  }
  return best;
}
const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
const smooth = (a: number, b: number, v: number) => { const t = clamp((v - a) / (b - a), 0, 1); return t * t * (3 - 2 * t); };
const mix = (a: number, b: number, t: number) => a + (b - a) * t;

// ── the coast: metres inland of the waterline (negative: out at sea) ───────
function inland(x: number, z: number): number {
  // The mainland lies left of its coast line; the wobble breaks it into
  // coves and points.
  const wobble = fbm(z * 0.035, 3.7, 3, 11) * 7 + fbm(z * 0.11, 9.1, 2, 12) * 2.5;
  const main = coastX(z) + wobble - x;
  // The headland: an ellipse turned a little, its edge broken the same way.
  const c = Math.cos(HEADLAND.turn), s = Math.sin(HEADLAND.turn);
  const dx = x - HEADLAND.x, dz = z - HEADLAND.z;
  const lx = (dx * c + dz * s) / HEADLAND.rx, lz = (-dx * s + dz * c) / HEADLAND.rz;
  const r = Math.hypot(lx, lz);
  const edge = 1 + fbm(Math.atan2(lz, lx) * 1.6, 5.3, 3, 13) * 0.22;
  const head = (edge - r) * Math.min(HEADLAND.rx, HEADLAND.rz);
  // A smooth union keeps the neck between them whole.
  const k = 10;
  const h = clamp(0.5 + (main - head) / (2 * k), 0, 1);
  // Buttresses and gullies: the rock stands in blocks a dozen metres across.
  const blocks = (cellular(x * 0.085, z * 0.085, 14) - 0.45) * 7 + (cellular(x * 0.23, z * 0.23, 15) - 0.45) * 2.2;
  return mix(head, main, h) + k * h * (1 - h) - blocks;
}

/** How close a point is to the path's centre line, in metres. */
function pathDistance(x: number, z: number): number {
  let best = 1e9;
  for (let i = 0; i + 1 < PATH.length; i++) {
    const [ax, az] = PATH[i]!, [bx, bz] = PATH[i + 1]!;
    const ex = bx - ax, ez = bz - az;
    const t = clamp(((x - ax) * ex + (z - az) * ez) / (ex * ex + ez * ez), 0, 1);
    best = Math.min(best, Math.hypot(x - (ax + ex * t), z - (az + ez * t)));
  }
  return best + fbm(x * 0.3, z * 0.3, 2, 41) * 0.35;
}

/** The plateau's height where the land is whole, before the cliffs cut it. */
function plateau(x: number, z: number): number {
  // The cliff path's shelf: falling gently towards the sea on the right,
  // rising to the hill on the left.
  let h = 19.5 - 0.2 * clamp(x, -30, 14) + 0.05 * Math.max(0, -x - 30);
  // The saddle the path crosses before the headland.
  h -= 5.5 * Math.exp(-(((z + 42) / 16) ** 2)) * smooth(-45, -5, x);
  // The headland's crown: a level yard under the lighthouse and cottage,
  // stepping down to the point at its seaward end.
  const c = Math.cos(HEADLAND.turn), s = Math.sin(HEADLAND.turn);
  const along = ((x - HEADLAND.x) * c + (z - HEADLAND.z) * s) / HEADLAND.rx;
  const across = (-(x - HEADLAND.x) * s + (z - HEADLAND.z) * c) / HEADLAND.rz;
  const hd = Math.hypot(along, across);
  // Three beds, each a step down towards the point.
  const crown = 25.5 - 4.5 * smooth(0.4, 0.52, along) - 6 * smooth(0.62, 0.72, along) - 6 * smooth(0.82, 0.92, along) - 1.2 * across * across;
  h = mix(h, crown, smooth(1.35, 0.75, hd));
  // The hills inland, far to the left and behind.
  h += 26 * smooth(-60, -190, x) * (0.6 + 0.4 * fbm(x * 0.012, z * 0.012, 3, 21));
  // Rolling ground and small hummocks.
  h += fbm(x * 0.02, z * 0.02, 4, 22) * 2.4 + fbm(x * 0.09, z * 0.09, 3, 23) * 0.5 + fbm(x * 0.5, z * 0.5, 2, 24) * 0.07;
  return h;
}

function height(x: number, z: number): { h: number; d: number } {
  const d = inland(x, z);
  if (d < 0) {
    // The seabed: a shelf of broken rock off the cliffs' feet, then falling
    // away into deep water, so the swell comes in whole and shoals and
    // breaks only before the rocks.
    const reef = ridged(x * 0.09, z * 0.09, 3, 31) * 2.2 * smooth(-22, -2, d);
    return { h: Math.max(-70, -1.2 + d * 0.22 - 0.011 * d * d) + reef - 0.6, d };
  }
  const top = plateau(x, z);
  // The cliff: a skirt of wave-cut rock, then a face broken into ledges.
  const skirt = 0.4 + d * 0.5 + ridged(x * 0.2, z * 0.2, 3, 32) * 1.8;
  // The headland's face towards the path falls in broad ledges where turf
  // holds; the rest stands steep.
  const broad = smooth(-58, -66, z) * smooth(-14, 0, x) * smooth(70, 40, x);
  const faceWidth = 3.2 + fbm(x * 0.05, z * 0.05, 2, 33) * 1.6 + top * 0.1 + broad * (7 + top * 0.22);
  const t = smooth(2.5, 2.5 + faceWidth, d);
  const ledges = (ridged(x * 0.13, z * 0.13, 4, 34) - 0.45) * 4.5 * Math.sin(Math.PI * t);
  const lip = smooth(2.5 + faceWidth, 2.5 + faceWidth + 6, d);
  let h = mix(skirt, top - 1.2 * (1 - lip), t ** 0.8) + ledges;
  // Strata: the face stands in beds a few metres thick, their tops ledges
  // where turf holds, their fronts near sheer.
  const bed = 3.4, lift = fbm(x * 0.03, z * 0.03, 2, 35) * 3 + 0.3 * (x + z * 0.4) * 0.1;
  const q = (h + lift) / bed, f = q - Math.floor(q);
  const stepped = (Math.floor(q) + smooth(0.62, 0.98, f)) * bed - lift;
  h = mix(h, stepped, 0.85 * Math.sin(Math.PI * clamp(t, 0, 1)) ** 0.5) + fbm(x * 0.7, z * 0.7, 3, 36) * 0.35 * Math.sin(Math.PI * t);
  return { h, d };
}

const heights = new Float32Array(SAMPLES * SAMPLES);
const inlands = new Float32Array(SAMPLES * SAMPLES);
const at = (col: number, row: number) => [(col / (SAMPLES - 1) - 0.5) * SIZE, (row / (SAMPLES - 1) - 0.5) * SIZE] as const;
for (let row = 0; row < SAMPLES; row++) {
  for (let col = 0; col < SAMPLES; col++) {
    const [x, z] = at(col, row);
    const { h, d } = height(x, z);
    // The path is trodden a hand's breadth into the turf.
    heights[row * SAMPLES + col] = h - (d > 6 ? 0.09 * smooth(1.3, 0.25, pathDistance(x, z)) : 0);
    inlands[row * SAMPLES + col] = d;
  }
}
// The layers follow the ground as made: rock wherever it is too steep for
// turf to hold, so every ledge of the cliffs keeps its grass.
const control = Buffer.alloc(SAMPLES * SAMPLES * 4);
const cell = SIZE / (SAMPLES - 1);
const sample = (col: number, row: number) => heights[clamp(row, 0, SAMPLES - 1) * SAMPLES + clamp(col, 0, SAMPLES - 1)]!;
for (let row = 0; row < SAMPLES; row++) {
  for (let col = 0; col < SAMPLES; col++) {
    const [x, z] = at(col, row);
    const h = sample(col, row), d = inlands[row * SAMPLES + col]!;
    const slope = Math.hypot(sample(col + 1, row) - sample(col - 1, row), sample(col, row + 1) - sample(col, row - 1)) / (2 * cell);
    const bare = fbm(x * 0.35, z * 0.35, 3, 51) * 0.18;
    const rock = d < 0 ? 1 : Math.max(smooth(0.6, 1.05, slope + bare), smooth(4.5, 2.5, h));
    const path = d > 6 ? smooth(0.95, 0.4, pathDistance(x, z)) * (1 - rock) : 0;
    const shore = smooth(2.4, 0.5, h + bare * 3);
    const i = (row * SAMPLES + col) * 4;
    control[i] = Math.round(255 * (1 - rock) * (1 - path) * (1 - shore));
    control[i + 1] = Math.round(255 * rock * (1 - shore));
    control[i + 2] = Math.round(255 * path * (1 - shore));
    control[i + 3] = Math.round(255 * shore);
  }
}

const [low, high] = HEIGHT_RANGE;
let min = 1e9, max = -1e9;
const pgm = Buffer.alloc(SAMPLES * SAMPLES * 2);
for (let i = 0; i < heights.length; i++) {
  min = Math.min(min, heights[i]!); max = Math.max(max, heights[i]!);
  pgm.writeUInt16BE(Math.round(clamp((heights[i]! - low) / (high - low), 0, 1) * 65535), i * 2);
}
if (min < low || max > high) throw new Error(`heights ${min.toFixed(1)}..${max.toFixed(1)} leave HEIGHT_RANGE ${low}..${high}`);

function magick(args: string[], input: Buffer): void {
  const r = spawnSync("magick", args, { input, maxBuffer: 1 << 28 });
  if (r.status !== 0) throw new Error(`magick ${args.join(" ")}: ${r.stderr}`);
}
const scratch = join(out, ".scratch");
mkdirSync(scratch, { recursive: true });
writeFileSync(join(scratch, "h.pgm"), Buffer.concat([Buffer.from(`P5\n${SAMPLES} ${SAMPLES}\n65535\n`), pgm]));
magick([join(scratch, "h.pgm"), "-depth", "16", "-define", "png:color-type=0", "-define", "png:bit-depth=16", join(out, "coast-height.png")], Buffer.alloc(0));
magick(["-size", `${SAMPLES}x${SAMPLES}`, "-depth", "8", "rgba:-", "-define", "png:color-type=6", join(out, "coast-control.png")], control);
rmSync(scratch, { recursive: true });
console.log(`coast: ${SAMPLES}x${SAMPLES} samples over ${SIZE} m, heights ${min.toFixed(2)}..${max.toFixed(2)} m`);
