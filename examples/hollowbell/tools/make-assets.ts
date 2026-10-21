// Hollowbell's asset generator: every source asset of the game is drawn or
// synthesised here, deterministically (seeded, no clock, no download).
//
//   bun tools/make-assets.ts            everything, the music included
//   bun tools/make-assets.ts --no-music everything but assets/music/hollow.wav
//
// What it writes under assets/:
// - palettes/hollowbell.hex: the game's 32 colours. Every pixel below is
//   drawn from them, and the look quantises the lit frame back to them
//   (Stylise.palette, looks/hollowbell.ogt).
// - tiles/hollow.png + hollow.ogt: the 16 px tileset (the cave rock's 47
//   blob shapes as the auto-tiled terrain `rock`, the back wall, planks,
//   an animated torch, crystals, grass, the far hills, a fence).
// - maps/hollow.tmj: the Tiled map (96 x 48 cells, one cell one world unit),
//   its layers (far hills with parallax, the cave's back wall, the rock
//   terrain with collision, the planks, the decor) and its objects (the
//   spawn, the bell, the chimes, glimmers, hearts, the enemies, the mole,
//   the checkpoint lanterns, the torches' lights, light occluders, one-way
//   planks, the lift, the pit, the arena, the areas).
// - sprites/*.png + *.ogt: flipbook sheets (Wren, the beetle, the moth, the
//   mole, the bell) and sprites/items/*.png, the layers of the texture
//   array sprites/items.ogt (glimmer, chime, heart, lantern frames).
// - fx/*.png and fx/motes/*.png (a texture array the dust emitter draws
//   from), fx/*.ogt particle emitters, sprites/sky.png.
// - audio/*.ogt: og sfx presets (cooked into sounds by og cook) and
//   audio/bell.wav, a synthesised bell toll.
// - music/hollow.wav: "Under the Hill", 32 bars at 90 bpm (about 85 s,
//   44.1 kHz stereo), cooked in chunks and streamed (music/hollow.wav.ogt).
//   About 15 MB, so it is not committed (.gitignore).
//
// The map's numbers are the game's: src/world.ts reads every position from
// the map's objects, never from here.

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

const clamp = (v: number, a: number, b: number) => Math.max(a, Math.min(b, v));

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

function hashn(...v: number[]): number {
  let h = 2166136261;
  for (const x of v) h = Math.imul(h ^ (x >>> 0), 16777619) >>> 0;
  return h >>> 0;
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

/** An RGBA8 PNG. */
function png(w: number, h: number, px: Uint8Array): Uint8Array {
  const raw = new Uint8Array(h * (w * 4 + 1));
  for (let y = 0; y < h; y++) {
    raw[y * (w * 4 + 1)] = 0;
    raw.set(px.subarray(y * w * 4, (y + 1) * w * 4), y * (w * 4 + 1) + 1);
  }
  const ihdr = new Uint8Array(13);
  const dv = new DataView(ihdr.buffer);
  dv.setUint32(0, w);
  dv.setUint32(4, h);
  ihdr[8] = 8;
  ihdr[9] = 6;
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

// ------------------------------------------------------------------ palette

/** The 32 colours, by name. Every pixel drawn below is one of them. */
const P = {
  ink: "#0b0a12",
  night: "#1a1726",
  shade: "#262138",
  rock: "#352d4d",
  rockHi: "#463b63",
  stone: "#5b4f7d",
  lilac: "#7d73a3",
  bark: "#4a3a30",
  wood: "#6e5643",
  tan: "#9a7d61",
  sand: "#c9ab88",
  mossDk: "#1e3d2f",
  moss: "#2f6140",
  green: "#4f9150",
  leaf: "#8cc65e",
  deep: "#1f3466",
  blue: "#2c5c99",
  sky: "#4b92c9",
  ice: "#8fcbe8",
  pale: "#d5f0f5",
  bronzeDk: "#6b4520",
  bronze: "#b07a2a",
  gold: "#e0ae3c",
  sun: "#f7dc74",
  cream: "#fff4c4",
  wine: "#6e1f30",
  red: "#b73c3c",
  coral: "#ec7a5c",
  skin: "#f2b48f",
  plum: "#8f4f9c",
  pink: "#d68fd0",
  white: "#f6f3e4",
} as const;
type Colour = keyof typeof P;

function rgb(c: Colour): [number, number, number] {
  const s = P[c].slice(1);
  return [parseInt(s.slice(0, 2), 16), parseInt(s.slice(2, 4), 16), parseInt(s.slice(4, 6), 16)];
}

function palette(): void {
  write("palettes/hollowbell.hex", Object.values(P).map((h) => h.slice(1)).join("\n") + "\n");
}

// --------------------------------------------------------------- pixel art

/** A small RGBA canvas drawn with the palette's colours. */
class Pix {
  readonly px: Uint8Array;
  constructor(readonly w: number, readonly h: number) {
    this.px = new Uint8Array(w * h * 4);
  }
  set(x: number, y: number, c: Colour | null, alpha = 255): void {
    x = Math.round(x);
    y = Math.round(y);
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return;
    const i = (y * this.w + x) * 4;
    if (c === null) {
      this.px.fill(0, i, i + 4);
      return;
    }
    const [r, g, b] = rgb(c);
    this.px[i] = r;
    this.px[i + 1] = g;
    this.px[i + 2] = b;
    this.px[i + 3] = alpha;
  }
  get(x: number, y: number): number {
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return 0;
    return this.px[(y * this.w + x) * 4 + 3]!;
  }
  rect(x: number, y: number, w: number, h: number, c: Colour): void {
    for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) this.set(x + i, y + j, c);
  }
  /** A filled ellipse centred at (cx, cy) with radii rx, ry. */
  oval(cx: number, cy: number, rx: number, ry: number, c: Colour): void {
    for (let y = Math.floor(cy - ry); y <= Math.ceil(cy + ry); y++) {
      for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x++) {
        const dx = (x - cx) / rx;
        const dy = (y - cy) / ry;
        if (dx * dx + dy * dy <= 1.0) this.set(x, y, c);
      }
    }
  }
  line(x0: number, y0: number, x1: number, y1: number, c: Colour): void {
    const n = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0), 1);
    for (let i = 0; i <= n; i++) this.set(x0 + ((x1 - x0) * i) / n, y0 + ((y1 - y0) * i) / n, c);
  }
  /** A one-pixel outline in `c` round every opaque pixel (4-neighbours). */
  outline(c: Colour = "ink"): void {
    const solid = new Uint8Array(this.w * this.h);
    for (let y = 0; y < this.h; y++) for (let x = 0; x < this.w; x++) solid[y * this.w + x] = this.get(x, y) > 0 ? 1 : 0;
    for (let y = 0; y < this.h; y++) {
      for (let x = 0; x < this.w; x++) {
        if (solid[y * this.w + x]) continue;
        const n = (dx: number, dy: number) => {
          const xx = x + dx;
          const yy = y + dy;
          return xx >= 0 && yy >= 0 && xx < this.w && yy < this.h && solid[yy * this.w + xx] === 1;
        };
        if (n(1, 0) || n(-1, 0) || n(0, 1) || n(0, -1)) this.set(x, y, c);
      }
    }
  }
  /** Copy another canvas in at (ox, oy), mirrored when flip. */
  blit(o: Pix, ox: number, oy: number, flip = false): void {
    for (let y = 0; y < o.h; y++) {
      for (let x = 0; x < o.w; x++) {
        const i = (y * o.w + x) * 4;
        if (o.px[i + 3] === 0) continue;
        const tx = ox + (flip ? o.w - 1 - x : x);
        const ty = oy + y;
        if (tx < 0 || ty < 0 || tx >= this.w || ty >= this.h) continue;
        const j = (ty * this.w + tx) * 4;
        this.px.set(o.px.subarray(i, i + 4), j);
      }
    }
  }
  png(): Uint8Array {
    return png(this.w, this.h, this.px);
  }
}

/** A sheet of equal cells, `cols` across. */
function sheet(frames: Pix[], cols: number): Pix {
  const fw = frames[0]!.w;
  const fh = frames[0]!.h;
  const rows = Math.ceil(frames.length / cols);
  const s = new Pix(fw * cols, fh * rows);
  frames.forEach((f, i) => s.blit(f, (i % cols) * fw, Math.floor(i / cols) * fh));
  return s;
}

const PIXEL_IMPORT = "type import\nversion 1\n---\n# Pixel art: lossless, no mips, sampled nearest.\ntexture {\n  kind pixel\n}\n";

function writePixel(rel: string, p: Pix): void {
  write(rel, p.png());
  write(`${rel}.ogt`, PIXEL_IMPORT);
}

// ------------------------------------------------------------------ tileset

const T = 16;
const N = 1, NE = 2, E = 4, SE = 8, S = 16, SW = 32, W = 64, NW = 128;

/** A corner bit counts only when both of its edge bits are set (the blob47 rule the tileset's terrain uses). */
function reduceBlob(m: number): number {
  let out = m & (N | E | S | W);
  for (const [corner, a, b] of [[NE, N, E], [SE, S, E], [SW, S, W], [NW, N, W]] as const) {
    if (m & corner && m & a && m & b) out |= corner;
  }
  return out;
}

/** The 47 blob shapes; a shape's tile id is its index here. */
const BLOB = [...new Set(Array.from({ length: 256 }, (_, m) => reduceBlob(m)))].sort((a, b) => a - b);
if (BLOB.length !== 47) throw new Error("blob47 needs 47 shapes");

const TILE = {
  back: [47, 48, 49, 68],
  plank: { left: 50, mid: 51, right: 52 },
  torch: [53, 54, 55],
  crystal: [56, 57, 58],
  grass: [59, 60, 61],
  hill: { fill: 62, top: 63, bump: 64, pine: 65 },
  fence: [66, 67],
  thorns: 69,
} as const;
const TILE_COLS = 8;
const TILE_ROWS = 9;

function rockTile(m: number): Pix {
  const t = new Pix(T, T);
  t.rect(0, 0, T, T, "rock");
  for (let i = 0; i < 9; i++) {
    const h = hashn(m, i, 7);
    t.set(h % T, (h >>> 8) % T, i % 3 === 0 ? "shade" : "rockHi");
  }
  // A stratum line or two.
  const ly = 5 + (hashn(m, 99) % 6);
  for (let x = 0; x < T; x++) if (hashn(m, x, ly) % 5 !== 0) t.set(x, ly, "shade");
  if (!(m & W)) t.rect(0, 0, 2, T, "rockHi");
  if (!(m & E)) t.rect(T - 2, 0, 2, T, "shade");
  if (!(m & S)) {
    t.rect(0, T - 3, T, 3, "shade");
    t.rect(0, T - 1, T, 1, "night");
  }
  if (!(m & N)) {
    // Moss on every top face: a bright lip, the green, roots dangling.
    t.rect(0, 0, T, 4, "moss");
    t.rect(0, 0, T, 2, "green");
    for (let x = 0; x < T; x++) {
      if (hashn(m, x, 1) % 3 === 0) t.set(x, 0, "leaf");
      if (hashn(m, x, 2) % 4 === 0) t.set(x, 4, "mossDk");
      if (hashn(m, x, 3) % 7 === 0) t.set(x, 5, "mossDk");
    }
  }
  // Inner corners: both edges filled, the diagonal open.
  if (m & N && m & W && !(m & NW)) t.rect(0, 0, 2, 2, "rockHi");
  if (m & N && m & E && !(m & NE)) t.rect(T - 2, 0, 2, 2, "shade");
  if (m & S && m & W && !(m & SW)) t.rect(0, T - 2, 2, 2, "shade");
  if (m & S && m & E && !(m & SE)) t.rect(T - 2, T - 2, 2, 2, "night");
  return t;
}

function backTile(v: number): Pix {
  const t = new Pix(T, T);
  t.rect(0, 0, T, T, "night");
  // Low-contrast blocks of the cave's far wall.
  const off = v % 2 === 0 ? 0 : 8;
  for (let y = 0; y < T; y += 8) {
    t.rect(0, y + 7, T, 1, "ink");
    for (let x = (off + (y === 0 ? 0 : 4)) % 8; x < T; x += 8) t.rect(x, y, 1, 7, "ink");
  }
  for (let i = 0; i < 6; i++) {
    const h = hashn(v, i, 31);
    t.set(h % T, (h >>> 8) % T, "shade");
  }
  if (v === 3) {
    // A root hanging over the wall.
    for (let y = 0; y < 12; y++) t.set(7 + ((y >> 2) % 2), y, "bark");
    t.set(6, 12, "bark");
  }
  return t;
}

function plankTile(kind: "left" | "mid" | "right"): Pix {
  const t = new Pix(T, T);
  t.rect(0, 0, T, 4, "wood");
  t.rect(0, 0, T, 1, "tan");
  t.rect(0, 3, T, 1, "bark");
  t.set(4, 1, "bark");
  t.set(11, 1, "bark");
  if (kind === "left") {
    t.rect(1, 4, 2, 7, "bark");
    t.line(3, 4, 6, 7, "bark");
  }
  if (kind === "right") {
    t.rect(13, 4, 2, 7, "bark");
    t.line(12, 4, 9, 7, "bark");
  }
  return t;
}

function torchTile(f: number): Pix {
  const t = new Pix(T, T);
  t.rect(7, 9, 2, 6, "wood");
  t.rect(5, 13, 6, 2, "bark");
  t.rect(6, 8, 4, 2, "bronzeDk");
  const h = [6, 7, 5][f]!;
  const sway = [0, 1, -1][f]!;
  for (let y = 0; y < h; y++) {
    const w = Math.max(1, Math.round((h - y) * 0.6));
    for (let x = -w; x <= w; x++) t.set(8 + x + (y > h / 2 ? sway : 0), 8 - y, y < 2 ? "coral" : Math.abs(x) < w - 1 ? "cream" : "sun");
  }
  t.set(8 + sway, 8 - h, "coral");
  return t;
}

function crystalTile(v: number): Pix {
  const t = new Pix(T, T);
  const shards: [number, number, number][] = v === 0 ? [[6, 15, 6], [10, 15, 4]] : v === 1 ? [[8, 15, 8], [4, 15, 3], [12, 15, 4]] : [[5, 15, 5], [9, 15, 7]];
  for (const [x, base, h] of shards) {
    for (let y = 0; y < h; y++) {
      const w = y < h - 2 ? 1 : 0;
      for (let dx = -w; dx <= w; dx++) t.set(x + dx, base - y, dx < 0 ? "pale" : dx > 0 ? "sky" : "ice");
    }
  }
  return t;
}

function grassTile(v: number): Pix {
  const t = new Pix(T, T);
  for (let i = 0; i < 7; i++) {
    const h = hashn(v, i, 5);
    const x = 1 + (h % 14);
    const len = 2 + ((h >>> 5) % 4);
    for (let y = 0; y < len; y++) t.set(x + (y === len - 1 ? (h >>> 9) % 2 : 0), 15 - y, y === len - 1 ? "leaf" : "green");
  }
  if (v === 2) {
    t.set(5, 11, "pink");
    t.set(4, 12, "pink");
    t.set(6, 12, "pink");
    t.set(5, 12, "sun");
  }
  return t;
}

function hillTile(kind: "fill" | "top" | "bump" | "pine"): Pix {
  const t = new Pix(T, T);
  if (kind === "fill") {
    t.rect(0, 0, T, T, "moss");
    for (let i = 0; i < 5; i++) {
      const h = hashn(i, 77);
      t.set(h % T, (h >>> 8) % T, "mossDk");
    }
  } else if (kind === "top" || kind === "bump") {
    for (let x = 0; x < T; x++) {
      const top = kind === "top" ? 8 : 8 - Math.round(Math.sin((x / (T - 1)) * Math.PI) * 5);
      for (let y = top; y < T; y++) t.set(x, y, y === top ? "green" : "moss");
    }
  } else {
    for (let y = 2; y < 15; y++) {
      const w = Math.floor((y - 1) / 2.6);
      for (let x = -w; x <= w; x++) t.set(8 + x, y, x < 0 ? "mossDk" : "moss");
    }
    t.rect(7, 14, 2, 2, "bark");
  }
  return t;
}

function fenceTile(v: number): Pix {
  const t = new Pix(T, T);
  for (const x of v === 0 ? [2, 10] : [6, 13]) t.rect(x, 5, 2, 11, "wood");
  t.rect(0, 7, T, 2, "tan");
  t.rect(0, 12, T, 2, "wood");
  return t;
}

function thornTile(): Pix {
  const t = new Pix(T, T);
  for (let i = 0; i < 6; i++) {
    const x = 1 + i * 3;
    const h = 5 + (hashn(i, 3) % 6);
    t.line(x, 15, x + 1, 15 - h, "wine");
    t.set(x + 1, 15 - h, "coral");
  }
  t.rect(0, 14, T, 2, "plum");
  return t;
}

function tileset(): void {
  const tiles = new Map<number, Pix>();
  BLOB.forEach((m, i) => tiles.set(i, rockTile(m)));
  TILE.back.forEach((id, v) => tiles.set(id, backTile(v)));
  tiles.set(TILE.plank.left, plankTile("left"));
  tiles.set(TILE.plank.mid, plankTile("mid"));
  tiles.set(TILE.plank.right, plankTile("right"));
  TILE.torch.forEach((id, f) => tiles.set(id, torchTile(f)));
  TILE.crystal.forEach((id, v) => tiles.set(id, crystalTile(v)));
  TILE.grass.forEach((id, v) => tiles.set(id, grassTile(v)));
  tiles.set(TILE.hill.fill, hillTile("fill"));
  tiles.set(TILE.hill.top, hillTile("top"));
  tiles.set(TILE.hill.bump, hillTile("bump"));
  tiles.set(TILE.hill.pine, hillTile("pine"));
  TILE.fence.forEach((id, v) => tiles.set(id, fenceTile(v)));
  tiles.set(TILE.thorns, thornTile());
  const atlas = new Pix(TILE_COLS * T, TILE_ROWS * T);
  for (const [id, p] of tiles) atlas.blit(p, (id % TILE_COLS) * T, Math.floor(id / TILE_COLS) * T);
  writePixel("tiles/hollow.png", atlas);
  const lines = [
    "type tileset",
    "version 1",
    "---",
    "# Written by tools/make-assets.ts: tiles 0-46 are the rock terrain's 47 blob",
    "# shapes, 47-49 and 68 the cave's back wall, 50-52 a plank, 53-55 the torch,",
    "# 56-58 crystals, 59-61 grass, 62-65 the far hills, 66-67 a fence, 69 thorns.",
    'texture "tiles/hollow.png"',
    `grid ${T} ${T} ${TILE_COLS} ${TILE_ROWS}`,
    "filter nearest",
    "solid " + BLOB.map((_, i) => i).join(" "),
    `tile ${TILE.torch[0]} {`,
    `  frames ${TILE.torch[0]} ${TILE.torch[1]} ${TILE.torch[0]} ${TILE.torch[2]}`,
    "  fps 8",
    "}",
    "terrain rock {",
    "  mode blob47",
    ...BLOB.map((m, i) => `  rule ${m} ${i}`),
    `  default ${BLOB.indexOf(255)}`,
    "  edge filled",
    "}",
  ];
  write("tiles/hollow.ogt", lines.join("\n") + "\n");
}

// ---------------------------------------------------------------------- map

const MW = 96;
const MH = 48;

/** Plank runs: [x0, x1, row]. Drawn on the `planks` layer; each is a `oneway` object the game gives a one-way collider. */
const PLANKS: [number, number, number][] = [
  // The well (down only: its bottom opens high over the upper cave).
  [22, 23, 15],
  [25, 26, 18],
  // The upper cave, up to the first chime: three rows apart, a full jump each.
  [50, 53, 29],
  [54, 58, 26],
  // The gloam's arena: out of reach of its charge.
  [43, 46, 42],
  [53, 56, 42],
  // The root climb back to the surface, alternating sides every three rows.
  ...[42, 39, 36, 33, 30, 27, 24, 21, 18, 15].map((row, i): [number, number, number] => (i % 2 === 0 ? [2, 4, row] : [5, 7, row])),
];

/** Rock that blocks the 2D light: [x, row, w, h] (stalactites, a pillar, the ledge, a mound). */
const OCCLUDERS: [number, number, number, number][] = [
  [30, 22, 1, 3],
  [46, 22, 2, 4],
  [58, 22, 1, 2],
  [19, 22, 1, 5],
  [34, 29, 4, 3],
  [36, 35, 1, 3],
  [70, 35, 2, 4],
  [62, 43, 3, 2],
];

const TORCHES: [number, number][] = [[14, 26], [38, 25], [64, 26], [74, 25], [12, 38], [20, 39], [50, 37], [78, 39], [88, 40]];
const CRYSTALS: [number, number][] = [[25, 31], [55, 31], [33, 44], [60, 44], [86, 44], [4, 30], [6, 20]];

const GLIMMERS: [number, number][] = [
  [24, 14.5], [24, 17.5], [24, 20.5],
  [13, 30.5], [15, 30.5], [17, 30.5], [33, 30.5], [35.5, 28.3], [36.5, 28.3], [40, 30.5], [44, 30.5],
  [51, 28.3], [52.5, 28.3], [55.5, 25.3], [57.5, 25.3], [62, 30.5], [68, 30.5], [71, 30.5],
  [88, 43.5], [84, 43.5], [76, 43.5], [70, 43.5], [63.5, 42.3], [58, 43.5], [42, 43.5], [44.5, 41.3], [54.5, 41.3],
  [35, 43.5], [27, 42.2], [29, 42.2], [18, 43.5], [11, 41.3], [14, 41.3],
  [6, 38.3], [6, 32.3], [6, 26.3], [6, 20.3], [6, 14.3],
];

function mapLayout() {
  const rock: number[][] = Array.from({ length: MH }, () => new Array<number>(MW).fill(1));
  const carve = (x0: number, y0: number, x1: number, y1: number, v = 0) => {
    for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) rock[y]![x] = v;
  };
  const fill = (x0: number, y0: number, x1: number, y1: number) => carve(x0, y0, x1, y1, 1);
  carve(0, 0, MW - 1, 11); // the sky over the village
  carve(2, 12, 7, 34); // the root climb
  carve(22, 12, 26, 21); // the well
  carve(10, 22, 88, 31); // the upper cave
  fill(34, 29, 37, 31); // a ledge
  fill(30, 22, 30, 24); // stalactites
  fill(46, 22, 47, 25);
  fill(58, 22, 58, 23);
  fill(19, 22, 19, 26); // a hanging pillar of rock: Wren walks under it
  carve(82, 32, 85, 34); // the drop to the lower cave
  carve(2, 35, 93, 44); // the lower cave
  carve(22, 45, 32, 46); // the pit
  fill(10, 42, 15, 44); // the second chime's ledge
  fill(36, 35, 36, 37);
  fill(70, 35, 71, 38);
  fill(62, 43, 64, 44); // a mound

  const empty = () => Array.from({ length: MH }, () => new Array<number>(MW).fill(-1));
  const back = empty();
  for (let y = 12; y < MH; y++) for (let x = 0; x < MW; x++) back[y]![x] = TILE.back[hashn(x, y, 3) % 11 === 0 ? 3 : hashn(x, y, 4) % 3]!;
  const planks = empty();
  for (const [x0, x1, row] of PLANKS) for (let x = x0; x <= x1; x++) planks[row]![x] = x === x0 ? TILE.plank.left : x === x1 ? TILE.plank.right : TILE.plank.mid;
  const decor = empty();
  for (const [x, y] of TORCHES) decor[y]![x] = TILE.torch[0];
  CRYSTALS.forEach(([x, y], i) => (decor[y]![x] = TILE.crystal[i % 3]!));
  for (let x = 10; x < MW - 2; x++) if (rock[11]![x] === 0 && hashn(x, 11) % 3 === 0) decor[11]![x] = TILE.grass[hashn(x, 12) % 3]!;
  for (let x = 17; x <= 20; x++) decor[11]![x] = TILE.fence[x % 2]!;
  for (let x = 22; x <= 32; x++) decor[46]![x] = TILE.thorns;
  // The far hills: a ridge line from a seeded walk, pines on some tops.
  const far = empty();
  const r = rng(48);
  let h = 7;
  for (let x = 0; x < MW; x++) {
    h = clamp(h + (r() < 0.33 ? -1 : r() < 0.5 ? 1 : 0), 7, 10);
    far[h]![x] = r() < 0.3 ? TILE.hill.bump : TILE.hill.top;
    for (let y = h + 1; y < 12; y++) far[y]![x] = TILE.hill.fill;
    if (r() < 0.18 && h > 7) far[h - 1]![x] = TILE.hill.pine;
  }
  return { rock, back, planks, decor, far };
}

type Prop = { name: string; type: string; value: string | number | boolean };
const prop = (name: string, type: string, value: string | number | boolean): Prop => ({ name, type, value });

function tiledMap(): object {
  const { rock, back, planks, decor, far } = mapLayout();
  const gid = (t: number) => (t < 0 ? 0 : t + 1); // firstgid 1
  let layerId = 0;
  const tileLayer = (name: string, cells: number[][], props: Prop[], extra: object = {}) => ({
    id: ++layerId, name, type: "tilelayer", x: 0, y: 0, width: MW, height: MH, opacity: 1, visible: true,
    data: cells.flat().map(gid), properties: props, ...extra,
  });
  let objectId = 0;
  const objects: object[] = [];
  const obj = (name: string, type: string, x: number, y: number, w = 0, h = 0, props: Prop[] = []) => {
    const o: Record<string, unknown> = { id: ++objectId, name, type, x: x * T, y: y * T, width: w * T, height: h * T, rotation: 0, visible: true };
    if (w === 0 && h === 0) o.point = true;
    if (props.length > 0) o.properties = props;
    objects.push(o);
  };
  // Points stand things on their feet: x is the middle of a cell, y the top of the floor's row.
  obj("spawn", "spawn", 16.5, 12);
  obj("bell", "bell", 11.5, 12);
  obj("chime-0", "chime", 56.5, 25.2);
  obj("chime-1", "chime", 12.5, 40.5);
  obj("chime-2", "chime", 50.5, 43.5, 0, 0, [prop("boss", "bool", true)]);
  GLIMMERS.forEach(([x, y], i) => obj(`glimmer-${i}`, "glimmer", x, y));
  obj("heart-0", "heart", 54.5, 25.3);
  obj("heart-1", "heart", 63.5, 42.3);
  obj("beetle-0", "beetle", 42.5, 32, 0, 0, [prop("range", "float", 4)]);
  obj("beetle-1", "beetle", 66.5, 32, 0, 0, [prop("range", "float", 4)]);
  obj("beetle-2", "beetle", 80.5, 45, 0, 0, [prop("range", "float", 5)]);
  obj("moth-0", "moth", 74.5, 39.5);
  obj("moth-1", "moth", 64.5, 37.5);
  obj("moth-2", "moth", 18.5, 38.5);
  obj("gloam", "gloam", 52.5, 45);
  obj("mole", "mole", 76.5, 32);
  obj("lantern-0", "lantern", 27.5, 32);
  obj("lantern-1", "lantern", 87.5, 45);
  obj("lantern-2", "lantern", 4.5, 45);
  TORCHES.forEach(([x, y], i) => obj(`torch-${i}`, "torch", x + 0.5, y + 0.4));
  CRYSTALS.forEach(([x, y], i) => obj(`crystal-${i}`, "crystal", x + 0.5, y + 0.6));
  OCCLUDERS.forEach(([x, y, w, h], i) => obj(`occluder-${i}`, "occluder", x, y, w, h));
  PLANKS.forEach(([x0, x1, row], i) => obj(`plank-${i}`, "oneway", x0, row, x1 - x0 + 1, 1));
  obj("lift", "lift", 23, 43, 3, 1, [prop("travel", "float", 6), prop("speed", "float", 2)]);
  obj("stone-0", "stone", 49.5, 32);
  obj("stone-1", "stone", 50.4, 32);
  obj("stone-2", "stone", 49.95, 31.2);
  obj("pit", "hazard", 22, 46, 11, 1);
  obj("arena", "arena", 40, 35, 20, 10);
  // Areas, in priority order (the first that holds the hero names it): the name's string key and the 2D ambient light.
  const area = (name: string, x: number, y: number, w: number, h: number, ambient: number) =>
    obj(`area-${name}`, "area", x, y, w, h, [prop("key", "string", `area.${name}`), prop("ambient", "float", ambient)]);
  area("surface", 0, 0, MW, 12.5, 1);
  area("arena", 40, 35, 20, 10, 0.1);
  area("nook", 72, 22, 17, 10, 0.3);
  area("well", 20, 12.5, 8, 9.5, 0.55);
  area("climb", 2, 12.5, 6, 22.5, 0.35);
  area("upper", 10, 22, 62, 10, 0.2);
  area("lower", 2, 32, 92, 15, 0.14);
  return {
    compressionlevel: -1, height: MH, width: MW, infinite: false,
    orientation: "orthogonal", renderorder: "right-down",
    tiledversion: "1.10.2", type: "map", version: "1.10",
    tilewidth: T, tileheight: T, nextlayerid: 7, nextobjectid: objectId + 1,
    tilesets: [{ firstgid: 1, source: "../tiles/hollow.tsx" }],
    layers: [
      // Tiled's own parallax factors on the far hills.
      tileLayer("far", far, [prop("repeat", "string", "x")], { parallaxx: 0.4, parallaxy: 1 }),
      tileLayer("back", back, []),
      tileLayer("rock", rock.map((row) => row.map((v) => (v ? BLOB.indexOf(255) : -1))), [prop("autotile", "string", "rock"), prop("collision", "bool", true)]),
      tileLayer("planks", planks, []),
      tileLayer("decor", decor, []),
      { id: ++layerId, name: "objects", type: "objectgroup", x: 0, y: 0, opacity: 1, visible: true, draworder: "topdown", objects },
    ],
  };
}

// ------------------------------------------------------------------ sprites

type HeroPose = {
  bob?: number; // body down by this
  front?: [number, number]; // front foot x offset, lift
  back?: [number, number];
  mallet?: "rest" | "up" | "out" | "down" | "none";
  arc?: number; // the swing's smear: 0 none, 1 full, 0.5 fading
  hurt?: boolean;
  lean?: number; // the whole body shifted along x
  tuck?: boolean; // knees up (a jump)
};

/** Wren, the bell-keeper: a gold hood with a little bell on its tip, a blue cloak, a bronze mallet. 24x24, facing right, feet on the bottom row. */
function heroFrame(p: HeroPose): Pix {
  const f = new Pix(24, 24);
  const b = p.bob ?? 0;
  const lx = p.lean ?? 0;
  // Legs and boots.
  const leg = (x: number, [dx, lift]: [number, number]) => {
    const top = 19 + b;
    const foot = 23 - lift;
    f.rect(x + dx + lx, top, 2, Math.max(1, foot - top - 1), "rock");
    f.rect(x + dx + lx, foot - 1, 3, 2, "bark");
  };
  if (p.tuck) {
    leg(9, [0, 3]);
    leg(13, [1, 2]);
  } else {
    leg(9, p.back ?? [0, 0]);
    leg(13, p.front ?? [0, 0]);
  }
  // The cloak.
  for (let y = 12 + b; y <= 19 + b; y++) {
    const spread = Math.floor((y - 12 - b) / 3);
    f.rect(8 - spread + lx, y, 8 + spread * 2 - (y > 17 + b ? 0 : 0), 1, "blue");
    f.set(8 - spread + lx, y, "deep");
    f.set(9 - spread + lx, y, "deep");
  }
  f.rect(8 + lx, 19 + b, 9, 1, "deep");
  f.rect(13 + lx, 13 + b, 1, 4, "sky");
  // The hood and the face.
  f.oval(12 + lx, 8 + b, 4.4, 4.2, "gold");
  f.oval(11 + lx, 9 + b, 3, 3, "bronze");
  f.rect(12 + lx, 7 + b, 4, 4, "skin");
  f.set(15 + lx, 6 + b, "gold");
  f.rect(12 + lx, 5 + b, 3, 2, "gold");
  if (p.hurt) {
    f.line(13 + lx, 8 + b, 15 + lx, 8 + b, "ink");
  } else {
    f.set(14 + lx, 8 + b, "ink");
    f.set(14 + lx, 7 + b, "ink");
  }
  f.set(15 + lx, 10 + b, "coral");
  // The hood's tip and its bell.
  f.line(9 + lx, 5 + b, 6 + lx, 3 + b, "gold");
  f.oval(5 + lx, 4 + b, 1.4, 1.6, "sun");
  f.set(5 + lx, 6 + b, "bronze");
  // The mallet: a wood handle, a bronze head.
  const handle = (x0: number, y0: number, x1: number, y1: number) => f.line(x0 + lx, y0 + b, x1 + lx, y1 + b, "wood");
  const head = (x: number, y: number, w: number, h: number) => {
    f.rect(x + lx, y + b, w, h, "bronze");
    f.rect(x + lx, y + b, w, 1, "sun");
  };
  const hand = (x: number, y: number) => f.set(x + lx, y + b, "skin");
  switch (p.mallet ?? "rest") {
    case "rest":
      handle(16, 13, 17, 18);
      head(16, 18, 3, 2);
      hand(16, 14);
      break;
    case "up":
      handle(11, 11, 7, 4);
      head(4, 1, 3, 4);
      hand(11, 11);
      break;
    case "out":
      handle(15, 12, 20, 12);
      head(20, 9, 3, 6);
      hand(15, 12);
      break;
    case "down":
      handle(15, 14, 19, 18);
      head(18, 18, 4, 3);
      hand(15, 14);
      break;
    case "none":
      break;
  }
  f.outline("ink");
  // The swing's smear goes over the outline: a quarter circle in front.
  const arc = p.arc ?? 0;
  if (arc > 0) {
    for (let a = -70; a <= 40; a += 4) {
      const rad = (a * Math.PI) / 180;
      const rr = 10.5;
      const x = 12 + lx + Math.cos(rad) * rr;
      const y = 12 + b + Math.sin(rad) * rr;
      if (arc < 1 && a < -20) continue;
      f.set(x, y, arc >= 1 ? "cream" : "pale");
      if (arc >= 1) f.set(x - Math.cos(rad), y - Math.sin(rad), "sun");
    }
  }
  return f;
}

/** The hero's sheet: idle 0-1, run 2-7, jump 8, fall 9, attack 10-13, hurt 14-15. */
function hero(): void {
  const run: [number, number][][] = [
    [[2, 0], [-2, 0]],
    [[1, 1], [-1, 0]],
    [[0, 0], [0, 1]],
    [[-2, 0], [2, 0]],
    [[-1, 0], [1, 1]],
    [[0, 1], [0, 0]],
  ];
  const frames = [
    heroFrame({}),
    heroFrame({ bob: 1 }),
    ...run.map(([front, back], i) => heroFrame({ front, back, bob: i % 3 === 1 ? 1 : 0 })),
    heroFrame({ tuck: true, mallet: "rest" }),
    heroFrame({ front: [1, 0], back: [-1, 0], mallet: "up" }),
    heroFrame({ mallet: "up", lean: -1 }),
    heroFrame({ mallet: "out", arc: 1, lean: 1 }),
    heroFrame({ mallet: "down", arc: 0.5, lean: 1 }),
    heroFrame({ mallet: "rest" }),
    heroFrame({ hurt: true, lean: -2, mallet: "none" }),
    heroFrame({ hurt: true, bob: 1, lean: -1, mallet: "rest" }),
  ];
  writePixel("sprites/wren.png", sheet(frames, 8));
  write(
    "sprites/wren.ogt",
    [
      "type flipbook",
      "version 1",
      "---",
      "# Written by tools/make-assets.ts. Wren faces right (flipX faces left); the",
      "# frames are 24x24 with the feet on the bottom row. The attack's strike is",
      "# its second frame (the `hit` event); the run's feet touch on frames 0 and 3.",
      'texture "wren.png"',
      "pivot 0.5 0",
      "grid 24 24 8 2",
      "clip idle {\n  frames 0 1\n  fps 3\n  mode pingPong\n}",
      'clip run {\n  frames 2 3 4 5 6 7\n  fps 12\n  event 0 "step"\n  event 3 "step"\n}',
      "clip jump {\n  frames 8\n  fps 1\n}",
      "clip fall {\n  frames 9\n  fps 1\n}",
      'clip attack {\n  frames 10 11 12 13\n  durations 0.05 0.07 0.08 0.08\n  mode once\n  event 1 "hit"\n}',
      "clip hurt {\n  frames 14 15\n  fps 10\n  mode once\n}",
    ].join("\n") + "\n",
  );
}

/** The beetle: a violet shell, a horn, three legs a side. 24x16, facing right. Walk 0-3, charge 4-6 (wind-up, dash), hurt 7-8. */
function beetleFrame(kind: "walk" | "wind" | "dash" | "hurt", i: number): Pix {
  const f = new Pix(24, 16);
  const low = kind === "wind" ? 1 : 0;
  const stretch = kind === "dash" ? 1 : 0;
  const cx = 11 - (kind === "wind" ? 1 : 0);
  // Legs.
  for (let k = 0; k < 3; k++) {
    const x = 6 + k * 5;
    const step = kind === "walk" ? ((i + k) % 2 === 0 ? 1 : -1) : kind === "dash" ? -2 : 0;
    f.line(x, 12, x + step, 15, "ink");
    f.set(x + step + 1, 15, "ink");
  }
  // The shell.
  f.oval(cx, 9 + low, 8 + stretch, 5 - stretch * 0.5, kind === "hurt" ? "lilac" : "stone");
  f.oval(cx - 1, 8 + low, 6, 3, kind === "hurt" ? "pale" : "rockHi");
  f.line(cx - 6, 8 + low, cx + 4, 8 + low, "rock");
  f.set(cx - 3, 6 + low, "lilac");
  f.set(cx - 2, 6 + low, "lilac");
  // Head, eye and horn.
  f.oval(cx + 8 + stretch, 10 + low, 3, 2.6, "shade");
  f.set(cx + 9 + stretch, 9 + low, kind === "wind" || kind === "dash" ? "red" : "coral");
  f.line(cx + 10 + stretch, 8 + low, cx + 12 + stretch, 5 + low, "bronze");
  f.set(cx + 12 + stretch, 5 + low, "sun");
  f.outline("ink");
  if (kind === "dash") for (let y = 6; y < 13; y += 3) f.line(0, y + i, 2, y + i, "pale");
  if (kind === "hurt" && i === 1) {
    f.set(cx + 9, 9, "ink");
    f.set(cx + 9, 10, "ink");
  }
  return f;
}

function beetle(): void {
  const frames = [0, 1, 2, 3].map((i) => beetleFrame("walk", i));
  frames.push(beetleFrame("wind", 0), beetleFrame("dash", 0), beetleFrame("dash", 1), beetleFrame("hurt", 0), beetleFrame("hurt", 1));
  writePixel("sprites/beetle.png", sheet(frames, 8));
  write(
    "sprites/beetle.ogt",
    [
      "type flipbook",
      "version 1",
      "---",
      "# Written by tools/make-assets.ts. Faces right; 24x16, feet on the bottom row.",
      "# The charge: a held wind-up (the telegraph), then the dash frames.",
      'texture "beetle.png"',
      "pivot 0.5 0",
      "grid 24 16 8 2",
      "clip walk {\n  frames 0 1 2 3\n  fps 8\n}",
      "clip wind {\n  frames 4\n  fps 1\n}",
      "clip dash {\n  frames 5 6\n  fps 16\n}",
      "clip hurt {\n  frames 7 8\n  fps 10\n  mode once\n}",
    ].join("\n") + "\n",
  );
}

/** The moth: a plum body, pink wings. 16x16. Fly 0-3 (a flap), dive 4-5, hurt 6-7. */
function mothFrame(kind: "fly" | "dive" | "hurt", i: number): Pix {
  const f = new Pix(16, 16);
  const span = kind === "fly" ? [5, 3, 1, 3][i]! : kind === "dive" ? 1 : 4;
  const wing: Colour = kind === "hurt" ? "pale" : "pink";
  if (kind === "dive") {
    f.oval(5, 7, 4, 1.5, wing);
    f.oval(11, 7, 4, 1.5, wing);
  } else {
    f.oval(4, 8 - span / 2, 3.5, span + 0.5, wing);
    f.oval(12, 8 - span / 2, 3.5, span + 0.5, wing);
    f.set(4, 8 - span / 2, "plum");
    f.set(12, 8 - span / 2, "plum");
  }
  f.oval(8, 9, 1.6, 4, kind === "hurt" ? "lilac" : "plum");
  f.rect(7, 11, 3, 1, "wine");
  f.oval(8, 5, 2, 1.8, "pink");
  f.set(7, 5, "ink");
  f.set(9, 5, "ink");
  f.line(7, 3, 5, 1, "plum");
  f.line(9, 3, 11, 1, "plum");
  f.outline("ink");
  return f;
}

function moth(): void {
  const frames = [0, 1, 2, 3].map((i) => mothFrame("fly", i));
  frames.push(mothFrame("dive", 0), mothFrame("dive", 1), mothFrame("hurt", 0), mothFrame("hurt", 1));
  writePixel("sprites/moth.png", sheet(frames, 8));
  write(
    "sprites/moth.ogt",
    [
      "type flipbook",
      "version 1",
      "---",
      "# Written by tools/make-assets.ts. 16x16, centred.",
      'texture "moth.png"',
      "pivot 0.5 0.5",
      "grid 16 16 8 1",
      "clip fly {\n  frames 0 1 2 3\n  fps 12\n  mode pingPong\n}",
      "clip dive {\n  frames 4 5\n  fps 10\n}",
      "clip hurt {\n  frames 6 7\n  fps 10\n  mode once\n}",
    ].join("\n") + "\n",
  );
}

/** Old Burrow the mole: brown and round, a pink nose, a candle on his hat. 16x16. Idle 0-1, talk 2-3. */
function moleFrame(talk: boolean, i: number): Pix {
  const f = new Pix(16, 16);
  const b = i;
  f.oval(8, 11 + b * 0.5, 6, 4.6, "wood");
  f.oval(9, 12 + b * 0.5, 3.4, 3, "sand");
  f.oval(12, 9 + b * 0.5, 2.5, 2, "wood");
  f.set(14, 9 + b, "pink");
  f.set(15, 9 + b, "pink");
  f.set(12, 8 + b, "ink");
  if (talk) f.rect(12, 10 + b, 2, 1 + i, "wine");
  f.rect(5, 15, 3, 1, "bark");
  f.rect(10, 15, 3, 1, "bark");
  // The hat and its candle.
  f.rect(5, 5 + b, 6, 2, "bronzeDk");
  f.rect(6, 3 + b, 4, 2, "bronze");
  f.rect(7, 1 + b, 1, 2, "cream");
  f.set(7, 0 + b, i === 0 ? "sun" : "coral");
  f.outline("ink");
  return f;
}

function mole(): void {
  writePixel("sprites/mole.png", sheet([moleFrame(false, 0), moleFrame(false, 1), moleFrame(true, 0), moleFrame(true, 1)], 4));
  write(
    "sprites/mole.ogt",
    [
      "type flipbook",
      "version 1",
      "---",
      "# Written by tools/make-assets.ts. 16x16, feet on the bottom row.",
      'texture "mole.png"',
      "pivot 0.5 0",
      "grid 16 16 4 1",
      "clip idle {\n  frames 0 1\n  fps 2\n}",
      "clip talk {\n  frames 2 3\n  fps 6\n}",
    ].join("\n") + "\n",
  );
}

/** The village bell in its wooden frame: still, swung left, swung right. 32x48, standing on the bottom row. */
function bellFrame(swing: number): Pix {
  const f = new Pix(32, 48);
  f.rect(3, 9, 3, 39, "wood");
  f.rect(26, 9, 3, 39, "wood");
  f.rect(3, 9, 1, 39, "tan");
  f.rect(1, 6, 30, 4, "wood");
  f.rect(1, 6, 30, 1, "tan");
  for (let y = 1; y < 6; y++) f.rect(16 - (y + 1) * 3, y, (y + 1) * 6, 1, y === 1 ? "bronzeDk" : "bark");
  f.line(16, 10, 16 + swing, 14, "bark");
  // The bell: a dome flaring to a lip, skewed by the swing.
  for (let y = 14; y <= 31; y++) {
    const t = (y - 14) / 17;
    const half = y < 18 ? 3 + (y - 14) : 6 + t * t * 4;
    const sx = Math.round(16 + swing * (1 + (y - 14) / 8));
    for (let x = -Math.round(half); x <= Math.round(half); x++) {
      const c: Colour = y >= 30 ? "bronze" : x === -Math.round(half) + 2 ? "cream" : x > half - 2 ? "bronze" : "gold";
      f.set(sx + x, y, c);
    }
  }
  const cx = Math.round(16 + swing * 3.4);
  f.oval(cx, 33, 1.5, 1.5, "bark");
  f.outline("ink");
  return f;
}

function bell(): void {
  writePixel("sprites/bell.png", sheet([bellFrame(0), bellFrame(-2), bellFrame(2)], 3));
  write(
    "sprites/bell.ogt",
    [
      "type flipbook",
      "version 1",
      "---",
      "# Written by tools/make-assets.ts. 32x48 on its feet; `ring` swings it.",
      'texture "bell.png"',
      "pivot 0.5 0",
      "grid 32 48 3 1",
      "clip still {\n  frames 0\n  fps 1\n}",
      'clip ring {\n  frames 1 0 2 0\n  fps 6\n  event 0 "toll"\n}',
    ].join("\n") + "\n",
  );
}

// ------------------------------------------------------- texture arrays

/** The items' layers (16x16 each): glimmer 0-3, chime 4-7, heart 8, lantern dark 9, lit 10-11. */
function items(): void {
  const layers: Pix[] = [];
  for (const w of [5, 3.4, 1, 3.4]) {
    const f = new Pix(16, 16);
    f.oval(8, 8, w, 5, "gold");
    if (w > 2) f.oval(8, 8, w - 1.5, 3.5, "sun");
    f.set(8 - Math.floor(w / 2), 6, "cream");
    f.outline("bronzeDk");
    layers.push(f);
  }
  for (let i = 0; i < 4; i++) {
    const f = new Pix(16, 16);
    for (let y = 3; y <= 12; y++) {
      const half = y < 6 ? y - 2 : 3 + (y - 6) * 0.45;
      for (let x = -Math.round(half); x <= Math.round(half); x++) f.set(8 + x, y, x <= -Math.round(half) + 1 ? "cream" : y === 12 ? "bronze" : "gold");
    }
    f.line(9, 4, 7, 9, "bronzeDk");
    f.set(8, 13, "bark");
    f.outline("ink");
    const gx = [4, 11, 12, 5][i]!;
    const gy = [4, 3, 9, 11][i]!;
    f.set(gx, gy, "white");
    f.set(gx - 1, gy, "ice");
    f.set(gx + 1, gy, "ice");
    f.set(gx, gy - 1, "ice");
    f.set(gx, gy + 1, "ice");
    layers.push(f);
  }
  {
    const f = new Pix(16, 16);
    f.oval(5.5, 6, 3, 3, "red");
    f.oval(10.5, 6, 3, 3, "red");
    for (let y = 6; y < 14; y++) for (let x = 2 + (y - 6); x <= 13 - (y - 6); x++) f.set(x, y, "red");
    f.set(5, 5, "coral");
    f.set(4, 6, "coral");
    f.outline("wine");
    layers.push(f);
  }
  for (const lit of [0, 1, 2]) {
    const f = new Pix(16, 16);
    f.rect(7, 9, 2, 7, "bark");
    f.rect(4, 2, 8, 7, "bark");
    f.rect(5, 3, 6, 5, lit === 0 ? "rock" : lit === 1 ? "sun" : "cream");
    if (lit > 0) f.rect(7, 4, 2, 3, lit === 1 ? "cream" : "sun");
    f.rect(3, 1, 10, 1, "bronze");
    f.outline("ink");
    layers.push(f);
  }
  const names = ["glimmer-0", "glimmer-1", "glimmer-2", "glimmer-3", "chime-0", "chime-1", "chime-2", "chime-3", "heart", "lantern-dark", "lantern-lit-0", "lantern-lit-1"];
  layers.forEach((l, i) => write(`sprites/items/${names[i]}.png`, l.png()));
  write(
    "sprites/items.ogt",
    ["type texture-array", "version 1", "---", "# Written by tools/make-assets.ts: the items' frames, one layer each (16x16).", "kind pixel", ...names.map((n) => `layer "items/${n}.png"`)].join("\n") + "\n",
  );
}

/** Dust motes for the caves' air (8x8): a dot, a diamond, a cross. A texture array the motes emitter draws from at random. */
function motes(): void {
  const dot = new Pix(8, 8);
  dot.oval(3.5, 3.5, 2, 2, "sun");
  dot.oval(3.5, 3.5, 1, 1, "cream");
  const diamond = new Pix(8, 8);
  for (let y = 0; y < 7; y++) for (let x = 0; x < 7; x++) if (Math.abs(x - 3) + Math.abs(y - 3) <= 3) diamond.set(x, y, Math.abs(x - 3) + Math.abs(y - 3) <= 1 ? "pale" : "ice");
  const cross = new Pix(8, 8);
  cross.rect(3, 0, 1, 7, "sun");
  cross.rect(0, 3, 7, 1, "sun");
  cross.set(3, 3, "white");
  const names = ["dot", "diamond", "cross"];
  [dot, diamond, cross].forEach((p, i) => write(`fx/motes/${names[i]}.png`, p.png()));
  write("fx/motes-array.ogt", ["type texture-array", "version 1", "---", "# Written by tools/make-assets.ts: three mote shapes, one layer each (8x8).", "kind pixel", ...names.map((n) => `layer "motes/${n}.png"`)].join("\n") + "\n");
  const spark = new Pix(8, 8);
  spark.oval(3.5, 3.5, 3.2, 3.2, "sun");
  spark.oval(3.5, 3.5, 2, 2, "cream");
  spark.oval(3.5, 3.5, 1, 1, "white");
  writePixel("fx/spark.png", spark);
  // A soft round pool of light (white, its alpha falling off with the square of the distance): drawn additive and tinted
  // over the darkened map where a light shines (src/props.ts Pools; FINDINGS.md #F-tilelight).
  const pool = new Pix(64, 64);
  for (let y = 0; y < 64; y++) {
    for (let x = 0; x < 64; x++) {
      const d = Math.hypot(x - 31.5, y - 31.5) / 32;
      if (d < 1) pool.set(x, y, "white", Math.round((1 - d) * (1 - d) * 255));
    }
  }
  writePixel("fx/pool.png", pool);
  const puff = new Pix(8, 8);
  puff.oval(3.5, 3.5, 3.2, 3.2, "tan");
  puff.oval(3, 3, 1.8, 1.8, "sand");
  writePixel("fx/puff.png", puff);
}

/** The sky over the village: a gradient 1 wide and 64 tall, stretched behind the far hills (the look's dither blends its bands). */
function sky(): void {
  const s = new Pix(1, 64);
  const bands: Colour[] = ["deep", "blue", "blue", "sky", "sky", "ice", "ice", "pale", "sun", "sun"];
  for (let y = 0; y < 64; y++) s.set(0, y, bands[Math.min(bands.length - 1, Math.floor((y / 64) * bands.length))]!);
  writePixel("sprites/sky.png", s);
}

// ------------------------------------------------------------- particles

function emitters(): void {
  const e = (name: string, lines: string[]) => write(`fx/${name}.ogt`, ["type particles", "version 1", "---", ...lines].join("\n") + "\n");
  e("hit", [
    "# A strike landing: a handful of sparks thrown out of the contact point.",
    "maxParticles 24",
    "shape point",
    "spread 180",
    "rate 0",
    "burst 0 10",
    "duration 0.1",
    "looping false",
    "lifetime 0.18 0.32",
    "speed 3 6",
    "size 0.12 0.22",
    "sizeOverLife 0 1 1 0",
    "colorOverLife {\n  key 0 1 0.95 0.75 1\n  key 1 0.95 0.55 0.2 0\n}",
    "drag 4",
    'texture "fx/spark.png"',
    "blend additive",
    "render sprite 12",
  ]);
  e("dust", [
    "# A landing or a turn: a few puffs of dust along the ground.",
    "maxParticles 16",
    "shape box 0.6 0.05 0",
    "spread 70",
    "rate 0",
    "burst 0 6",
    "duration 0.1",
    "looping false",
    "lifetime 0.3 0.5",
    "speed 0.6 1.4",
    "size 0.18 0.32",
    "sizeOverLife 0 0.6 1 0",
    "colorOverLife {\n  key 0 0.58 0.48 0.38 0.9\n  key 1 0.58 0.48 0.38 0\n}",
    "gravity 0 0.6 0",
    "drag 3",
    'texture "fx/puff.png"',
    "render sprite 4",
  ]);
  e("chime", [
    "# A chime taken: a fountain of gold and ice sparks.",
    "maxParticles 48",
    "shape point",
    "spread 60",
    "rate 0",
    "burst 0 32",
    "duration 0.1",
    "looping false",
    "lifetime 0.5 1.0",
    "speed 3 6.5",
    "size 0.1 0.2",
    "colorOverLife {\n  key 0 1 0.92 0.5 1\n  key 0.5 0.6 0.85 1 0.9\n  key 1 0.5 0.7 1 0\n}",
    "gravity 0 -6 0",
    "drag 0.8",
    'texture "fx/spark.png"',
    "blend additive",
    "render sprite 14",
  ]);
  e("burst", [
    "# The gloam's fall: a big slow ring of sparks.",
    "maxParticles 96",
    "shape sphere 0.6",
    "spread 180",
    "rate 0",
    "burst 0 72",
    "duration 0.1",
    "looping false",
    "lifetime 0.8 1.6",
    "speed 2 7",
    "size 0.12 0.3",
    "colorOverLife {\n  key 0 1 0.6 0.95 1\n  key 0.6 0.85 0.55 0.9 0.8\n  key 1 0.4 0.3 0.6 0\n}",
    "gravity 0 -2 0",
    "drag 1.5",
    'texture "fx/spark.png"',
    "blend additive",
    "render sprite 14",
  ]);
  e("motes", [
    "# The caves' air: slow motes drifting up, each a layer of fx/motes-array.ogt at random.",
    "maxParticles 40",
    "shape box 14 6 0",
    "rate 3",
    "lifetime 4 7",
    "speed 0.05 0.2",
    "size 0.08 0.14",
    "sizeOverLife 0 0 0.2 1 0.8 1 1 0",
    "colorOverLife {\n  key 0 1 0.9 0.6 0\n  key 0.3 1 0.9 0.6 0.7\n  key 1 1 0.9 0.6 0\n}",
    "gravity 0 0.08 0",
    "noise 0.3 0.4 0.3",
    'array "fx/motes-array.ogt" random',
    "render sprite 6",
  ]);
  e("embers", [
    "# A torch's embers rising.",
    "maxParticles 16",
    "shape sphere 0.1",
    "spread 25",
    "rate 2.5",
    "lifetime 0.7 1.3",
    "speed 0.6 1.2",
    "size 0.05 0.09",
    "colorOverLife {\n  key 0 1 0.85 0.4 1\n  key 1 0.9 0.35 0.15 0\n}",
    "gravity 0 0.4 0",
    "noise 0.4 1 0.4",
    'texture "fx/spark.png"',
    "blend additive",
    "render sprite 6",
  ]);
}

// ------------------------------------------------------------------ sound

/** og sfx presets, written as `og sfx <preset> --out <file>` writes them, with the game's own seed, pitch, length and volume. */
function sfx(): void {
  const presets: [string, string, string, number, number, number, number][] = [
    ["jump", "jump", "a quick upward square chirp", 3, 1.15, 0.8, 0.8],
    ["land", "land", "a soft low thud", 0, 1, 1, 0.9],
    ["step", "footstep", "a short filtered noise tick", 2, 1.1, 0.8, 0.7],
    ["swing", "whoosh", "a band-passed noise swell", 1, 1.6, 0.45, 0.7],
    ["hit", "hit", "a punchy noise crack with a low body", 0, 1.05, 1, 1],
    ["hurt", "hurt", "a falling buzzy saw", 1, 1, 1, 0.9],
    ["glimmer", "coin", "a bright two-note pickup chime", 0, 1.1, 1, 0.7],
    ["chime", "powerup", "a rising vibrato sweep with an arpeggio", 2, 1.2, 1, 0.9],
    ["buy", "pickup", "a short rising blip for items", 0, 1, 1, 0.9],
    ["checkpoint", "levelup", "a three-note rising fanfare", 4, 1.3, 0.7, 0.7],
    ["boss", "explosion", "a long low-passed noise boom", 0, 0.8, 1.2, 1],
    ["win", "levelup", "a three-note rising fanfare", 0, 1, 1.4, 1],
    ["ui-click", "click", "a tiny UI tick", 0, 1, 1, 0.8],
    ["ui-select", "select", "a two-step UI confirm", 0, 1, 1, 0.8],
    ["ui-back", "back", "a falling UI cancel", 0, 1, 1, 0.8],
    ["ui-error", "error", "a low double buzz", 0, 1, 1, 0.8],
  ];
  for (const [file, preset, doc, seed, pitch, length, volume] of presets) {
    write(
      `audio/${file}.ogt`,
      ["type sfx", "version 1", "---", `# ${doc}: og.load.sound(world, "<this file>") plays it.`, `preset "${preset}"`, `seed ${seed}`, `pitch ${pitch}`, `length ${length}`, `volume ${volume}`].join("\n") + "\n",
    );
  }
}

function wav(rate: number, channels: number, samples: Float32Array): Uint8Array {
  const pcm = new Int16Array(samples.length);
  for (let i = 0; i < samples.length; i++) pcm[i] = Math.round(clamp(samples[i]!, -1, 1) * 32767);
  const out = new Uint8Array(44 + pcm.byteLength);
  const dv = new DataView(out.buffer);
  const tag = (at: number, s: string) => {
    for (let i = 0; i < 4; i++) out[at + i] = s.charCodeAt(i);
  };
  tag(0, "RIFF");
  dv.setUint32(4, 36 + pcm.byteLength, true);
  tag(8, "WAVE");
  tag(12, "fmt ");
  dv.setUint32(16, 16, true);
  dv.setUint16(20, 1, true);
  dv.setUint16(22, channels, true);
  dv.setUint32(24, rate, true);
  dv.setUint32(28, rate * channels * 2, true);
  dv.setUint16(32, channels * 2, true);
  dv.setUint16(34, 16, true);
  tag(36, "data");
  dv.setUint32(40, pcm.byteLength, true);
  out.set(new Uint8Array(pcm.buffer), 44);
  return out;
}

/** A church-bell toll: the hum, prime, tierce, quint and nominal partials, each decaying at its own rate. 2.6 s mono. */
function bellToll(): Uint8Array {
  const RATE = 44100;
  const n = Math.round(2.6 * RATE);
  const out = new Float32Array(n);
  const f = 392;
  const partials: [number, number, number][] = [
    [0.5, 0.5, 0.9],
    [1, 0.8, 1.4],
    [1.183, 0.45, 2.2],
    [1.506, 0.3, 2.8],
    [2, 0.5, 3.0],
    [2.514, 0.18, 4.5],
    [3.011, 0.12, 6],
  ];
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    let v = 0;
    for (const [ratio, amp, decay] of partials) v += Math.sin(2 * Math.PI * f * ratio * t + ratio) * amp * Math.exp(-t * decay);
    out[i] = v * Math.min(1, t / 0.002) * 0.28;
  }
  return wav(RATE, 1, out);
}

/**
 * "Under the Hill": 32 bars at 90 bpm in D minor pentatonic, about 85 s,
 * looping seamlessly (each voice's tail wraps to the start). A warm pad
 * changes chord every two bars; a bell-like pluck walks the chord in
 * eighths, panned left and right; a soft bass on beats one and three from
 * bar 5; brushed noise on the off-beats from bar 9; a seeded melody in bars
 * 9 to 28. 44.1 kHz stereo.
 */
function music(): Uint8Array {
  const RATE = 44100;
  const BPM = 90;
  const BARS = 32;
  const beat = 60 / BPM;
  const N = Math.round(BARS * 4 * beat * RATE);
  const L = new Float32Array(N);
  const R = new Float32Array(N);
  const midi = (m: number) => 440 * 2 ** ((m - 69) / 12);
  // Dm(add9), Bb, F, C/E, Gm7, Bb, Am7, Asus4: two bars each.
  const CHORDS = [
    [38, 57, 62, 65, 69],
    [34, 58, 62, 65, 70],
    [41, 57, 60, 65, 69],
    [40, 55, 60, 64, 67],
    [43, 58, 62, 65, 69],
    [34, 58, 62, 65, 70],
    [45, 57, 60, 64, 67],
    [45, 57, 62, 64, 69],
  ];
  const PENTA = [62, 65, 67, 69, 72, 74, 77, 79];
  const r = rng(90);
  const add = (start: number, dur: number, fn: (t: number) => number, pan: number) => {
    const s0 = Math.floor(start * RATE);
    const len = Math.floor(dur * RATE);
    const gl = Math.cos(((pan + 1) * Math.PI) / 4);
    const gr = Math.sin(((pan + 1) * Math.PI) / 4);
    for (let i = 0; i < len; i++) {
      const v = fn(i / RATE);
      const k = (s0 + i) % N; // the tail wraps to the start: a seamless loop
      L[k]! += v * gl;
      R[k]! += v * gr;
    }
  };
  const pad = (f: number, dur: number, amp: number) => (t: number) =>
    amp * Math.min(1, t / 1.2) * Math.min(1, (dur - t) / 1.2) * (Math.sin(2 * Math.PI * f * t) + 0.4 * Math.sin(2 * Math.PI * f * 1.004 * t + 0.7) + 0.2 * Math.sin(4 * Math.PI * f * t));
  const pluck = (f: number, amp: number) => (t: number) => {
    const mod = Math.sin(2 * Math.PI * f * 2 * t) * 1.4 * Math.exp(-t * 9);
    return amp * Math.min(1, t / 0.003) * Math.exp(-t * 4.2) * Math.sin(2 * Math.PI * f * t + mod);
  };
  const bass = (f: number, amp: number) => (t: number) => amp * Math.min(1, t / 0.015) * Math.exp(-t * 2) * (Math.sin(2 * Math.PI * f * t) + 0.25 * Math.sin(4 * Math.PI * f * t));
  const bellv = (f: number, amp: number) => (t: number) => amp * Math.min(1, t / 0.003) * Math.exp(-t * 2) * Math.sin(2 * Math.PI * f * t + Math.sin(2 * Math.PI * f * 3.5 * t) * 1.8 * Math.exp(-t * 3));
  const hat = (amp: number, seed: number) => {
    const g = rng(seed);
    let lp = 0;
    return (t: number) => {
      const w = g() * 2 - 1;
      lp += (w - lp) * 0.6;
      return amp * Math.exp(-t * 40) * (w - lp);
    };
  };
  let note = 3;
  for (let bar = 0; bar < BARS; bar++) {
    const chord = CHORDS[Math.floor(bar / 2) % CHORDS.length]!;
    const t0 = bar * 4 * beat;
    if (bar % 2 === 0) chord.slice(1).forEach((m, i) => add(t0, 8 * beat + 1.2, pad(midi(m - 12), 8 * beat + 1.2, 0.035), i % 2 === 0 ? -0.4 : 0.4));
    for (let e = 0; e < 8; e++) {
      const m = chord[1 + ((e * 2 + (bar % 3)) % 4)]!;
      add(t0 + e * beat * 0.5, 1.6, pluck(midi(m), (e % 2 === 0 ? 0.06 : 0.04)), e % 2 === 0 ? -0.5 : 0.5);
    }
    if (bar >= 4) for (const b of [0, 2]) add(t0 + b * beat, 1.6, bass(midi(chord[0]!), 0.16), 0);
    if (bar >= 8) for (let q = 0; q < 4; q++) add(t0 + (q + 0.5) * beat, 0.12, hat(0.05, bar * 8 + q), q % 2 === 0 ? 0.3 : -0.3);
    if (bar >= 8 && bar < 28) {
      for (let q = 0; q < 4; q++) {
        if (r() < 0.4) continue;
        note = clamp(note + Math.floor(r() * 5) - 2, 0, PENTA.length - 1);
        add(t0 + q * beat + (r() < 0.25 ? beat / 2 : 0), 2.2, bellv(midi(PENTA[note]!), 0.055), (r() - 0.5) * 0.6);
      }
    }
  }
  let peak = 0;
  let ll = 0;
  let lr = 0;
  for (let i = 0; i < N; i++) {
    ll += (L[i]! - ll) * 0.4;
    lr += (R[i]! - lr) * 0.4;
    L[i] = ll;
    R[i] = lr;
    peak = Math.max(peak, Math.abs(ll), Math.abs(lr));
  }
  const gain = 0.7 / peak;
  const inter = new Float32Array(N * 2);
  for (let i = 0; i < N; i++) {
    inter[i * 2] = L[i]! * gain;
    inter[i * 2 + 1] = R[i]! * gain;
  }
  return wav(RATE, 2, inter);
}

// ------------------------------------------------------------------- main

palette();
tileset();
write("maps/hollow.tmj", JSON.stringify(tiledMap(), null, 1) + "\n");
hero();
beetle();
moth();
mole();
bell();
items();
motes();
sky();
emitters();
sfx();
write("audio/bell.wav", bellToll());
write("music/hollow.wav.ogt", "type import\nversion 1\n---\n# The music (tools/make-assets.ts): cooked in 4-second chunks and\n# streamed, never resident whole (og-audio §5).\nsound {\n  stream true\n  chunk 4\n}\n");
if (!process.argv.includes("--no-music")) write("music/hollow.wav", music());
console.log(`made Hollowbell's assets under ${ASSETS}`);
