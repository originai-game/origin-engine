// The engine's built-in look assets (og-cook's `BUILTINS`, named from a
// look as "@canvas/<name>" and "@palettes/<name>"), made by this script and
// committed with its output: `bun templates/looks/assets/generate.ts`.
//
//   canvas/paper.png       a fine, light paper grain with scattered fibres
//   canvas/linen.png       a rough woven canvas, threads over and under
//   canvas/watercolor.png  cold-pressed watercolour paper: soft bumps, pits
//   palettes/*.hex         the palettes, one rrggbb a line
//
// Every texture is 256 x 256 grey, tileable (each noise wraps at the tile's
// edge) and made from integer hashes of fixed seeds, so a run writes the same
// bytes. The paint pass reads the red channel as the grain
// (stylise.wgsl `paint`): light means no change, darker darkens the frame by
// up to half of Stylise.paintCanvasStrength. All of it is the engine's own
// work, CC0.
import { deflateSync } from "node:zlib";
import { writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const SIZE = 256;
const here = import.meta.dir;

/** A 32-bit integer hash of (x, y, seed) in [0, 1). */
function hash(x: number, y: number, seed: number): number {
  let h = Math.imul(x | 0, 0x27d4eb2d) ^ Math.imul(y | 0, 0x165667b1) ^ Math.imul(seed | 0, 0x9e3779b1);
  h = Math.imul(h ^ (h >>> 15), 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}

const smooth = (t: number) => t * t * (3 - 2 * t);
const wrap = (v: number, p: number) => ((v % p) + p) % p;

/** Value noise on a lattice of `period` cells across the tile, wrapping. */
function valueNoise(x: number, y: number, period: number, seed: number): number {
  const fx = (x / SIZE) * period;
  const fy = (y / SIZE) * period;
  const ix = Math.floor(fx);
  const iy = Math.floor(fy);
  const tx = smooth(fx - ix);
  const ty = smooth(fy - iy);
  const at = (i: number, j: number) => hash(wrap(i, period), wrap(j, period), seed);
  const a = at(ix, iy) + (at(ix + 1, iy) - at(ix, iy)) * tx;
  const b = at(ix, iy + 1) + (at(ix + 1, iy + 1) - at(ix, iy + 1)) * tx;
  return a + (b - a) * ty;
}

/** Octaves of value noise from `period` up, each half as strong: in [0, 1). */
function fbm(x: number, y: number, period: number, octaves: number, seed: number): number {
  let sum = 0;
  let norm = 0;
  let amp = 1;
  for (let o = 0; o < octaves; o++) {
    sum += valueNoise(x, y, period << o, seed + o * 101) * amp;
    norm += amp;
    amp *= 0.5;
  }
  return sum / norm;
}

/** A grey image as a PNG (8-bit greyscale, each row Paeth-filtered). */
function png(pixels: Uint8Array): Buffer {
  const rows = Buffer.alloc((SIZE + 1) * SIZE);
  for (let y = 0; y < SIZE; y++) {
    rows[y * (SIZE + 1)] = 4;
    for (let x = 0; x < SIZE; x++) {
      const a = x > 0 ? pixels[y * SIZE + x - 1]! : 0;
      const b = y > 0 ? pixels[(y - 1) * SIZE + x]! : 0;
      const c = x > 0 && y > 0 ? pixels[(y - 1) * SIZE + x - 1]! : 0;
      const p = a + b - c;
      const pa = Math.abs(p - a);
      const pb = Math.abs(p - b);
      const pc = Math.abs(p - c);
      const pred = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      rows[y * (SIZE + 1) + 1 + x] = (pixels[y * SIZE + x]! - pred) & 255;
    }
  }
  const crcTable = Array.from({ length: 256 }, (_, n) => {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    return c >>> 0;
  });
  const crc = (buf: Buffer) => {
    let c = 0xffffffff;
    for (const byte of buf) c = crcTable[(c ^ byte) & 255]! ^ (c >>> 8);
    return (c ^ 0xffffffff) >>> 0;
  };
  const chunk = (type: string, data: Buffer) => {
    const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
    const len = Buffer.alloc(4);
    len.writeUInt32BE(data.length);
    const sum = Buffer.alloc(4);
    sum.writeUInt32BE(crc(body));
    return Buffer.concat([len, body, sum]);
  };
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(SIZE, 0);
  ihdr.writeUInt32BE(SIZE, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 0; // greyscale
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(rows, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function image(f: (x: number, y: number) => number): Uint8Array {
  const out = new Uint8Array(SIZE * SIZE);
  for (let y = 0; y < SIZE; y++) for (let x = 0; x < SIZE; x++) out[y * SIZE + x] = Math.max(0, Math.min(255, Math.round(f(x, y))));
  return out;
}

/** Short fibres laid into `img`, wrapping: each a few pixels darker or lighter. */
function fibres(img: Uint8Array, count: number, seed: number, strength: number) {
  for (let i = 0; i < count; i++) {
    const x0 = hash(i, 1, seed) * SIZE;
    const y0 = hash(i, 2, seed) * SIZE;
    const angle = hash(i, 3, seed) * Math.PI;
    const length = 6 + hash(i, 4, seed) * 22;
    const shade = (hash(i, 5, seed) < 0.6 ? -1 : 1) * strength * (0.5 + hash(i, 6, seed) * 0.5);
    const bend = (hash(i, 7, seed) - 0.5) * 0.08;
    for (let s = 0; s < length; s += 0.5) {
      const a = angle + bend * s;
      const x = Math.floor(wrap(x0 + Math.cos(a) * s, SIZE));
      const y = Math.floor(wrap(y0 + Math.sin(a) * s, SIZE));
      const k = y * SIZE + x;
      img[k] = Math.max(0, Math.min(255, img[k]! + shade * 0.5));
    }
  }
}

// A light paper: soft cloudiness, a fine tooth and fibres.
const paper = image((x, y) => {
  const cloud = fbm(x, y, 4, 4, 11);
  const tooth = valueNoise(x, y, 128, 12) * 0.6 + hash(x, y, 13) * 0.4;
  return 236 + (cloud - 0.5) * 22 + (tooth - 0.5) * 16;
});
fibres(paper, 420, 14, 14);

// A rough linen canvas: a plain weave of threads every 4 px, each passing
// over one crossing and under the next (so it rises and dips along its
// length), of its own thickness and shade, wobbling a little, with slubs
// along it and a coarse tooth.
const THREAD = 4;
const linen = image((x0, y0) => {
  // The threads wobble by up to a pixel (a wrapped domain warp).
  const x = x0 + (fbm(x0, y0, 8, 2, 27) - 0.5) * 2;
  const y = y0 + (fbm(x0, y0, 8, 2, 28) - 0.5) * 2;
  const threads = SIZE / THREAD;
  const i = wrap(Math.floor(x / THREAD), threads);
  const j = wrap(Math.floor(y / THREAD), threads);
  const u = wrap(x, THREAD) / THREAD;
  const v = wrap(y, THREAD) / THREAD;
  // The weft (horizontal threads) over the warp where (i + j) is even.
  const warpOnTop = (i + j) % 2 === 1;
  const warpShade = 0.8 + hash(i, 0, 21) * 0.2 + (valueNoise(i * THREAD, y0, 16, 23 + i) - 0.5) * 0.3;
  const weftShade = 0.8 + hash(0, j, 22) * 0.2 + (valueNoise(x0, j * THREAD, 16, 24 + j) - 0.5) * 0.3;
  // Across a thread its round profile; along it the rise over the crossing.
  const top = warpOnTop
    ? Math.sin(u * Math.PI) ** 0.6 * (0.55 + 0.45 * Math.sin(v * Math.PI)) * warpShade
    : Math.sin(v * Math.PI) ** 0.6 * (0.55 + 0.45 * Math.sin(u * Math.PI)) * weftShade;
  const tooth = fbm(x0, y0, 16, 3, 25);
  return 160 + top * 72 + (tooth - 0.5) * 36 + (hash(x0, y0, 26) - 0.5) * 10;
});

// Cold-pressed watercolour paper: soft bumps a dozen pixels across, shallow
// pits between them, a fine tooth.
const watercolor = image((x, y) => {
  const bumps = fbm(x, y, 16, 3, 31);
  const pits = Math.max(0, 0.42 - fbm(x, y, 24, 2, 32)) * 2.2;
  const tooth = hash(x, y, 33);
  return 238 + (bumps - 0.5) * 34 - pits * 40 + (tooth - 0.5) * 8;
});

const palettes: Record<string, { about: string; colors: string[] }> = {
  lcd2: {
    about: "Two greens of an early monochrome LCD: the dark ink and the lit glass.",
    colors: ["1f3320", "a9c27a"],
  },
  handheld4: {
    about: "Four greens of a green-screen handheld, darkest to lightest.",
    colors: ["14261b", "3d6234", "84a440", "c9da8f"],
  },
  dusk16: {
    about: "Sixteen colours of a dusk: near-black through purples and oranges to cream, and a cool sea ramp.",
    colors: [
      "100c18", "281838", "482458", "703068", "a04068", "c85860", "e88058", "f8b060",
      "fcd888", "fff4c8", "304860", "407080", "60a098", "98d0b0", "d0f0d0", "ffffff",
    ],
  },
  bright32: {
    about: "Thirty-two bright, slightly warm colours for 16-bit-era pixel art: a grey ramp, skin, earth, foliage, water, fire and accents.",
    colors: [
      // greys from ink to paper
      "16141f", "2c2a3a", "4a4858", "6e6c7a", "9896a0", "c4c2c8", "efedf0",
      // skin and earth
      "5c3424", "8e5230", "c47c48", "e8b07c", "f8dcb4",
      // foliage
      "1e3a2c", "2f6038", "4c9a3c", "8cd048", "d4f070",
      // water and sky
      "1c2a5c", "2a4c98", "3c80d0", "6cbcf0", "b4e8fc",
      // fire and warmth
      "6c1c2c", "b42c30", "e8582c", "f89c34", "fcd84c",
      // accents
      "3c1c58", "7c34a0", "d04ca8", "f490c4", "48d0b0",
    ],
  },
  "1bit": {
    about: "One bit: black and white.",
    colors: ["000000", "ffffff"],
  },
};

mkdirSync(join(here, "canvas"), { recursive: true });
mkdirSync(join(here, "palettes"), { recursive: true });
for (const [name, pixels] of [["paper", paper], ["linen", linen], ["watercolor", watercolor]] as const) {
  const bytes = png(pixels);
  writeFileSync(join(here, "canvas", `${name}.png`), bytes);
  console.log(`canvas/${name}.png ${bytes.length} B`);
}
for (const [name, p] of Object.entries(palettes)) {
  const text = `; ${p.about}\n; The engine's built-in palette @palettes/${name} (templates/looks/assets/generate.ts), CC0.\n${p.colors.join("\n")}\n`;
  writeFileSync(join(here, "palettes", `${name}.hex`), text);
  console.log(`palettes/${name}.hex ${p.colors.length} colours`);
}
