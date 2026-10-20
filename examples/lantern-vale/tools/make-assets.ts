// Lantern Vale's asset generator: every source asset of the game is made
// here, deterministically (a seeded generator, no clock), with no download.
//
//   bun tools/make-assets.ts            everything, the 4-minute music included
//   bun tools/make-assets.ts --no-music everything but assets/music/dusk.wav
//
// What it writes under assets/:
// - models/*.vox: MagicaVoxel files (hero, lantern, tree, rock, and the far
//   side's pine and ruined arch), each with its import settings
//   (`model { voxScale voxOrigin }`).
// - models/slime.glb: the slime's voxels meshed here into a skinned glTF
//   (two joints, clips `idle` and `chase`), because a .vox cooks into a
//   static model and og.AnimationPlayer needs a skeleton (FINDINGS.md).
// - textures/*.png: tileable grass, path, rock and moss for the terrain, a
//   spark for the lantern's burst.
// - sprites/faces/*.png: the slimes' expression bubbles (32x32 pixel art),
//   the layers of the texture array sprites/faces.ogt.
// - terrain/vale.png and vale-control.png: the valley's 16-bit height field
//   and its splat control map.
// - env/dusk.hdr: a 64x32 synthetic dusk sky (Radiance RGBE, run-length
//   encoded), prefiltered by og cook through env/dusk.ogt.
// - music/dusk.wav: a 4-minute ambient piece (48 kHz mono, 16-bit), cooked
//   in chunks (music/dusk.wav.ogt: `sound { stream true }`). About 23 MB, so
//   it is not committed (.gitignore): run this script once after a clone.
//
// Keep the numbers in step with src/world.ts (LANTERNS, START, the terrain's
// size and height range).

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { deflateSync } from "node:zlib";

const ROOT = resolve(import.meta.dir, "..");
const ASSETS = join(ROOT, "assets");

function write(rel: string, data: Uint8Array | string): void {
  const p = join(ASSETS, rel);
  mkdirSync(dirname(p), { recursive: true });
  writeFileSync(p, data);
}

// ---------------------------------------------------------------- randomness

/** mulberry32: a small seeded generator, the same numbers on every machine. */
function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function hash2(x: number, y: number, s: number): number {
  let h = Math.imul(x | 0, 374761393) ^ Math.imul(y | 0, 668265263) ^ Math.imul(s | 0, 2147483647);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

const smooth = (t: number) => t * t * (3 - 2 * t);
const clamp = (v: number, a: number, b: number) => Math.max(a, Math.min(b, v));
const mix = (a: number, b: number, t: number) => a + (b - a) * t;
function smoothstep(a: number, b: number, v: number): number {
  return smooth(clamp((v - a) / (b - a), 0, 1));
}

/** Value noise, wrapping every `period` cells (0 for none). */
function vnoise(x: number, y: number, s: number, period = 0): number {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const fx = smooth(x - xi);
  const fy = smooth(y - yi);
  const w = (v: number) => (period > 0 ? ((v % period) + period) % period : v);
  const a = hash2(w(xi), w(yi), s);
  const b = hash2(w(xi + 1), w(yi), s);
  const c = hash2(w(xi), w(yi + 1), s);
  const d = hash2(w(xi + 1), w(yi + 1), s);
  return mix(mix(a, b, fx), mix(c, d, fx), fy);
}

function fbm(x: number, y: number, s: number, octaves = 4, period = 0): number {
  let sum = 0;
  let amp = 0.5;
  let norm = 0;
  for (let o = 0; o < octaves; o++) {
    const f = 1 << o;
    sum += amp * vnoise(x * f, y * f, s + o * 17, period > 0 ? period * f : 0);
    norm += amp;
    amp *= 0.5;
  }
  return sum / norm;
}

// ---------------------------------------------------------------------- PNG

const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function crc32(bytes: Uint8Array): number {
  let c = 0xffffffff;
  for (const b of bytes) c = CRC[(c ^ b) & 255]! ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type: string, data: Uint8Array): Uint8Array {
  const out = new Uint8Array(12 + data.length);
  const dv = new DataView(out.buffer);
  dv.setUint32(0, data.length);
  for (let i = 0; i < 4; i++) out[4 + i] = type.charCodeAt(i);
  out.set(data, 8);
  dv.setUint32(8 + data.length, crc32(out.subarray(4, 8 + data.length)));
  return out;
}

/** A PNG: RGBA8 (`channels` 4) or 16-bit grey (`channels` 1, values 0..65535). */
function png(w: number, h: number, px: ArrayLike<number>, channels: 4 | 1): Uint8Array {
  const bpp = channels === 4 ? 4 : 2;
  const raw = new Uint8Array(h * (w * bpp + 1));
  for (let y = 0; y < h; y++) {
    const row = y * (w * bpp + 1);
    raw[row] = 0;
    for (let x = 0; x < w; x++) {
      if (channels === 4) {
        for (let k = 0; k < 4; k++) raw[row + 1 + x * 4 + k] = clamp(Math.round(px[(y * w + x) * 4 + k]!), 0, 255);
      } else {
        const v = clamp(Math.round(px[y * w + x]!), 0, 65535);
        raw[row + 1 + x * 2] = v >> 8;
        raw[row + 2 + x * 2] = v & 255;
      }
    }
  }
  const ihdr = new Uint8Array(13);
  const dv = new DataView(ihdr.buffer);
  dv.setUint32(0, w);
  dv.setUint32(4, h);
  ihdr[8] = channels === 4 ? 8 : 16;
  ihdr[9] = channels === 4 ? 6 : 0;
  const sig = Uint8Array.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const parts = [sig, chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", new Uint8Array())];
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

function hex(h: string): [number, number, number] {
  const s = h.replace("#", "");
  return [parseInt(s.slice(0, 2), 16), parseInt(s.slice(2, 4), 16), parseInt(s.slice(4, 6), 16)];
}

// ---------------------------------------------------------------- textures

/** A tileable texture: `colour(u, v)` with u, v in cells of a `period`-cell wrap. */
function tileable(size: number, period: number, colour: (u: number, v: number) => [number, number, number]): Uint8Array {
  const px = new Float32Array(size * size * 4);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const [r, g, b] = colour((x / size) * period, (y / size) * period);
      const i = (y * size + x) * 4;
      px[i] = r;
      px[i + 1] = g;
      px[i + 2] = b;
      px[i + 3] = 255;
    }
  }
  return png(size, size, px, 4);
}

function shade(a: string, b: string, t: number): [number, number, number] {
  const x = hex(a);
  const y = hex(b);
  return [mix(x[0], y[0], t), mix(x[1], y[1], t), mix(x[2], y[2], t)];
}

function textures(): void {
  // Meadow grass at dusk: blue-green, blades as fine bright specks.
  write("textures/grass.png", tileable(128, 8, (u, v) => {
    const n = fbm(u, v, 11, 4, 8);
    const blade = vnoise(u * 8, v * 8, 12, 64);
    const c = shade("#2f5a46", "#4f8a5c", n);
    const k = blade > 0.78 ? 1.18 : blade < 0.2 ? 0.86 : 1;
    return [c[0] * k, c[1] * k, c[2] * k];
  }));
  // The beaten path: warm dirt with pebbles.
  write("textures/path.png", tileable(128, 8, (u, v) => {
    const n = fbm(u, v, 21, 4, 8);
    const pebble = vnoise(u * 6, v * 6, 22, 48);
    const c = shade("#6e5640", "#9c7b56", n);
    return pebble > 0.8 ? shade("#b8a58c", "#cbbba3", pebble) : c;
  }));
  // Cliff rock: grey-violet strata.
  write("textures/rock.png", tileable(128, 8, (u, v) => {
    const n = fbm(u * 0.5, v * 2, 31, 5, 4);
    return shade("#4a4652", "#7d7686", n);
  }));
  // The far side's moss: deep teal with pale tips.
  write("textures/moss.png", tileable(128, 8, (u, v) => {
    const n = fbm(u, v, 41, 4, 8);
    const tip = vnoise(u * 10, v * 10, 42, 80);
    const c = shade("#1f4a4a", "#3b7a66", n);
    return tip > 0.9 ? shade("#4f8a72", "#5f9a80", tip) : c;
  }));
  // A soft round spark for the lantern's burst (white, the emitter tints it).
  const s = 32;
  const spark = new Float32Array(s * s * 4);
  for (let y = 0; y < s; y++) {
    for (let x = 0; x < s; x++) {
      const d = Math.hypot(x + 0.5 - s / 2, y + 0.5 - s / 2) / (s / 2);
      const a = Math.pow(clamp(1 - d, 0, 1), 1.6);
      const i = (y * s + x) * 4;
      spark[i] = 255;
      spark[i + 1] = 255;
      spark[i + 2] = 255;
      spark[i + 3] = a * 255;
    }
  }
  write("textures/spark.png", png(s, s, spark, 4));
  // The far side's banner: indigo cloth, a gold lantern sign, a fringe (the far-side group's texture).
  const bw = 64, bh = 96;
  const banner = new Float32Array(bw * bh * 4);
  const gold = hex("#e0aa48");
  for (let y = 0; y < bh; y++) {
    for (let x = 0; x < bw; x++) {
      const i = (y * bw + x) * 4;
      const weave = 0.9 + 0.1 * hash2(x, y, 77);
      let c: [number, number, number] = [44 * weave, 36 * weave, 96 * weave];
      const cx = x - bw / 2;
      // The sign: a lantern's cage (a rounded box), its cap and its glow.
      const inCage = Math.abs(cx) < 11 && y > 34 && y < 64;
      const inCap = y >= 28 && y <= 34 && Math.abs(cx) < 15 - (34 - y);
      const ring = Math.abs(Math.hypot(cx, y - 49) - 20) < 1.6;
      if (inCage) c = Math.abs(cx) > 8 || y < 37 || y > 61 ? gold : [255 * 0.98, 205, 120];
      if (inCap || ring) c = gold;
      if (x < 3 || x >= bw - 3) c = [c[0] * 0.7, c[1] * 0.7, c[2] * 0.7];
      // A fringe of tassels along the bottom.
      const a = y > bh - 8 && (x % 6 > 2 || y > bh - 4) ? 0 : 255;
      if (y > bh - 10 && a > 0) c = gold;
      banner[i] = c[0];
      banner[i + 1] = c[1];
      banner[i + 2] = c[2];
      banner[i + 3] = a;
    }
  }
  write("textures/banner.png", png(bw, bh, banner, 4));
}

// ------------------------------------------------------- expression bubbles

/** 5x7 glyphs for the bubbles, '#' inked. */
const GLYPHS: Record<string, string[]> = {
  calm: ["     ", "     ", "     ", "#.#.#", "     ", "     ", "     "],
  alert: ["  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "     ", "  #  "],
  lost: [" ### ", "#   #", "    #", "   # ", "  #  ", "     ", "  #  "],
  sleepy: ["#### ", "   # ", "  #  ", " #   ", "#### ", "     ", "     "],
};
const FACES = ["calm", "alert", "lost", "sleepy"] as const;
const INK: Record<string, string> = { calm: "#2a2440", alert: "#c0392b", lost: "#2a2440", sleepy: "#3b4a8c" };

function faces(): void {
  const S = 32;
  for (const name of FACES) {
    const px = new Float32Array(S * S * 4);
    const put = (x: number, y: number, c: [number, number, number], a = 255) => {
      if (x < 0 || y < 0 || x >= S || y >= S) return;
      const i = (y * S + x) * 4;
      px[i] = c[0];
      px[i + 1] = c[1];
      px[i + 2] = c[2];
      px[i + 3] = a;
    };
    const cream = hex("#fff3d6");
    const edge = hex("#2a2440");
    // A rounded bubble 26x18 from (3, 3), its tail down to (16, 27).
    const inBubble = (x: number, y: number) => {
      if (x < 3 || x > 28 || y < 3 || y > 21) return false;
      const cx = Math.min(Math.max(x, 6), 25);
      const cy = Math.min(Math.max(y, 6), 18);
      return Math.hypot(x - cx, y - cy) <= 3.2;
    };
    const inTail = (x: number, y: number) => y >= 21 && y <= 26 && Math.abs(x - 15.5) <= (26 - y) * 0.6;
    const inside = (x: number, y: number) => inBubble(x, y) || inTail(x, y);
    for (let y = 0; y < S; y++) {
      for (let x = 0; x < S; x++) {
        if (inside(x, y)) {
          const border = !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1);
          put(x, y, border ? edge : cream);
        }
      }
    }
    const g = GLYPHS[name]!;
    const ink = hex(INK[name]!);
    // Each glyph cell is 2x2 pixels, the glyph centred in the bubble.
    for (let gy = 0; gy < 7; gy++) {
      for (let gx = 0; gx < 5; gx++) {
        if (g[gy]![gx] !== "#") continue;
        for (let k = 0; k < 4; k++) put(11 + gx * 2 + (k & 1), 5 + gy * 2 + (k >> 1), ink);
      }
    }
    write(`sprites/faces/${name}.png`, png(S, S, px, 4));
  }
}

// ---------------------------------------------------------------- terrain

/** The terrain: 96 m square centred on the origin, heights -2 to 14 m. */
const TERRAIN = { size: 96, samples: 129, low: -2, high: 14 };
/** The lanterns (x, z): keep in step with LANTERNS in src/world.ts. */
const LANTERNS = [
  { x: -22, z: 8 },
  { x: 20, z: 12 },
  { x: -16, z: -14 },
  { x: 14, z: -30 },
];
const START = { x: 0, z: 26 };
const PATH: [number, number][][] = [
  [[START.x, START.z], [0, 16], [-10, 10], [-22, 8]],
  [[0, 16], [10, 12], [20, 12]],
  [[0, 16], [-2, 2], [-8, -8], [-16, -14]],
  [[-2, 2], [4, -12], [8, -22], [14, -30]],
];

function segDist(px: number, pz: number, a: [number, number], b: [number, number]): number {
  const dx = b[0] - a[0];
  const dz = b[1] - a[1];
  const t = clamp(((px - a[0]) * dx + (pz - a[1]) * dz) / (dx * dx + dz * dz), 0, 1);
  return Math.hypot(px - (a[0] + dx * t), pz - (a[1] + dz * t));
}

function pathDist(x: number, z: number): number {
  let d = Infinity;
  for (const p of PATH) for (let i = 0; i + 1 < p.length; i++) d = Math.min(d, segDist(x, z, p[i]!, p[i + 1]!));
  return d;
}

/** The ground's height at (x, z), metres. */
export function height(x: number, z: number): number {
  // The valley floor rises into a ring of ridges past 34 m from the centre.
  const edge = Math.max(Math.abs(x) * 1.05, Math.abs(z)) + fbm(x * 0.05, z * 0.05, 5) * 6;
  const ridge = smoothstep(33, 46, edge) * 12;
  // A gentle roll of the meadow, flattened along the paths and round the lanterns.
  let roll = (fbm(x * 0.06, z * 0.06, 7) - 0.5) * 1.4;
  const flat = Math.min(1, pathDist(x, z) / 4);
  roll *= flat;
  for (const l of LANTERNS) roll *= Math.min(1, Math.hypot(x - l.x, z - l.z) / 4);
  // The far side (north, -z) stands a step higher.
  const step = smoothstep(-14, -22, z) * 1.2;
  return ridge + roll + step;
}

function terrain(): void {
  const n = TERRAIN.samples;
  const hm = new Float32Array(n * n);
  const ctrl = new Float32Array(n * n * 4);
  for (let j = 0; j < n; j++) {
    for (let i = 0; i < n; i++) {
      const x = (i / (n - 1) - 0.5) * TERRAIN.size;
      const z = (j / (n - 1) - 0.5) * TERRAIN.size;
      const h = height(x, z);
      hm[j * n + i] = ((h - TERRAIN.low) / (TERRAIN.high - TERRAIN.low)) * 65535;
      const path = 1 - smoothstep(0.9, 1.9, pathDist(x, z));
      const moss = smoothstep(-15, -20, z) * (1 - path);
      const k = (j * n + i) * 4;
      ctrl[k] = (1 - path) * (1 - moss) * 255; // R: grass
      ctrl[k + 1] = path * 255; // G: path
      ctrl[k + 2] = 255; // B: rock, by slope alone
      ctrl[k + 3] = moss * 255; // A: moss on the far side
    }
  }
  write("terrain/vale.png", png(n, n, hm, 1));
  write("terrain/vale-control.png", png(n, n, ctrl, 4));
}

// -------------------------------------------------------------------- HDR

function rgbe(r: number, g: number, b: number): [number, number, number, number] {
  const v = Math.max(r, g, b);
  if (v < 1e-32) return [0, 0, 0, 0];
  const e = Math.floor(Math.log2(v)) + 1;
  const s = 256 / 2 ** e;
  return [Math.min(255, Math.floor(r * s)), Math.min(255, Math.floor(g * s)), Math.min(255, Math.floor(b * s)), e + 128];
}

/** A Radiance .hdr, each scanline run-length encoded (as literal runs). */
function hdr(w: number, h: number, px: (u: number, v: number) => [number, number, number]): Uint8Array {
  const head = new TextEncoder().encode(`#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y ${h} +X ${w}\n`);
  const body: number[] = [];
  for (let y = 0; y < h; y++) {
    const row: [number, number, number, number][] = [];
    for (let x = 0; x < w; x++) row.push(rgbe(...px((x + 0.5) / w, (y + 0.5) / h)));
    body.push(2, 2, w >> 8, w & 255);
    for (let c = 0; c < 4; c++) {
      for (let at = 0; at < w; at += 128) {
        const n = Math.min(128, w - at);
        body.push(n);
        for (let k = 0; k < n; k++) body.push(row[at + k]![c]);
      }
    }
  }
  const out = new Uint8Array(head.length + body.length);
  out.set(head);
  out.set(body, head.length);
  return out;
}

function environment(): void {
  // Equirectangular: u 0.5 faces +X, v 0 is the zenith. The sun has just set
  // at +X: a hot band on the horizon there, indigo overhead, the valley's
  // dark floor below.
  write("env/dusk.hdr", hdr(64, 32, (u, v) => {
    const el = (0.5 - v) * Math.PI;
    const az = (u - 0.5) * 2 * Math.PI;
    if (el < 0) {
      const t = smoothstep(0, -0.5, el);
      return [mix(0.09, 0.02, t), mix(0.07, 0.025, t), mix(0.06, 0.03, t)];
    }
    const up = Math.pow(el / (Math.PI / 2), 0.55);
    const zen: [number, number, number] = [0.05, 0.06, 0.2];
    const hor: [number, number, number] = [0.9, 0.42, 0.22];
    const c: [number, number, number] = [mix(hor[0], zen[0], up), mix(hor[1], zen[1], up), mix(hor[2], zen[2], up)];
    // The afterglow round the sun's azimuth, low on the horizon.
    const toward = Math.max(0, Math.cos(az));
    const glow = Math.pow(toward, 6) * Math.exp(-el * 9) * 4;
    const sun = toward > 0.995 && el < 0.12 ? 18 : 0;
    return [c[0] + glow * 1.6 + sun, c[1] + glow * 0.7 + sun * 0.7, c[2] + glow * 0.3 + sun * 0.4];
  }));
}

// -------------------------------------------------------------------- VOX

type Vox = { size: [number, number, number]; voxels: [number, number, number, number][]; palette: string[] };

/** A voxel model: z up, y away from the viewer (MagicaVoxel's axes); colours by palette name. */
class Grid {
  readonly cells = new Map<string, number>();
  readonly palette: string[] = [];
  constructor(readonly sx: number, readonly sy: number, readonly sz: number) {}
  colour(c: string): number {
    let i = this.palette.indexOf(c);
    if (i < 0) {
      this.palette.push(c);
      i = this.palette.length - 1;
    }
    return i + 1;
  }
  set(x: number, y: number, z: number, c: string): void {
    if (x < 0 || y < 0 || z < 0 || x >= this.sx || y >= this.sy || z >= this.sz) return;
    this.cells.set(`${x},${y},${z}`, this.colour(c));
  }
  clear(x: number, y: number, z: number): void {
    this.cells.delete(`${x},${y},${z}`);
  }
  has(x: number, y: number, z: number): boolean {
    return this.cells.has(`${x},${y},${z}`);
  }
  box(x0: number, y0: number, z0: number, x1: number, y1: number, z1: number, c: string): void {
    for (let z = z0; z <= z1; z++) for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) this.set(x, y, z, c);
  }
  vox(): Vox {
    const voxels: [number, number, number, number][] = [];
    for (const [k, c] of [...this.cells.entries()].sort()) {
      const [x, y, z] = k.split(",").map(Number) as [number, number, number];
      voxels.push([x, y, z, c]);
    }
    return { size: [this.sx, this.sy, this.sz], voxels, palette: this.palette };
  }
}

function voxBytes(v: Vox): Uint8Array {
  const out: number[] = [];
  const u32 = (a: number[], n: number) => a.push(n & 255, (n >>> 8) & 255, (n >>> 16) & 255, (n >>> 24) & 255);
  const chunkOf = (id: string, body: number[]) => {
    const c: number[] = [];
    for (const ch of id) c.push(ch.charCodeAt(0));
    u32(c, body.length);
    u32(c, 0);
    return c.concat(body);
  };
  const size: number[] = [];
  for (const s of v.size) u32(size, s);
  const xyzi: number[] = [];
  u32(xyzi, v.voxels.length);
  for (const p of v.voxels) xyzi.push(...p);
  const rgba: number[] = [];
  for (let i = 0; i < 256; i++) {
    const c = v.palette[i];
    if (c) rgba.push(...hex(c), 255);
    else rgba.push(0, 0, 0, 0);
  }
  // A scene graph (root transform 0, group 1, transform 2 at the origin, shape 3 of model 0): without
  // one, og cook places a lone model with its corner, not its centre, at the origin (FINDINGS.md).
  const str = (a: number[], t: string) => {
    u32(a, t.length);
    for (const ch of t) a.push(ch.charCodeAt(0));
  };
  const trn = (node: number, child: number, frame: [string, string][]) => {
    const b: number[] = [];
    u32(b, node);
    u32(b, 0);
    u32(b, child);
    u32(b, 0xffffffff);
    u32(b, 0);
    u32(b, 1);
    u32(b, frame.length);
    for (const [k, v] of frame) {
      str(b, k);
      str(b, v);
    }
    return b;
  };
  const grp: number[] = [];
  u32(grp, 1);
  u32(grp, 0);
  u32(grp, 1);
  u32(grp, 2);
  const shp: number[] = [];
  for (const n of [3, 0, 1, 0, 0]) u32(shp, n);
  const children = [
    ...chunkOf("SIZE", size), ...chunkOf("XYZI", xyzi), ...chunkOf("RGBA", rgba),
    ...chunkOf("nTRN", trn(0, 1, [])), ...chunkOf("nGRP", grp), ...chunkOf("nTRN", trn(2, 3, [["_t", "0 0 0"]])), ...chunkOf("nSHP", shp),
  ];
  for (const ch of "VOX ") out.push(ch.charCodeAt(0));
  u32(out, 150);
  for (const ch of "MAIN") out.push(ch.charCodeAt(0));
  u32(out, 0);
  u32(out, children.length);
  return Uint8Array.from(out.concat(children));
}

function writeVox(name: string, g: Grid, scale: number, why: string): void {
  write(`models/${name}.vox`, voxBytes(g.vox()));
  write(`models/${name}.vox.ogt`, `type import\nversion 1\n---\n# ${why} (tools/make-assets.ts).\nmodel {\n  voxScale ${scale}\n  voxOrigin bottom\n}\n`);
}

function hero(): Grid {
  // 8 x 6 x 16 at 0.1 m: a lamplighter in a teal hooded cloak and an amber scarf. The face is at y 0 (+z in the game).
  const g = new Grid(8, 6, 16);
  const boot = "#3a2a20", leg = "#2b3148", cloak = "#2f7a78", cloakDark = "#215857", belt = "#c98a2c", skin = "#e9b892", scarf = "#e6a23a", eye = "#1b1a2a";
  for (const x of [1, 2, 5, 6]) {
    g.box(x, 2, 0, x, 3, 1, boot);
    g.box(x, 2, 2, x, 3, 5, leg);
  }
  g.box(1, 1, 6, 6, 4, 10, cloak);
  g.box(1, 1, 6, 6, 1, 6, belt);
  g.box(1, 4, 6, 6, 4, 10, cloakDark);
  for (const x of [0, 7]) {
    g.box(x, 2, 7, x, 3, 10, cloak);
    g.box(x, 2, 6, x, 3, 6, skin);
  }
  g.box(1, 1, 11, 6, 4, 11, scarf);
  g.box(1, 1, 12, 6, 4, 14, skin);
  g.set(2, 1, 13, eye);
  g.set(5, 1, 13, eye);
  // The hood: over the top and the back, and down the sides of the face.
  g.box(1, 1, 15, 6, 4, 15, cloakDark);
  g.box(1, 2, 12, 6, 4, 14, cloakDark);
  g.box(1, 1, 12, 1, 1, 14, cloakDark);
  g.box(6, 1, 12, 6, 1, 14, cloakDark);
  g.box(2, 0, 14, 5, 0, 15, cloakDark);
  return g;
}

function lantern(): Grid {
  // 7 x 7 x 26 at 0.1 m: a stone foot, a red-lacquered post, an iron cage of amber glass, a lacquered cap.
  const g = new Grid(7, 7, 26);
  const stone = "#6b6672", lacquer = "#8e2a1e", lacquerDark = "#6a1d16", iron = "#232027", glass = "#f0c76a", brass = "#c9973a";
  g.box(0, 0, 0, 6, 6, 1, stone);
  g.box(2, 2, 2, 4, 4, 15, lacquer);
  g.box(2, 2, 2, 4, 2, 15, lacquerDark);
  g.box(1, 1, 16, 5, 5, 16, brass);
  // The cage: iron corners and a brass rim round an open middle, where the game hangs the glowing wick (src/world.ts).
  for (const [x, y] of [[1, 1], [1, 5], [5, 1], [5, 5]] as const) g.box(x, y, 17, x, y, 22, iron);
  for (const z of [17, 22]) {
    g.box(1, 1, z, 5, 1, z, brass);
    g.box(1, 5, z, 5, 5, z, brass);
    g.box(1, 1, z, 1, 5, z, brass);
    g.box(5, 1, z, 5, 5, z, brass);
  }
  g.set(3, 3, 17, glass);
  g.box(0, 0, 23, 6, 6, 23, lacquer);
  g.box(1, 1, 24, 5, 5, 24, lacquer);
  g.box(2, 2, 25, 4, 4, 25, lacquerDark);
  return g;
}

function tree(seed: number): Grid {
  // 15 x 15 x 28 at 0.15 m: a broadleaf, its canopy three blobs of dusk greens.
  const g = new Grid(15, 15, 28);
  const r = rng(seed);
  g.box(6, 6, 0, 8, 8, 11, "#4a3328");
  g.box(5, 7, 0, 9, 7, 1, "#3d2a21");
  g.box(7, 5, 0, 7, 9, 1, "#3d2a21");
  const blobs = [
    [7, 7, 16, 6.2],
    [4.5 + r() * 2, 8 + r(), 20, 4.6],
    [9 + r(), 5.5 + r(), 22, 4.2],
  ];
  const greens = ["#1f4d3d", "#2a6148", "#3b7652", "#4f8c5a"];
  for (let z = 8; z < 28; z++) {
    for (let y = 0; y < 15; y++) {
      for (let x = 0; x < 15; x++) {
        let inside = false;
        for (const [bx, by, bz, br] of blobs) {
          const d = Math.hypot(x - bx!, y - by!, (z - bz!) * 1.1);
          if (d < br! - hash2(x * 7 + z, y * 13, seed) * 1.1) inside = true;
        }
        if (!inside) continue;
        const light = clamp((z - 10) / 16 + (hash2(x, y * 3 + z, seed + 1) - 0.5) * 0.5, 0, 0.999);
        g.set(x, y, z, greens[Math.floor(light * greens.length)]!);
      }
    }
  }
  return g;
}

function pine(seed: number): Grid {
  // 11 x 11 x 30 at 0.15 m: a dark pine of stacked tiers (the far side's).
  const g = new Grid(11, 11, 30);
  g.box(5, 5, 0, 5, 5, 6, "#3d2a21");
  for (let z = 4; z < 30; z++) {
    const t = (z - 4) / 26;
    const tier = ((z - 4) % 6) / 6;
    const rad = (1 - t) * 5.4 * (0.75 + 0.35 * (1 - tier));
    for (let y = 0; y < 11; y++) {
      for (let x = 0; x < 11; x++) {
        const d = Math.hypot(x - 5, y - 5);
        if (d > rad - hash2(x, y + z * 11, seed) * 0.8) continue;
        g.set(x, y, z, hash2(x * 3, y + z, seed + 2) > 0.7 ? "#2c5a4f" : z > 24 ? "#3a6e5d" : "#1d3f39");
      }
    }
  }
  return g;
}

function rock(seed: number): Grid {
  // 7 x 6 x 5 at 0.15 m: a lumpy boulder, mossy on top.
  const g = new Grid(7, 6, 5);
  for (let z = 0; z < 5; z++) {
    for (let y = 0; y < 6; y++) {
      for (let x = 0; x < 7; x++) {
        const d = Math.hypot((x - 3) / 3.6, (y - 2.5) / 3.1, (z - 1.2) / 3.3);
        if (d > 1 - hash2(x * 5 + z, y * 3, seed) * 0.22) continue;
        const top = z >= 3 && !g.has(x, y, z + 1);
        g.set(x, y, z, top && hash2(x, y, seed + 5) > 0.35 ? "#4f7a5a" : hash2(x + z, y, seed + 9) > 0.5 ? "#6d6878" : "#5a5664");
      }
    }
  }
  return g;
}

function ruins(): Grid {
  // 24 x 6 x 20 at 0.2 m: a broken arch of dressed stone (the far side's).
  const g = new Grid(24, 6, 20);
  const stone = ["#8a8494", "#7a7486", "#9a94a2"];
  const s = (x: number, y: number, z: number) => stone[Math.floor(hash2(Math.floor(x / 2), Math.floor(z / 2), y) * 3)]!;
  for (let z = 0; z < 16; z++) for (let y = 1; y < 5; y++) for (let x = 2; x < 6; x++) g.set(x, y, z, s(x, y, z));
  for (let z = 0; z < 11; z++) for (let y = 1; y < 5; y++) for (let x = 18; x < 22; x++) g.set(x, y, z, s(x, y, z));
  // The lintel, broken off over the right pillar.
  for (let z = 16; z < 19; z++) for (let y = 1; y < 5; y++) for (let x = 1; x < 17; x++) if (!(x > 13 && z === 18)) g.set(x, y, z, s(x, y, z));
  // Moss along the tops, a fallen block at the foot.
  for (let x = 1; x < 17; x++) for (let y = 1; y < 5; y++) if (hash2(x, y, 3) > 0.4) g.set(x, y, 19, "#3f6e58");
  g.box(12, 0, 0, 15, 3, 2, "#7a7486");
  return g;
}

function voxels(): void {
  writeVox("hero", hero(), 0.1, "The lamplighter: 0.8 x 0.6 x 1.6 m, standing on y 0");
  writeVox("lantern", lantern(), 0.1, "A valley lantern: 0.7 m square, 2.6 m tall, standing on y 0");
  writeVox("tree", tree(7), 0.15, "A broadleaf: 2.25 m across, 4.2 m tall, standing on y 0");
  writeVox("rock", rock(3), 0.15, "A boulder of the pile: about 1 m across, standing on y 0");
  writeVox("pine", pine(5), 0.15, "A far-side pine (the far-side streaming group): 1.65 m across, 4.5 m tall");
  writeVox("ruins", ruins(), 0.2, "The far side's broken arch (the far-side streaming group): 4.8 m wide, 4 m tall");
}

// ------------------------------------------------------- the skinned slime

/**
 * The slime: a 10 x 8 x 10 voxel dome (0.1 m voxels, y up, its face towards
 * +z) meshed face by face into a glTF with a skin of two joints (`root` at
 * its base, `crown` 0.4 m up; each vertex weighed between them by height)
 * and two clips: `idle` (a 1 s squash and hop) and `chase` (a 0.5 s bound).
 * Its material is glassy: KHR_materials_transmission, volume and ior, with
 * the palette in COLOR_0.
 */
function slime(): Uint8Array {
  const SX = 10, SY = 8, SZ = 10, V = 0.1;
  const solid = new Map<string, [number, number, number]>();
  const body = hex("#5fd39a"), core = hex("#2f9a76"), eye = hex("#14202a"), shine = hex("#e8fff2");
  for (let y = 0; y < SY; y++) {
    for (let z = 0; z < SZ; z++) {
      for (let x = 0; x < SX; x++) {
        const d = Math.hypot((x + 0.5 - SX / 2) / (SX / 2), (y + 0.5) / 7.6, (z + 0.5 - SZ / 2) / (SZ / 2));
        if (d > 1) continue;
        let c = d < 0.7 ? core : body;
        if (z === SZ - 1 || (z === SZ - 2 && d > 0.85)) c = body;
        solid.set(`${x},${y},${z}`, c);
      }
    }
  }
  // Eyes on the front face, a shine on the crown.
  for (const [x, y] of [[3, 3], [6, 3], [3, 4], [6, 4]] as const) {
    for (let z = SZ - 1; z >= 0; z--) if (solid.has(`${x},${y},${z}`)) {
      solid.set(`${x},${y},${z}`, eye);
      break;
    }
  }
  for (let y = SY - 1; y >= 0; y--) if (solid.has(`3,${y},6`)) {
    solid.set(`3,${y},6`, shine);
    break;
  }
  const pos: number[] = [], nor: number[] = [], col: number[] = [], joints: number[] = [], weights: number[] = [], idx: number[] = [];
  const lin = (c: number) => {
    const s = c / 255;
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  const DIRS: [number, number, number][] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
  for (const [k, c] of [...solid.entries()].sort()) {
    const [x, y, z] = k.split(",").map(Number) as [number, number, number];
    for (const [dx, dy, dz] of DIRS) {
      if (solid.has(`${x + dx},${y + dy},${z + dz}`)) continue;
      // The face's four corners, counter-clockwise seen from outside.
      const axis = dx !== 0 ? 0 : dy !== 0 ? 1 : 2;
      const sign = dx + dy + dz;
      const u = (axis + 1) % 3, w = (axis + 2) % 3;
      const base = [x, y, z];
      base[axis]! += sign > 0 ? 1 : 0;
      const corners: number[][] = [];
      for (const [a, b] of sign > 0 ? [[0, 0], [1, 0], [1, 1], [0, 1]] : [[0, 0], [0, 1], [1, 1], [1, 0]]) {
        const p = [...base];
        p[u]! += a!;
        p[w]! += b!;
        corners.push(p);
      }
      const first = pos.length / 3;
      for (const p of corners) {
        const px = (p[0]! - SX / 2) * V, py = p[1]! * V, pz = (p[2]! - SZ / 2) * V;
        pos.push(px, py, pz);
        nor.push(dx, dy, dz);
        col.push(lin(c[0]), lin(c[1]), lin(c[2]), 1);
        const crown = smoothstep(0.05, 0.6, py);
        joints.push(0, 1, 0, 0);
        weights.push(1 - crown, crown, 0, 0);
      }
      idx.push(first, first + 1, first + 2, first, first + 2, first + 3);
    }
  }
  // Clips: the root hops, the crown squashes and stretches.
  const idle = {
    times: [0, 0.25, 0.5, 0.75, 1],
    root: [0, 0, 0, 0, 0, 0, 0, 0.12, 0, 0, 0.03, 0, 0, 0, 0],
    scale: [1, 1, 1, 1.14, 0.84, 1.14, 0.92, 1.12, 0.92, 1.05, 0.95, 1.05, 1, 1, 1],
  };
  const chase = {
    times: [0, 0.125, 0.25, 0.375, 0.5],
    root: [0, 0, 0, 0, 0, 0, 0, 0.3, 0, 0, 0.1, 0, 0, 0, 0],
    scale: [1, 1, 1, 1.22, 0.76, 1.22, 0.86, 1.22, 0.86, 1.08, 0.94, 1.08, 1, 1, 1],
  };
  const crownY = 0.4;
  // Binary layout: every array 4-aligned.
  const blobs: { data: Uint8Array; target?: number }[] = [];
  const views: object[] = [];
  const accessors: object[] = [];
  let offset = 0;
  const add = (data: ArrayBufferView, target: number | undefined, accessor: object) => {
    const bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    const pad = (4 - (bytes.length % 4)) % 4;
    blobs.push({ data: bytes });
    if (pad) blobs.push({ data: new Uint8Array(pad) });
    views.push({ buffer: 0, byteOffset: offset, byteLength: bytes.length, ...(target ? { target } : {}) });
    offset += bytes.length + pad;
    accessors.push({ bufferView: views.length - 1, ...accessor });
    return accessors.length - 1;
  };
  const min = [Infinity, Infinity, Infinity], max = [-Infinity, -Infinity, -Infinity];
  for (let i = 0; i < pos.length; i += 3) for (let k = 0; k < 3; k++) {
    min[k] = Math.min(min[k]!, pos[i + k]!);
    max[k] = Math.max(max[k]!, pos[i + k]!);
  }
  const n = pos.length / 3;
  const aPos = add(new Float32Array(pos), 34962, { componentType: 5126, count: n, type: "VEC3", min, max });
  const aNor = add(new Float32Array(nor), 34962, { componentType: 5126, count: n, type: "VEC3" });
  const aCol = add(new Float32Array(col), 34962, { componentType: 5126, count: n, type: "VEC4" });
  const aJoi = add(new Uint8Array(joints), 34962, { componentType: 5121, count: n, type: "VEC4" });
  const aWei = add(new Float32Array(weights), 34962, { componentType: 5126, count: n, type: "VEC4" });
  const aIdx = add(new Uint16Array(idx), 34963, { componentType: 5123, count: idx.length, type: "SCALAR" });
  const ibm = new Float32Array(32);
  for (const m of [0, 16]) for (let k = 0; k < 4; k++) ibm[m + k * 5] = 1;
  ibm[16 + 13] = -crownY;
  const aIbm = add(ibm, undefined, { componentType: 5126, count: 2, type: "MAT4" });
  const animations: object[] = [];
  for (const [name, clip] of [["idle", idle], ["chase", chase]] as const) {
    const t = add(new Float32Array(clip.times), undefined, { componentType: 5126, count: clip.times.length, type: "SCALAR", min: [0], max: [clip.times[clip.times.length - 1]!] });
    const rootOut = add(new Float32Array(clip.root), undefined, { componentType: 5126, count: clip.times.length, type: "VEC3" });
    const crownT = clip.times.map(() => [0, crownY, 0]).flat();
    const crownOut = add(new Float32Array(crownT), undefined, { componentType: 5126, count: clip.times.length, type: "VEC3" });
    const scaleOut = add(new Float32Array(clip.scale), undefined, { componentType: 5126, count: clip.times.length, type: "VEC3" });
    animations.push({
      name,
      samplers: [
        { input: t, output: rootOut, interpolation: "LINEAR" },
        { input: t, output: crownOut, interpolation: "LINEAR" },
        { input: t, output: scaleOut, interpolation: "LINEAR" },
      ],
      channels: [
        { sampler: 0, target: { node: 1, path: "translation" } },
        { sampler: 1, target: { node: 2, path: "translation" } },
        { sampler: 2, target: { node: 2, path: "scale" } },
      ],
    });
  }
  const json = {
    asset: { version: "2.0", generator: "lantern-vale tools/make-assets.ts" },
    extensionsUsed: ["KHR_materials_transmission", "KHR_materials_volume", "KHR_materials_ior"],
    scene: 0,
    scenes: [{ nodes: [0, 1] }],
    nodes: [
      { name: "slime", mesh: 0, skin: 0 },
      { name: "root", children: [2] },
      { name: "crown", translation: [0, crownY, 0] },
    ],
    skins: [{ joints: [1, 2], inverseBindMatrices: aIbm, skeleton: 1 }],
    meshes: [{ name: "slime", primitives: [{ attributes: { POSITION: aPos, NORMAL: aNor, COLOR_0: aCol, JOINTS_0: aJoi, WEIGHTS_0: aWei }, indices: aIdx, material: 0 }] }],
    materials: [{
      name: "slime-jelly",
      pbrMetallicRoughness: { baseColorFactor: [1, 1, 1, 1], metallicFactor: 0, roughnessFactor: 0.22 },
      extensions: {
        KHR_materials_transmission: { transmissionFactor: 0.65 },
        KHR_materials_volume: { thicknessFactor: 0.5, attenuationDistance: 0.45, attenuationColor: [0.35, 0.95, 0.6] },
        KHR_materials_ior: { ior: 1.34 },
      },
    }],
    animations,
    accessors,
    bufferViews: views,
    buffers: [{ byteLength: offset }],
  };
  const jsonBytes = new TextEncoder().encode(JSON.stringify(json));
  const jpad = (4 - (jsonBytes.length % 4)) % 4;
  const total = 12 + 8 + jsonBytes.length + jpad + 8 + offset;
  const out = new Uint8Array(total);
  const dv = new DataView(out.buffer);
  dv.setUint32(0, 0x46546c67, true);
  dv.setUint32(4, 2, true);
  dv.setUint32(8, total, true);
  dv.setUint32(12, jsonBytes.length + jpad, true);
  dv.setUint32(16, 0x4e4f534a, true);
  out.set(jsonBytes, 20);
  out.fill(0x20, 20 + jsonBytes.length, 20 + jsonBytes.length + jpad);
  let at = 20 + jsonBytes.length + jpad;
  dv.setUint32(at, offset, true);
  dv.setUint32(at + 4, 0x004e4942, true);
  at += 8;
  for (const b of blobs) {
    out.set(b.data, at);
    at += b.data.length;
  }
  return out;
}

// ------------------------------------------------------------------ music

/**
 * "Dusk over the vale": 4 minutes at 72 bpm in D dorian, 72 bars of 4/4.
 * A slow pad changes chord every two bars; a plucked arpeggio walks the
 * chord in eighths; from bar 17 a soft bass on beats one and three, from
 * bar 33 a bell melody drawn from the pentatonic by a seeded walk; bars 57
 * to 72 thin back to the pad and the pluck and fade out. 48 kHz mono.
 */
function music(): Uint8Array {
  const RATE = 48000;
  const BPM = 72;
  const BARS = 72;
  const beat = 60 / BPM;
  const seconds = BARS * 4 * beat;
  const N = Math.round(seconds * RATE);
  const out = new Float32Array(N);
  const midi = (m: number) => 440 * 2 ** ((m - 69) / 12);
  // D dorian chords, two bars each: Dm9, G6/D, Fmaj7, C6/E, Bbmaj7, Am7, Gm9, Asus4.
  const CHORDS = [
    [50, 57, 60, 64, 65],
    [50, 55, 59, 62, 64],
    [53, 57, 60, 64, 67],
    [52, 55, 60, 62, 67],
    [46, 53, 57, 62, 65],
    [45, 52, 55, 60, 64],
    [43, 50, 53, 57, 62],
    [45, 50, 52, 57, 62],
  ];
  const PENTA = [62, 64, 65, 67, 69, 72, 74, 76];
  const r = rng(2026);
  const addTone = (start: number, dur: number, f: number, amp: number, kind: "pad" | "pluck" | "bass" | "bell") => {
    const s0 = Math.floor(start * RATE);
    const len = Math.floor(dur * RATE);
    for (let i = 0; i < len && s0 + i < N; i++) {
      const t = i / RATE;
      let env: number;
      let v: number;
      if (kind === "pad") {
        env = Math.min(1, t / 1.6) * Math.min(1, (dur - t) / 1.6);
        v = Math.sin(2 * Math.PI * f * t) + 0.5 * Math.sin(2 * Math.PI * f * 1.003 * t + 1) + 0.25 * Math.sin(2 * Math.PI * f * 2.001 * t);
      } else if (kind === "pluck") {
        env = Math.min(1, t / 0.004) * Math.exp(-t * 5);
        v = Math.sin(2 * Math.PI * f * t) + 0.35 * Math.sin(4 * Math.PI * f * t) * Math.exp(-t * 9) + 0.15 * Math.sin(6 * Math.PI * f * t) * Math.exp(-t * 14);
      } else if (kind === "bass") {
        env = Math.min(1, t / 0.02) * Math.exp(-t * 1.6);
        v = Math.sin(2 * Math.PI * f * t) + 0.2 * Math.sin(4 * Math.PI * f * t);
      } else {
        env = Math.min(1, t / 0.003) * Math.exp(-t * 2.2);
        const mod = Math.sin(2 * Math.PI * f * 3.5 * t) * 2.2 * Math.exp(-t * 3);
        v = Math.sin(2 * Math.PI * f * t + mod);
      }
      out[s0 + i]! += v * env * amp;
    }
  };
  let note = 3;
  for (let bar = 0; bar < BARS; bar++) {
    const chord = CHORDS[Math.floor(bar / 2) % CHORDS.length]!;
    const t0 = bar * 4 * beat;
    // The section's level: in over the first bars, out over the last eight.
    const level = Math.min(1, (bar + 1) / 4) * Math.min(1, (BARS - bar) / 8);
    if (bar % 2 === 0) for (const m of chord.slice(1)) addTone(t0, 8 * beat + 0.8, midi(m), 0.045 * level, "pad");
    for (let e = 0; e < 8; e++) {
      const m = chord[1 + ((e * 3 + (bar % 3)) % 4)]! + 12;
      addTone(t0 + e * beat * 0.5, 1.4, midi(m), 0.05 * level * (e % 2 === 0 ? 1 : 0.7), "pluck");
    }
    if (bar >= 16 && bar < 64) for (const b of [0, 2]) addTone(t0 + b * beat, 1.8, midi(chord[0]! - 12), 0.11 * level, "bass");
    if (bar >= 32 && bar < 60) {
      for (let q = 0; q < 4; q++) {
        if (r() < 0.45) continue;
        note = clamp(note + Math.floor(r() * 5) - 2, 0, PENTA.length - 1);
        addTone(t0 + q * beat + (r() < 0.3 ? beat / 2 : 0), 2.5, midi(PENTA[note]! + 12), 0.06 * level, "bell");
      }
    }
  }
  // A one-pole low-pass for warmth, then normalise to -3 dBFS.
  let lp = 0;
  let peak = 0;
  for (let i = 0; i < N; i++) {
    lp += (out[i]! - lp) * 0.35;
    out[i] = lp;
    peak = Math.max(peak, Math.abs(lp));
  }
  const gain = 0.7 / peak;
  const pcm = new Int16Array(N);
  for (let i = 0; i < N; i++) pcm[i] = Math.round(clamp(out[i]! * gain, -1, 1) * 32767);
  const wav = new Uint8Array(44 + pcm.byteLength);
  const dv = new DataView(wav.buffer);
  const tag = (at: number, s: string) => {
    for (let i = 0; i < 4; i++) wav[at + i] = s.charCodeAt(i);
  };
  tag(0, "RIFF");
  dv.setUint32(4, 36 + pcm.byteLength, true);
  tag(8, "WAVE");
  tag(12, "fmt ");
  dv.setUint32(16, 16, true);
  dv.setUint16(20, 1, true);
  dv.setUint16(22, 1, true);
  dv.setUint32(24, RATE, true);
  dv.setUint32(28, RATE * 2, true);
  dv.setUint16(32, 2, true);
  dv.setUint16(34, 16, true);
  tag(36, "data");
  dv.setUint32(40, pcm.byteLength, true);
  wav.set(new Uint8Array(pcm.buffer), 44);
  return wav;
}

// ------------------------------------------------------------------- main

textures();
faces();
terrain();
environment();
voxels();
write("models/slime.glb", slime());
if (!process.argv.includes("--no-music")) write("music/dusk.wav", music());
console.log(`made Lantern Vale's assets under ${ASSETS}`);
