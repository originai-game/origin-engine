// Generates the valley's sources: assets/terrain/valley-height.png (16-bit
// grey) and its two control maps (valley-control-a.png: meadow, rock, shore,
// snow; valley-control-b.png: forest floor, path, field, alpine).
//
//   bun tools/terrain.ts
//
// Everything is written in scene metres: the camera stands on the knoll at
// the origin looking down -z, the lake's surface is y 0. `layout.ts` holds
// the places the game reads too (the knoll, the village, the paths).
import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { CENTER, FIELD, HEIGHT, KNOLL, LAKE, MILL, PADDOCK, PATHS, SAMPLES, SIZE, VILLAGE } from "../src/layout.ts";

const out = join(import.meta.dir, "..", "assets", "terrain");
mkdirSync(out, { recursive: true });

// ---- noise: lattice gradient noise, bit-exact everywhere ----
function hash(x: number, y: number, seed: number): number {
  let h = (Math.imul(x, 374761393) + Math.imul(y, 668265263) + Math.imul(seed, 1274126177)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
function gradient(ix: number, iy: number, fx: number, fy: number, seed: number): number {
  const a = hash(ix, iy, seed) * Math.PI * 2;
  return Math.cos(a) * fx + Math.sin(a) * fy;
}
const fade = (t: number) => t * t * t * (t * (t * 6 - 15) + 10);
/** Gradient noise in about -1 to 1. */
function noise(x: number, y: number, seed: number): number {
  const ix = Math.floor(x), iy = Math.floor(y), fx = x - ix, fy = y - iy;
  const u = fade(fx), v = fade(fy);
  const a = gradient(ix, iy, fx, fy, seed), b = gradient(ix + 1, iy, fx - 1, fy, seed);
  const c = gradient(ix, iy + 1, fx, fy - 1, seed), d = gradient(ix + 1, iy + 1, fx - 1, fy - 1, seed);
  return (a + (b - a) * u + (c - a + (d - c - (b - a)) * u) * v) * 1.6;
}
function fbm(x: number, y: number, octaves: number, seed: number, gain = 0.5): number {
  let sum = 0, amp = 1, norm = 0;
  for (let o = 0; o < octaves; o++) {
    sum += amp * noise(x, y, seed + o);
    norm += amp;
    amp *= gain;
    x = x * 2.03 + 17.1;
    y = y * 2.03 - 9.7;
  }
  return sum / norm;
}
/** Ridged multifractal in 0 to 1: sharp crests, valleys between. */
function ridged(x: number, y: number, octaves: number, seed: number): number {
  let sum = 0, amp = 0.5, weight = 1, norm = 0;
  for (let o = 0; o < octaves; o++) {
    let n = 1 - Math.abs(noise(x, y, seed + o));
    n *= n;
    sum += n * amp * weight;
    norm += amp;
    weight = Math.min(1, Math.max(0, n * 2));
    amp *= 0.5;
    x = x * 2.07 + 31.3;
    y = y * 2.07 + 11.9;
  }
  return sum / norm;
}

const clamp01 = (t: number) => Math.min(1, Math.max(0, t));
const smooth = (a: number, b: number, t: number) => {
  const k = clamp01((t - a) / (b - a));
  return k * k * (3 - 2 * k);
};
const mix = (a: number, b: number, t: number) => a + (b - a) * t;

/** Distance from (x, z) to a polyline, and how far along it (0 to 1). */
function toPolyline(x: number, z: number, points: readonly (readonly [number, number])[]): [number, number] {
  let best = Infinity, along = 0, total = 0, at = 0;
  const lengths: number[] = [];
  for (let i = 0; i + 1 < points.length; i++) {
    const [ax, az] = points[i]!, [bx, bz] = points[i + 1]!;
    lengths.push(Math.hypot(bx - ax, bz - az));
    total += lengths[i]!;
  }
  for (let i = 0; i + 1 < points.length; i++) {
    const [ax, az] = points[i]!, [bx, bz] = points[i + 1]!;
    const dx = bx - ax, dz = bz - az;
    const t = clamp01(((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz));
    const d = Math.hypot(x - ax - dx * t, z - az - dz * t);
    if (d < best) {
      best = d;
      along = (at + lengths[i]! * t) / total;
    }
    at += lengths[i]!;
  }
  return [best, along];
}

/** The lake's edge: negative inside, metres from the shoreline. */
function lakeEdge(x: number, z: number): number {
  const [d, along] = toPolyline(x, z, LAKE.spine);
  const half = mix(LAKE.widthNear, LAKE.widthFar, along);
  // A shoreline that wanders, but for the village's bay, which the houses and the jetty are placed against.
  const wander = (fbm(x / 160, z / 160, 3, 91) * 22 + fbm(x / 45, z / 45, 2, 97) * 5) * (1 - 0.85 * VILLAGE.shelf(x, z));
  return d - half - wander;
}

/** A round hill: 1 at its centre, 0 past its radius. */
function bump(x: number, z: number, cx: number, cz: number, radius: number): number {
  const d = Math.hypot(x - cx, z - cz) / radius;
  return d >= 1 ? 0 : (1 - d * d) ** 2;
}

/** The ground's height at a scene point (metres above the lake). */
export function ground(x: number, z: number): number {
  const edge = lakeEdge(x, z);
  const far = Math.hypot(x, z);
  // Hills rise away from the lake; the near right shore (the village) stays low.
  const shelf = VILLAGE.shelf(x, z);
  const rise = smooth(20, 1400, edge) * 260 * (1 - 0.75 * shelf);
  const roll = fbm(x / 620, z / 620, 3, 3, 0.42) * (26 + 110 * smooth(120, 1500, edge));
  const small = fbm(x / 90, z / 90, 3, 11, 0.45) * 2.4 * smooth(4, 60, edge) * (1 - 0.6 * shelf);
  // The shore: a gentle bank just above the water, then the land.
  let h = smooth(0, 90, edge) * 9 + rise + roll * smooth(30, 400, edge) + small;
  // Mountains: a range from the centre to the right, a lower one to the left
  // where the lake's valley runs out and the sun rises.
  const bearing = Math.atan2(x, -z); // 0 ahead, positive to the right
  const gap = Math.exp(-(((bearing + 0.27) / 0.2) ** 2)); // the sunrise valley
  const range = smooth(2500, 5200, far) * (1 - 0.82 * gap) * smooth(-2.4, -1.4, bearing) * (0.72 + 0.28 * smooth(-0.3, 0.5, bearing));
  const warpX = fbm(x / 2100, z / 2100, 3, 41) * 520, warpZ = fbm(x / 2100, z / 2100, 3, 47) * 520;
  const crest = ridged((x + warpX) / 2300, (z + warpZ) / 2300, 7, 53);
  // Gullies and spurs down the flanks, so rock breaks through the snow.
  const spurs = ridged(x / 430, z / 430, 4, 61);
  h += range * (120 + 1500 * crest ** 1.35 + 190 * crest * (spurs - 0.5)) + smooth(900, 2600, far) * 120 * (1 - 0.7 * gap);
  // Behind the camera the land keeps rising gently into wooded hills.
  h += smooth(60, 900, z) * 90;
  // The knoll the shepherd stands on: the brow of the hill behind the
  // camera, a rounded top falling away to the lake and the paddock.
  const knoll = KNOLL.height * (1 - smooth(KNOLL.top, KNOLL.radius, Math.hypot(x - KNOLL.x, z - KNOLL.z))) ** 1.5;
  const brow = smooth(-30, 70, z) * (1 - smooth(120, 420, Math.abs(x))) * (KNOLL.height + smooth(40, 500, z) * 40);
  h = Math.max(h, brow, knoll);
  // The windmill's knoll.
  const mill = bump(x, z, MILL.x, MILL.z, MILL.radius);
  h = Math.max(h, mix(h, MILL.height, smooth(0, 0.55, mill) * smooth(0, 40, edge)));
  // The village's shelf: gentle, even ground.
  h = mix(h, VILLAGE.height(x, z), shelf * smooth(0, 24, edge));
  // The lake bed.
  if (edge < 0) h = Math.max(-26, edge * 0.09 - 0.25 + fbm(x / 60, z / 60, 3, 71) * 0.8 * smooth(0, -40, edge));
  else h = Math.max(h, 0.12 + smooth(0, 14, edge) * 0.8);
  return h;
}

// ---- sample the field ----
const n = SAMPLES, spacing = SIZE / (n - 1);
const heights = new Float32Array(n * n);
console.time("heights");
for (let j = 0; j < n; j++) {
  const z = CENTER.z - SIZE / 2 + j * spacing;
  for (let i = 0; i < n; i++) heights[j * n + i] = ground(CENTER.x - SIZE / 2 + i * spacing, z);
}
console.timeEnd("heights");

// ---- PNG ----
const crcTable = new Uint32Array(256).map((_, k) => {
  let c = k;
  for (let i = 0; i < 8; i++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
function crc(bytes: Uint8Array): number {
  let c = 0xffffffff;
  for (const b of bytes) c = crcTable[(c ^ b) & 255]! ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type: string, data: Uint8Array): Uint8Array {
  const o = new Uint8Array(12 + data.length), view = new DataView(o.buffer);
  view.setUint32(0, data.length);
  for (let i = 0; i < 4; i++) o[4 + i] = type.charCodeAt(i);
  o.set(data, 8);
  view.setUint32(8 + data.length, crc(o.subarray(4, 8 + data.length)));
  return o;
}
/** A PNG of `channels` samples a pixel, `depth` bits each; rows carry a filter byte of 0. */
function png(width: number, height: number, depth: 8 | 16, colour: 0 | 6, rows: Uint8Array): Uint8Array {
  const head = new Uint8Array(13), view = new DataView(head.buffer);
  view.setUint32(0, width);
  view.setUint32(4, height);
  head[8] = depth;
  head[9] = colour;
  const parts = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", head), chunk("IDAT", deflateSync(rows, { level: 6 })), chunk("IEND", new Uint8Array(0))];
  const o = new Uint8Array(parts.reduce((s, p) => s + p.length, 0));
  let at = 0;
  for (const p of parts) {
    o.set(p, at);
    at += p.length;
  }
  return o;
}

{
  const rows = new Uint8Array(n * (1 + n * 2));
  let low = Infinity, high = -Infinity;
  for (let j = 0; j < n; j++) {
    const row = j * (1 + n * 2);
    for (let i = 0; i < n; i++) {
      const h = heights[j * n + i]!;
      low = Math.min(low, h);
      high = Math.max(high, h);
      const v = Math.round(clamp01((h - HEIGHT.low) / (HEIGHT.high - HEIGHT.low)) * 65535);
      rows[row + 1 + i * 2] = v >> 8;
      rows[row + 2 + i * 2] = v & 255;
    }
  }
  if (low < HEIGHT.low || high > HEIGHT.high) throw new Error(`heights ${low} to ${high} leave ${HEIGHT.low} to ${HEIGHT.high}`);
  writeFileSync(join(out, "valley-height.png"), png(n, n, 16, 0, rows));
  console.log(`valley-height.png ${n}x${n}, ${low.toFixed(1)} to ${high.toFixed(1)} m`);
}

// ---- control maps ----
{
  const a = new Uint8Array(n * (1 + n * 4)), b = new Uint8Array(n * (1 + n * 4));
  const at = (i: number, j: number) => heights[Math.min(n - 1, Math.max(0, j)) * n + Math.min(n - 1, Math.max(0, i))]!;
  for (let j = 0; j < n; j++) {
    const z = CENTER.z - SIZE / 2 + j * spacing;
    for (let i = 0; i < n; i++) {
      const x = CENTER.x - SIZE / 2 + i * spacing;
      const h = at(i, j);
      const slope = Math.atan(Math.hypot(at(i + 1, j) - at(i - 1, j), at(i, j + 1) - at(i, j - 1)) / (2 * spacing)) * 180 / Math.PI;
      const patch = fbm(x / 260, z / 260, 4, 131), fine = fbm(x / 38, z / 38, 3, 137);
      const edge = lakeEdge(x, z);
      const rock = smooth(31, 40, slope + fine * 5 + 9 * smooth(500, 900, h)) * smooth(3, 12, h);
      const snow = smooth(690, 860, h + patch * 150 + fine * 40) * (1 - smooth(48, 62, slope));
      const shore = 1 - smooth(0.5, 2.6, h + fine * 0.5);
      const alpine = smooth(380, 560, h + patch * 90) * (1 - snow);
      // Paths: packed earth from the gate to the village and along the shore.
      let path = 0;
      for (const p of PATHS) path = Math.max(path, 1 - smooth(p.width * 0.35, p.width * 0.75, toPolyline(x, z, p.points)[0] + fine * 0.6));
      const field = FIELD.weight(x, z) * (1 - smooth(14, 22, slope));
      const open = Math.max(smooth(0, 0.3, PADDOCK.weight(x, z)), smooth(0.25, 0.6, VILLAGE.shelf(x, z)), 1 - smooth(60, 85, Math.hypot(x - KNOLL.x, z - KNOLL.z)), smooth(0, 0.4, bump(x, z, MILL.x, MILL.z, MILL.radius * 1.3)), field);
      // Woods: patches on the hills, thinning at the tree line, kept off the open ground.
      const woods = smooth(-0.08, 0.2, patch + 0.25 * smooth(40, 500, edge) + fine * 0.12) * smooth(6, 30, edge) * (1 - smooth(420, 560, h + fine * 30)) * (1 - smooth(34, 42, slope)) * (1 - open);
      let w = [0, rock, shore, snow, woods, path, field, alpine];
      // Each later claim covers the ones before it; the meadow takes what is left.
      const cover = (k: number, over: number[]) => { for (const o of over) w[o]! *= 1 - w[k]!; };
      cover(3, [7, 4, 6, 5]);
      cover(1, [3, 7, 4, 6, 5]);
      cover(2, [1, 4, 6, 5, 7]);
      cover(5, [4, 6, 7]);
      cover(6, [4, 7]);
      cover(7, [4]);
      w[0] = clamp01(1 - w.slice(1).reduce((s, v) => s + v, 0));
      const row = j * (1 + n * 4) + 1 + i * 4;
      for (let k = 0; k < 4; k++) {
        a[row + k] = Math.round(clamp01(w[k]!) * 255);
        b[row + k] = Math.round(clamp01(w[4 + k]!) * 255);
      }
    }
  }
  writeFileSync(join(out, "valley-control-a.png"), png(n, n, 8, 6, a));
  writeFileSync(join(out, "valley-control-b.png"), png(n, n, 8, 6, b));
  console.log("valley-control-a.png, valley-control-b.png");
}

// ---- a map to place things by (not an asset): og-out/valley-map.png, 1 px a sample around the village ----
{
  const x0 = -220, z0 = -260, w = 1120, hgt = 840, step = 0.5;
  const rows = new Uint8Array(hgt * (1 + w * 4));
  const mark = (x: number, z: number, r: number, c: [number, number, number]) => {
    for (let dz = -r; dz <= r; dz++) for (let dx = -r; dx <= r; dx++) {
      const px = Math.round((x - x0) / step) + dx, pz = Math.round((z - z0) / step) + dz;
      if (px < 0 || pz < 0 || px >= w || pz >= hgt) continue;
      rows.set([c[0], c[1], c[2], 255], pz * (1 + w * 4) + 1 + px * 4);
    }
  };
  for (let j = 0; j < hgt; j++) for (let i = 0; i < w; i++) {
    const x = x0 + i * step, z = z0 + j * step, h = ground(x, z);
    const shade = clamp01(0.75 + (ground(x - 2, z - 2) - h) * 0.25);
    const band = Math.floor(h / 5) % 2 === 0 ? 1 : 0.9;
    const c = h < 0 ? [40, 90, 150] : h < 2.2 ? [200, 190, 140] : [70 + h * 1.2, 150 - h * 0.5, 60];
    rows.set([c[0]! * shade * band, c[1]! * shade * band, c[2]! * shade * band, 255].map(v => Math.min(255, Math.max(0, Math.round(v)))), j * (1 + w * 4) + 1 + i * 4);
  }
  for (const p of PATHS) for (const [x, z] of p.points) mark(x, z, 1, [120, 80, 40]);
  for (const hse of VILLAGE.houses) mark(hse.x, hse.z, 4, [200, 40, 40]);
  mark(VILLAGE.jetty.x, VILLAGE.jetty.z, 3, [255, 255, 0]);
  for (const [x, z] of PADDOCK.flock) mark(x, z, 1, [255, 255, 255]);
  mark(MILL.x, MILL.z, 5, [255, 0, 255]);
  mark(KNOLL.x, KNOLL.z, 3, [255, 255, 255]);
  for (const [x, z] of PADDOCK.wall) mark(x, z, 2, [0, 0, 0]);
  mkdirSync(join(import.meta.dir, "..", "og-out"), { recursive: true });
  writeFileSync(join(import.meta.dir, "..", "og-out", "valley-map.png"), png(w, hgt, 8, 6, rows));
}
