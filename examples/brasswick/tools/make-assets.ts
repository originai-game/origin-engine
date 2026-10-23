// Brasswick's asset generator: every source asset of the game is drawn or
// synthesised here, deterministically (seeded, no clock, no download).
//
//   bun tools/make-assets.ts            everything, the music included
//   bun tools/make-assets.ts --no-music everything but assets/music/vault.wav
//
// What it writes under assets/:
// - tiles/vault.png + vault.ogt: the 16 px tileset (the vault plate's 47
//   blob shapes as the auto-tiled terrain `plate`, the back wall's panels,
//   grates, spikes, the far pipes and mid girders of the parallax layers,
//   gauges, valves and an animated vent).
// - maps/level-1.ogt .. level-3.ogt: the three levels in the engine's own
//   tilemap grammar (og-game §8), from tools/levels.ts: the far and mid
//   layers repeating behind with parallax, the back wall, the plate terrain
//   with collision, the grates, the decor, and the objects (the spawn,
//   crates, plates, doors, the gate, gears, lamps, the exit, ticks, wall
//   lamps, vents, the lift, one-way grates, spikes, the level's zone).
// - sprites/*.png + *.ogt: flipbook sheets (Wick the automaton, the tick,
//   the gear, the plate, the lamp, the exit hatch) and single sprites (the
//   crate, the door, the gate, the lift).
// - fx/*.png and fx/*.ogt: a spark and a puff, and the emitters.
// - audio/*.ogt: og sfx presets, cooked into sounds by og cook.
// - music/vault.wav: "The Vault Turns", 24 bars at 100 bpm (about 58 s,
//   44.1 kHz stereo), cooked in chunks and streamed (music/vault.wav.ogt).
//   About 10 MB, so it is not committed (.gitignore).
// - skins/brasswick/: the engine's fantasy style pack copied from
//   templates/ui-kit (its atlas, icons and fonts are not committed:
//   .gitignore) with the game's skin.ogt, recoloured brass and soot.

import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { deflateSync } from "node:zlib";
import { LEVELS, solid, type Level } from "./levels.ts";

const ROOT = resolve(import.meta.dir ?? ".", "..");
const ASSETS = join(ROOT, "assets");
const ENGINE = resolve(ROOT, "..", "..");

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
  for (let y = 0; y < h; y++) raw.set(px.subarray(y * w * 4, (y + 1) * w * 4), y * (w * 4 + 1) + 1);
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

/** The colours, by name: long brass, copper, iron and teal ramps (the detailed-pixel style's hue-shifted ramps). */
const P = {
  soot0: "#0b0a0f",
  soot1: "#15121b",
  soot2: "#1f1a27",
  soot3: "#2a2333",
  iron0: "#332b3d",
  iron1: "#40374a",
  iron2: "#51475a",
  iron3: "#695d70",
  iron4: "#877b8c",
  iron5: "#a99dab",
  brass0: "#38260f",
  brass1: "#5a3d14",
  brass2: "#86601c",
  brass3: "#b5872a",
  brass4: "#dcb244",
  brass5: "#f4da7c",
  copper0: "#3b1a14",
  copper1: "#682d1d",
  copper2: "#9c4a28",
  copper3: "#cc743f",
  copper4: "#eea565",
  teal0: "#0e2628",
  teal1: "#18443f",
  teal2: "#257163",
  teal3: "#43aa92",
  teal4: "#94e2c8",
  wood0: "#2a1a11",
  wood1: "#47301c",
  wood2: "#6a4828",
  wood3: "#906338",
  wood4: "#b98a53",
  ember: "#ff8f3a",
  flame: "#ffd36a",
  glow: "#fff4c8",
  red: "#c43a3c",
  redDk: "#6a1d24",
  white: "#f6f0e2",
} as const;
type Colour = keyof typeof P;

function rgb(c: Colour): [number, number, number] {
  const h = P[c];
  return [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
}

class Pix {
  readonly px: Uint8Array;
  constructor(readonly w: number, readonly h: number) {
    this.px = new Uint8Array(w * h * 4);
  }
  set(x: number, y: number, c: Colour, a = 255): void {
    x = Math.round(x);
    y = Math.round(y);
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return;
    const [r, g, b] = rgb(c);
    const i = (y * this.w + x) * 4;
    this.px[i] = r;
    this.px[i + 1] = g;
    this.px[i + 2] = b;
    this.px[i + 3] = a;
  }
  alpha(x: number, y: number): number {
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return 0;
    return this.px[(y * this.w + x) * 4 + 3]!;
  }
  rect(x: number, y: number, w: number, h: number, c: Colour, a = 255): this {
    for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) this.set(x + i, y + j, c, a);
    return this;
  }
  /** A filled ellipse about (cx, cy). */
  oval(cx: number, cy: number, rx: number, ry: number, c: Colour): this {
    for (let y = Math.floor(cy - ry); y <= Math.ceil(cy + ry); y++) {
      for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x++) {
        const dx = (x + 0.5 - cx) / rx;
        const dy = (y + 0.5 - cy) / ry;
        if (dx * dx + dy * dy <= 1) this.set(x, y, c);
      }
    }
    return this;
  }
  line(x0: number, y0: number, x1: number, y1: number, c: Colour): this {
    const n = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0), 1);
    for (let i = 0; i <= n; i++) this.set(x0 + ((x1 - x0) * i) / n, y0 + ((y1 - y0) * i) / n, c);
    return this;
  }
  /** A one-pixel outline round every opaque pixel, in `c` (the style's coloured outline). */
  outline(c: Colour): this {
    const solid = new Uint8Array(this.w * this.h);
    for (let y = 0; y < this.h; y++) for (let x = 0; x < this.w; x++) solid[y * this.w + x] = this.alpha(x, y) > 0 ? 1 : 0;
    for (let y = 0; y < this.h; y++) {
      for (let x = 0; x < this.w; x++) {
        if (solid[y * this.w + x]) continue;
        const near = [[1, 0], [-1, 0], [0, 1], [0, -1]].some(([dx, dy]) => {
          const nx = x + dx!;
          const ny = y + dy!;
          return nx >= 0 && ny >= 0 && nx < this.w && ny < this.h && solid[ny * this.w + nx] === 1;
        });
        if (near) this.set(x, y, c);
      }
    }
    return this;
  }
  blit(src: Pix, ox: number, oy: number): this {
    for (let y = 0; y < src.h; y++) {
      for (let x = 0; x < src.w; x++) {
        const i = (y * src.w + x) * 4;
        if (src.px[i + 3] === 0) continue;
        const tx = ox + x;
        const ty = oy + y;
        if (tx < 0 || ty < 0 || tx >= this.w || ty >= this.h) continue;
        this.px.set(src.px.subarray(i, i + 4), (ty * this.w + tx) * 4);
      }
    }
    return this;
  }
  /** A copy mirrored left to right. */
  flipped(): Pix {
    const out = new Pix(this.w, this.h);
    for (let y = 0; y < this.h; y++) for (let x = 0; x < this.w; x++) {
      const i = (y * this.w + x) * 4;
      out.px.set(this.px.subarray(i, i + 4), (y * this.w + (this.w - 1 - x)) * 4);
    }
    return out;
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

const PIXEL_IMPORT = "type import\nversion 1\n---\n# Pixel art (tools/make-assets.ts): lossless, no mips, sampled nearest.\ntexture {\n  kind pixel\n}\n";

function writePixel(rel: string, p: Pix): void {
  write(rel, p.png());
  write(`${rel}.ogt`, PIXEL_IMPORT);
}

// ------------------------------------------------------------------ tileset

const T = 16;
const N = 1, NE = 2, E = 4, SE = 8, S = 16, SW = 32, W = 64, NW = 128;

/** A corner bit counts only when both of its edge bits are set (blob47). */
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

export const TILE = {
  back: [47, 48, 49, 50],
  grate: { left: 51, mid: 52, right: 53 },
  spikes: 54,
  far: { pipeL: 55, pipeR: 56, pipeH: 57, joint: 58 },
  mid: { beam: 59, up: 60, down: 61, post: 62 },
  decor: { gauge: 63, valve: 64, pipeV: 65, pipeH: 66 },
  vent: [67, 68, 69, 70],
  chain: 71,
} as const;
const TILE_COLS = 8;
const TILE_ROWS = 9;

/** A riveted vault plate; its exposed edges take a brass rim, its underside a soot shadow. */
function plateTile(m: number): Pix {
  const t = new Pix(T, T);
  t.rect(0, 0, T, T, "iron1");
  // Clusters of wear, not noise: short horizontal streaks in two tones.
  for (let i = 0; i < 6; i++) {
    const h = hashn(m, i, 3);
    const x = h % 13;
    const y = (h >>> 8) % 15;
    const len = 2 + ((h >>> 16) % 3);
    for (let k = 0; k < len; k++) t.set(x + k, y, i % 2 === 0 ? "iron2" : "iron0");
  }
  // The seams between plates every 8 px, and their rivets.
  for (let x = 0; x < T; x++) t.set(x, 7, "iron0");
  for (let y = 0; y < T; y++) if (y !== 7) t.set(7, y, (y & 1) === 0 ? "iron0" : "iron1");
  for (const [rx, ry] of [[2, 2], [12, 2], [2, 11], [12, 11]] as const) {
    t.set(rx, ry, "iron4");
    t.set(rx + 1, ry + 1, "iron0");
  }
  if (!(m & W)) {
    t.rect(0, 0, 2, T, "iron3");
    t.rect(0, 0, 1, T, "iron4");
  }
  if (!(m & E)) {
    t.rect(T - 2, 0, 2, T, "iron0");
    t.rect(T - 1, 0, 1, T, "soot2");
  }
  if (!(m & S)) {
    t.rect(0, T - 3, T, 3, "iron0");
    t.rect(0, T - 1, T, 1, "soot1");
  }
  if (!(m & N)) {
    // A brass lip on every top face: what Wick stands on reads at a glance.
    t.rect(0, 0, T, 4, "brass2");
    t.rect(0, 0, T, 2, "brass3");
    for (let x = 0; x < T; x++) {
      if (hashn(m, x, 1) % 4 === 0) t.set(x, 0, "brass4");
      if (x % 4 === 1) t.set(x, 2, "brass1");
    }
    t.rect(0, 4, T, 1, "iron0");
  }
  if (m & N && m & W && !(m & NW)) t.rect(0, 0, 2, 2, "iron3");
  if (m & N && m & E && !(m & NE)) t.rect(T - 2, 0, 2, 2, "iron0");
  if (m & S && m & W && !(m & SW)) t.rect(0, T - 2, 2, 2, "iron0");
  if (m & S && m & E && !(m & SE)) t.rect(T - 2, T - 2, 2, 2, "soot2");
  return t;
}

/** The back wall's riveted panels, dim, drawn as they look in full light (og.Lighting2D darkens them). */
function backTile(v: number): Pix {
  const t = new Pix(T, T);
  t.rect(0, 0, T, T, "soot3");
  for (let i = 0; i < 5; i++) {
    const h = hashn(v, i, 11);
    const x = h % 14;
    const y = (h >>> 8) % 16;
    t.set(x, y, "iron0");
    t.set(x + 1, y, "iron0");
  }
  t.rect(0, 15, T, 1, "soot2");
  t.rect(15, 0, 1, T, "soot2");
  if (v === 1) for (const [x, y] of [[3, 3], [12, 3], [3, 12], [12, 12]] as const) t.set(x, y, "iron1");
  if (v === 2) {
    // A vent grille.
    for (let y = 4; y < 12; y += 2) t.rect(4, y, 8, 1, "soot1");
  }
  if (v === 3) {
    // A bolted hatch plate.
    t.rect(3, 3, 10, 10, "soot2");
    t.rect(3, 3, 10, 1, "iron0");
    for (const [x, y] of [[4, 4], [11, 4], [4, 11], [11, 11]] as const) t.set(x, y, "iron1");
  }
  return t;
}

function grateTile(kind: "left" | "mid" | "right"): Pix {
  const t = new Pix(T, T);
  t.rect(0, 0, T, 3, "brass2");
  t.rect(0, 0, T, 1, "brass4");
  for (let x = 1; x < T; x += 3) t.set(x, 1, "soot1");
  t.rect(0, 3, T, 1, "brass0");
  // Struts under it.
  for (const x of kind === "left" ? [1] : kind === "right" ? [14] : [7]) {
    t.line(x, 4, x + (kind === "left" ? 3 : kind === "right" ? -3 : 0), 9, "iron2");
  }
  return t;
}

function spikesTile(): Pix {
  const t = new Pix(T, T);
  for (let k = 0; k < 4; k++) {
    const cx = 2 + k * 4;
    for (let y = 7; y < 16; y++) {
      const half = Math.floor((y - 7) / 4);
      for (let x = cx - half; x <= cx + half; x++) t.set(x, y, y < 10 ? "iron5" : y < 13 ? "iron4" : "iron3");
    }
  }
  t.rect(0, 14, T, 2, "iron1");
  t.outline("soot0");
  return t;
}

/** The far layer: big pipes in silhouette (parallax 0.25), barely above the dark. */
function farTile(kind: "pipeL" | "pipeR" | "pipeH" | "joint"): Pix {
  const t = new Pix(T, T);
  // Drawn as dark as they show: the far layer is `lit false` (og.Lighting2D leaves it as authored).
  t.rect(0, 0, T, T, "soot0");
  if (kind === "pipeL") {
    t.rect(4, 0, 12, T, "soot1");
    t.rect(6, 0, 2, T, "soot2");
  } else if (kind === "pipeR") {
    t.rect(0, 0, 12, T, "soot1");
    t.rect(10, 0, 2, T, "soot0");
  } else if (kind === "pipeH") {
    t.rect(0, 3, T, 10, "soot1");
    t.rect(0, 5, T, 2, "soot2");
  } else {
    t.rect(0, 3, T, 10, "soot1");
    t.rect(3, 1, 10, 14, "soot2");
    t.rect(5, 5, 6, 6, "soot1");
  }
  return t;
}

/** The mid layer: girders (parallax 0.55), transparent between them. */
function midTile(kind: "beam" | "up" | "down" | "post"): Pix {
  const t = new Pix(T, T);
  if (kind === "beam") {
    t.rect(0, 5, T, 6, "iron0");
    t.rect(0, 5, T, 1, "iron1");
    for (let x = 2; x < T; x += 6) t.set(x, 8, "soot3");
  } else if (kind === "post") {
    t.rect(5, 0, 6, T, "iron0");
    t.rect(5, 0, 1, T, "iron1");
  } else {
    for (let i = 0; i < T; i++) {
      const y = kind === "up" ? T - 1 - i : i;
      t.rect(i, y - 1, 1, 3, "iron0");
    }
  }
  return t;
}

function decorTile(kind: "gauge" | "valve" | "pipeV" | "pipeH"): Pix {
  const t = new Pix(T, T);
  if (kind === "gauge") {
    t.oval(8, 8, 6, 6, "brass2").oval(8, 8, 4.5, 4.5, "white").line(8, 8, 11, 5, "red");
    t.set(8, 8, "soot1");
    t.outline("soot1");
  } else if (kind === "valve") {
    t.rect(7, 8, 2, 8, "iron2");
    t.oval(8, 7, 6, 6, "copper2").oval(8, 7, 4, 4, "soot3").line(3, 7, 13, 7, "copper3").line(8, 2, 8, 12, "copper3");
    t.outline("soot1");
  } else if (kind === "pipeV") {
    t.rect(5, 0, 6, T, "copper2");
    t.rect(6, 0, 1, T, "copper4");
    t.rect(10, 0, 1, T, "copper1");
    t.rect(4, 6, 8, 3, "copper3");
  } else {
    t.rect(0, 5, T, 6, "copper2");
    t.rect(0, 6, T, 1, "copper4");
    t.rect(0, 10, T, 1, "copper1");
    t.rect(6, 4, 3, 8, "copper3");
  }
  return t;
}

function ventTile(f: number): Pix {
  const t = new Pix(T, T);
  t.rect(2, 12, 12, 4, "iron2");
  t.rect(2, 12, 12, 1, "iron4");
  for (let x = 3; x < 13; x += 2) t.set(x, 14, "soot0");
  const glow: Colour[] = ["teal1", "teal2", "teal3", "teal2"];
  t.rect(4, 13, 8, 1, glow[f]!);
  return t;
}

function chainTile(): Pix {
  const t = new Pix(T, T);
  for (let y = 0; y < T; y += 4) {
    t.rect(7, y, 2, 3, "iron3");
    t.set(7, y + 1, "soot1");
  }
  return t;
}

function tileset(): void {
  const atlas = new Pix(T * TILE_COLS, T * TILE_ROWS);
  const put = (id: number, p: Pix) => atlas.blit(p, (id % TILE_COLS) * T, Math.floor(id / TILE_COLS) * T);
  BLOB.forEach((m, i) => put(i, plateTile(m)));
  TILE.back.forEach((id, v) => put(id, backTile(v)));
  put(TILE.grate.left, grateTile("left"));
  put(TILE.grate.mid, grateTile("mid"));
  put(TILE.grate.right, grateTile("right"));
  put(TILE.spikes, spikesTile());
  for (const k of ["pipeL", "pipeR", "pipeH", "joint"] as const) put(TILE.far[k], farTile(k));
  for (const k of ["beam", "up", "down", "post"] as const) put(TILE.mid[k], midTile(k));
  for (const k of ["gauge", "valve", "pipeV", "pipeH"] as const) put(TILE.decor[k], decorTile(k));
  TILE.vent.forEach((id, f) => put(id, ventTile(f)));
  put(TILE.chain, chainTile());
  writePixel("tiles/vault.png", atlas);
  const lines = [
    "type tileset",
    "version 1",
    "---",
    "# Written by tools/make-assets.ts. Tiles 0-46 are the plate terrain's 47 blob shapes",
    "# (tile id = the index of its reduced neighbour mask), 47-50 the back wall, 51-53 a grate,",
    "# 54 spikes, 55-58 the far pipes, 59-62 the mid girders, 63-66 decor, 67-70 a vent, 71 a chain.",
    'texture "tiles/vault.png"',
    `grid ${T} ${T} ${TILE_COLS} ${TILE_ROWS}`,
    "filter nearest",
    `solid ${BLOB.map((_, i) => i).join(" ")}`,
    `tile ${TILE.vent[0]} {`,
    `  frames ${TILE.vent.join(" ")}`,
    "  fps 6",
    "}",
    "terrain plate {",
    "  mode blob47",
    ...BLOB.map((m, i) => `  rule ${m} ${i}`),
    "  default 46",
    "  edge filled",
    "}",
  ];
  write("tiles/vault.ogt", lines.join("\n") + "\n");
}

// --------------------------------------------------------------------- maps

interface Obj {
  name: string;
  kind: string;
  rect: [number, number, number, number];
  props?: Record<string, string>;
}

/** Horizontal runs of one character on a row: [x, y, length]. */
function runs(cells: string[][], ch: string): [number, number, number][] {
  const out: [number, number, number][] = [];
  cells.forEach((row, y) => {
    let x = 0;
    while (x < row.length) {
      if (row[x] !== ch) {
        x++;
        continue;
      }
      const x0 = x;
      while (row[x] === ch) x++;
      out.push([x0, y, x - x0]);
    }
  });
  return out;
}

function mapOf(level: Level, index: number): string {
  const cells = level.rows.map((r) => [...r]);
  const h = cells.length;
  const w = cells[0]!.length;
  cells.forEach((r, y) => {
    if (r.length !== w) throw new Error(`${level.id} row ${y} is ${r.length} wide, not ${w}`);
  });
  const at = (x: number, y: number) => cells[y]?.[x];
  const ids = (fn: (x: number, y: number) => number) => Array.from({ length: h }, (_, y) => Array.from({ length: w }, (_, x) => fn(x, y)));
  const rowsOf = (grid: number[][]) => grid.map((r) => `  row ${r.join(" ")}`);

  // The far pipes: a pair every 11 columns, a run across at rows 5 and 13.
  const far = ids((x, y) => {
    const k = x % 11;
    if (y === 5 || y === 13) return k === 3 || k === 4 ? TILE.far.joint : TILE.far.pipeH;
    if (k === 3) return TILE.far.pipeL;
    if (k === 4) return TILE.far.pipeR;
    return -1;
  });
  // The mid girders: a beam at row 3, posts and braces every 8 columns.
  const mid = ids((x, y) => {
    const k = x % 8;
    if (y === 3) return TILE.mid.beam;
    if (y > 3 && k === 0) return TILE.mid.post;
    if (y === 4 && k === 1) return TILE.mid.down;
    if (y === 4 && k === 7) return TILE.mid.up;
    return -1;
  });
  // The back wall stands near the plate only (two cells round it): the open halls show the far pipes and girders.
  const near = (x: number, y: number) => {
    for (let dy = -2; dy <= 2; dy++) for (let dx = -2; dx <= 2; dx++) if (solid(at(x + dx, y + dy))) return true;
    return false;
  };
  const back = ids((x, y) => (near(x, y) ? TILE.back[hashn(x, y, index) % 9 === 0 ? 1 + (hashn(y, x) % 3) : 0]! : -1));
  const grates = ids((x, y) => {
    if (at(x, y) !== "=") return -1;
    if (at(x - 1, y) !== "=") return TILE.grate.left;
    if (at(x + 1, y) !== "=") return TILE.grate.right;
    return TILE.grate.mid;
  });
  const decor = ids((x, y) => {
    const c = at(x, y);
    if (c === "^") return TILE.spikes;
    if (c === "V") return TILE.vent[0];
    if (c !== "." || solid(at(x, y + 1)) === false) {
      // Pipes and chains hang from ceilings.
      if (c === "." && solid(at(x, y - 1)) && hashn(x, y, 5) % 7 === 0) return TILE.chain;
      return -1;
    }
    // Gauges and valves on the wall just above a floor, now and then.
    const r = hashn(x, y, index, 9) % 23;
    return r === 0 ? TILE.decor.gauge : r === 1 ? TILE.decor.valve : -1;
  });

  const objs: Obj[] = [];
  const counts: Record<string, number> = {};
  const name = (kind: string) => {
    const n = counts[kind] ?? 0;
    counts[kind] = n + 1;
    return `l${index + 1}-${kind}-${n}`;
  };
  // A tick walks its floor: from the cell over the last floor cell (or wall) to the left to the one on the right.
  const patrol = (x: number, y: number): [number, number] => {
    let a = x;
    while (a - 1 > 0 && !solid(at(a - 1, y)) && solid(at(a - 1, y + 1))) a--;
    let b = x;
    while (b + 1 < w - 1 && !solid(at(b + 1, y)) && solid(at(b + 1, y + 1))) b++;
    return [a, b];
  };
  cells.forEach((row, y) =>
    row.forEach((c, x) => {
      switch (c) {
        case "P":
          objs.push({ name: `l${index + 1}-spawn`, kind: "spawn", rect: [x, y, 1, 1] });
          break;
        case "C":
          objs.push({ name: name("crate"), kind: "crate", rect: [x, y, 1, 1] });
          break;
        case "a":
        case "b":
        case "j":
          objs.push({ name: `l${index + 1}-plate-${c}`, kind: "plate", rect: [x, y, 1, 1], props: { opens: `l${index + 1}-${c === "j" ? "gate" : "door"}-${c}` } });
          break;
        case "A":
        case "B": {
          // Up to the ceiling over it.
          let top = y;
          while (top - 1 > 0 && !solid(at(x, top - 1))) top--;
          objs.push({ name: `l${index + 1}-door-${c.toLowerCase()}`, kind: "door", rect: [x, top, 1, y - top + 1] });
          break;
        }
        case "J":
          objs.push({ name: `l${index + 1}-gate-j`, kind: "gate", rect: [x, y - 2, 1, 3] });
          break;
        case "G":
          objs.push({ name: name("gear"), kind: "gear", rect: [x, y, 1, 1] });
          break;
        case "L":
          objs.push({ name: name("lamp"), kind: "lamp", rect: [x, y - 1, 1, 2] });
          break;
        case "E":
          objs.push({ name: `l${index + 1}-exit`, kind: "exit", rect: [x, y - 1, 2, 2] });
          break;
        case "T": {
          const [a, b] = patrol(x, y);
          objs.push({ name: name("tick"), kind: "tick", rect: [x, y, 1, 1], props: { min: `${a}`, max: `${b + 1}` } });
          break;
        }
        case "*":
          objs.push({ name: name("light"), kind: "light", rect: [x, y, 1, 1] });
          break;
        case "V":
          objs.push({ name: name("vent"), kind: "vent", rect: [x, y, 1, 1] });
          break;
        case "M": {
          const end = row.indexOf("m");
          if (end < 0) throw new Error(`${level.id}: a lift without its m`);
          objs.push({ name: name("lift"), kind: "lift", rect: [x, y, 3, 0.5], props: { to: `${end}` } });
          break;
        }
      }
    }),
  );
  for (const [x, y, n] of runs(cells, "=")) objs.push({ name: name("grate"), kind: "oneway", rect: [x, y, n, 0.25] });
  for (const [x, y, n] of runs(cells, "^")) objs.push({ name: name("spikes"), kind: "spikes", rect: [x, y + 0.5, n, 0.5] });
  objs.push({ name: `l${index + 1}-zone`, kind: "zone", rect: [0, 0, w, h], props: { title: level.name, par: `${level.par}`, ambient: level.ambient.join(" ") } });

  const lines = [
    "type tilemap",
    "version 1",
    "---",
    `# ${level.id} (tools/make-assets.ts from tools/levels.ts): ${w} x ${h} cells, one cell one world unit.`,
    `size ${w} ${h}`,
    `tile ${T} ${T}`,
    "unit 1",
    'tileset "tiles/vault.ogt"',
    "layer far {",
    "  z 0",
    "  parallax 0.25 0.1",
    "  repeat x",
    "  lit false",
    ...rowsOf(far),
    "}",
    "layer mid {",
    "  z 1",
    "  parallax 0.55 0.3",
    "  repeat x",
    ...rowsOf(mid),
    "}",
    "layer back {",
    "  z 2",
    ...rowsOf(back),
    "}",
    "layer plate {",
    "  z 4",
    "  collision true",
    "  terrain plate",
    '  legend "#" 1',
    ...cells.map((r) => `  row "${r.map((c) => (c === "#" ? "#" : ".")).join("")}"`),
    "}",
    "layer planks {",
    "  z 5",
    ...rowsOf(grates),
    "}",
    "layer decor {",
    "  z 6",
    ...rowsOf(decor),
    "}",
  ];
  for (const o of objs) {
    lines.push(`object ${o.name} {`, `  kind "${o.kind}"`, `  rect ${o.rect.join(" ")}`);
    for (const [k, v] of Object.entries(o.props ?? {})) lines.push(`  prop ${k} "${v}"`);
    lines.push("}");
  }
  return lines.join("\n") + "\n";
}

function maps(): void {
  LEVELS.forEach((l, i) => write(`maps/level-${i + 1}.ogt`, mapOf(l, i)));
}

// ------------------------------------------------------------------ sprites

interface Pose {
  bob: number;
  /** Leg offsets: x shift, lift (px up). */
  legL: [number, number];
  legR: [number, number];
  /** The front arm: dx, dy from rest. */
  arm: [number, number];
  lean: number;
  flame: number;
  eye: Colour;
  /** The arm raised above the head (jump, fall, cheer). */
  up?: boolean;
  /** Both arms forward (push). */
  push?: boolean;
  /** The flame out (hurt). */
  out?: boolean;
}

/** Wick, 24 x 32, facing right, the feet on the bottom row. */
function wickFrame(p: Pose): Pix {
  const f = new Pix(24, 32);
  const b = p.bob;
  const lx = p.lean;
  // Legs: iron, brass feet.
  const leg = (dx: number, lift: number, back: boolean) => {
    const x = 9 + dx;
    const top = 25 + b;
    const bottom = 30 - lift;
    f.rect(x, top, 3, Math.max(1, bottom - top), back ? "iron2" : "iron3");
    f.rect(x - (back ? 1 : 0), bottom, 4, 2, back ? "brass1" : "brass2");
    f.set(x + 3 - (back ? 1 : 0), bottom, back ? "brass2" : "brass4");
  };
  leg(p.legR[0] + 3, p.legR[1], true);
  // Back arm.
  if (p.push) f.rect(15 + lx, 18 + b, 6, 3, "copper1");
  else if (p.up) f.rect(6 + lx, 9 + b, 3, 7, "copper1");
  else f.rect(6 + lx - p.arm[0], 17 + b + p.arm[1], 3, 6, "copper1");
  // The body: a round-shouldered brass can with a chest dial.
  f.rect(7 + lx, 15 + b, 11, 11, "brass3");
  f.rect(8 + lx, 14 + b, 9, 1, "brass3");
  f.rect(16 + lx, 15 + b, 2, 11, "brass2");
  f.rect(7 + lx, 15 + b, 1, 10, "brass4");
  f.rect(7 + lx, 24 + b, 11, 2, "brass1");
  f.oval(12 + lx, 19.5 + b, 2.5, 2.5, "teal1");
  f.set(12 + lx, 19 + b, p.out ? "iron2" : "teal3");
  f.set(11 + lx, 18 + b, "teal4");
  for (const ry of [16, 23]) {
    f.set(9 + lx, ry + b, "brass1");
    f.set(15 + lx, ry + b, "brass1");
  }
  leg(p.legL[0], p.legL[1], false);
  // The head: a brass globe with one teal lens.
  const hx = 12 + lx + (p.push ? 1 : 0);
  f.oval(hx, 9 + b, 6, 5.5, "brass3");
  f.oval(hx - 1, 8 + b, 4, 3.5, "brass4");
  f.rect(hx - 6, 12 + b, 12, 2, "brass2");
  f.oval(hx + 2.5, 9 + b, 2.6, 2.6, "brass1");
  f.oval(hx + 2.5, 9 + b, 1.8, 1.8, p.eye);
  f.set(hx + 2, 8 + b, "white");
  // The wick on top and its flame.
  f.rect(hx - 1, 2 + b, 2, 2, "white");
  f.set(hx - 1, 3 + b, "iron4");
  if (!p.out) {
    const fl = p.flame;
    f.set(hx - 1 + (fl === 1 ? 1 : 0), 1 + b, "flame");
    f.set(hx - 1, 0 + b + (fl === 2 ? 1 : 0), "glow");
    f.set(hx, 1 + b, "ember");
  } else f.set(hx, 1 + b, "iron3");
  // Front arm.
  if (p.push) {
    f.rect(16 + lx, 17 + b, 7, 3, "copper2");
    f.rect(21 + lx, 16 + b, 2, 5, "copper3");
  } else if (p.up) {
    f.rect(16 + lx, 8 + b, 3, 8, "copper2");
    f.rect(16 + lx, 6 + b, 3, 2, "copper3");
  } else {
    f.rect(16 + lx + p.arm[0], 17 + b + p.arm[1], 3, 6, "copper2");
    f.rect(16 + lx + p.arm[0], 22 + b + p.arm[1], 3, 2, "copper3");
  }
  return f.outline("soot1");
}

const REST: Pose = { bob: 0, legL: [0, 0], legR: [0, 0], arm: [0, 0], lean: 0, flame: 0, eye: "teal3" };

/** Wick's sheet: idle 0-1, run 2-7, jump 8, fall 9, wall 10, push 11-14, hurt 15, cheer 16-17. */
function wick(): void {
  const frames: Pix[] = [];
  frames.push(wickFrame({ ...REST, flame: 0 }), wickFrame({ ...REST, bob: 1, flame: 2 }));
  const stride: [number, number][] = [[-3, 0], [-1, 2], [2, 1], [3, 0], [1, 2], [-2, 1]];
  for (let i = 0; i < 6; i++) {
    const l = stride[i]!;
    const r = stride[(i + 3) % 6]!;
    frames.push(wickFrame({ ...REST, bob: i % 3 === 1 ? 1 : 0, legL: l, legR: r, arm: [-l[0] >> 1, 0], flame: i % 3 }));
  }
  frames.push(wickFrame({ ...REST, legL: [1, 3], legR: [-1, 2], up: true, flame: 1 }));
  frames.push(wickFrame({ ...REST, legL: [2, 0], legR: [-2, 1], up: true, flame: 2 }));
  // On a wall (facing it, to the right): pressed close, a hand up the wall, knees bent.
  frames.push(wickFrame({ ...REST, lean: 2, legL: [1, 2], legR: [0, 1], arm: [2, -6], flame: 1 }));
  for (let i = 0; i < 4; i++) {
    const l = stride[i * 1.5 >> 0]!;
    frames.push(wickFrame({ ...REST, lean: 2, push: true, bob: i % 2, legL: [l[0] - 2, l[1]], legR: [-l[0] - 2, 0], flame: i % 3 }));
  }
  frames.push(wickFrame({ ...REST, lean: -2, legL: [-2, 1], legR: [2, 0], arm: [-2, -3], eye: "red", out: true }));
  frames.push(wickFrame({ ...REST, up: true, flame: 0 }), wickFrame({ ...REST, bob: 1, up: true, legL: [0, 2], legR: [0, 2], flame: 2 }));
  writePixel("sprites/wick.png", sheet(frames, 6));
  write(
    "sprites/wick.ogt",
    [
      "type flipbook",
      "version 1",
      "---",
      "# Wick (tools/make-assets.ts): 24 x 32 frames facing right (flipX faces left),",
      "# the feet on the bottom row (the pivot). The run's feet touch on frames 0 and 3.",
      'texture "wick.png"',
      "pivot 0.5 0",
      "grid 24 32 6 3",
      "clip idle {",
      "  frames 0 1",
      "  fps 3",
      "  mode pingPong",
      "}",
      "clip run {",
      "  frames 2 3 4 5 6 7",
      "  fps 12",
      '  event 0 "step"',
      '  event 3 "step"',
      "}",
      "clip jump {",
      "  frames 8",
      "  fps 1",
      "}",
      "clip fall {",
      "  frames 9",
      "  fps 1",
      "}",
      "clip wall {",
      "  frames 10",
      "  fps 1",
      "}",
      "clip push {",
      "  frames 11 12 13 14",
      "  fps 6",
      '  event 1 "shove"',
      '  event 3 "shove"',
      "}",
      "clip hurt {",
      "  frames 15",
      "  fps 1",
      "}",
      "clip cheer {",
      "  frames 16 17",
      "  fps 4",
      "  mode pingPong",
      "}",
    ].join("\n") + "\n",
  );
}

/** A tick: a clockwork beetle, 24 x 16, facing right; its key turns as it walks. */
function tickFrame(i: number, squash: boolean): Pix {
  const f = new Pix(24, 16);
  if (squash) {
    f.oval(12, 13, 10, 3, "copper2");
    f.rect(3, 12, 18, 1, "brass3");
    f.rect(2, 15, 20, 1, "iron2");
    f.set(18, 13, "red");
    return f.outline("soot1");
  }
  // Legs, three a side, alternating.
  for (let k = 0; k < 3; k++) {
    const x = 7 + k * 5;
    const up = (k + i) % 2;
    f.rect(x, 13 - up, 1, 3, "iron2");
    f.rect(x - 1 + (up ? 2 : 0), 15 - up, 2, 1, "iron3");
  }
  f.oval(11, 9, 9, 5.5, "copper2");
  f.oval(10, 8, 6, 3.5, "copper3");
  f.rect(3, 9, 16, 1, "brass3");
  f.rect(3, 13, 16, 1, "copper0");
  // The head and its eye.
  f.oval(20, 11, 3, 2.5, "iron2");
  f.set(21, 10, "red");
  // The wind-up key on its back: four turns of a flat key.
  f.rect(11, 2, 1, 2, "iron4");
  const key: [number, number, number, number][] = [[8, 1, 7, 1], [10, 0, 3, 1], [11, 0, 1, 1], [10, 0, 3, 1]];
  const [kx, ky, kw, kh] = key[i % 4]!;
  f.rect(kx, ky, kw, kh, "brass4");
  if (i % 4 === 2) f.rect(10, 0, 3, 2, "brass3");
  return f.outline("soot1");
}

function tick(): void {
  const frames = [0, 1, 2, 3].map((i) => tickFrame(i, false));
  frames.push(tickFrame(0, true));
  writePixel("sprites/tick.png", sheet(frames, 5));
  write(
    "sprites/tick.ogt",
    [
      "type flipbook",
      "version 1",
      "---",
      "# A tick (tools/make-assets.ts): 24 x 16 frames facing right, the feet on the bottom row.",
      'texture "tick.png"',
      "pivot 0.5 0",
      "grid 24 16 5 1",
      "clip walk {",
      "  frames 0 1 2 3",
      "  fps 8",
      "}",
      "clip squashed {",
      "  frames 4",
      "  fps 1",
      "}",
    ].join("\n") + "\n",
  );
}

/** A gear, 12 x 12, eight teeth turning through 45 degrees in six frames. */
function gearFrame(i: number): Pix {
  const f = new Pix(12, 12);
  const a0 = (i / 6) * (Math.PI / 4);
  for (let y = 0; y < 12; y++) {
    for (let x = 0; x < 12; x++) {
      const dx = x + 0.5 - 6;
      const dy = y + 0.5 - 6;
      const r = Math.hypot(dx, dy);
      const a = Math.atan2(dy, dx) - a0;
      const tooth = Math.cos(a * 8) > 0.3;
      if (r < 1.6) continue;
      if (r < 3.8) f.set(x, y, dx + dy < 0 ? "brass5" : "brass3");
      else if (r < 5.4 && tooth) f.set(x, y, dx + dy < 0 ? "brass4" : "brass2");
    }
  }
  return f.outline("brass0");
}

function gear(): void {
  writePixel("sprites/gear.png", sheet([0, 1, 2, 3, 4, 5].map(gearFrame), 6));
  write("sprites/gear.ogt", ["type flipbook", "version 1", "---", "# A gear (tools/make-assets.ts): six frames of a turn.", 'texture "gear.png"', "grid 12 12 6 1", "clip spin {", "  frames 0 1 2 3 4 5", "  fps 10", "}"].join("\n") + "\n");
}

/** A pressure plate, 16 x 16 on the cell's floor: up (its lamp dark), down (lit). */
function plate(): void {
  const frames = [false, true].map((down) => {
    const f = new Pix(16, 16);
    f.rect(1, 14, 14, 2, "iron2");
    f.rect(1, 14, 14, 1, "iron3");
    const top = down ? 13 : 11;
    f.rect(2, top, 12, down ? 1 : 3, "brass3");
    f.rect(2, top, 12, 1, "brass5");
    f.rect(7, 15, 2, 1, down ? "teal4" : "redDk");
    return f.outline("soot1");
  });
  writePixel("sprites/plate.png", sheet(frames, 2));
  write("sprites/plate.ogt", ["type flipbook", "version 1", "---", "# A pressure plate (tools/make-assets.ts): up, then down.", 'texture "plate.png"', "pivot 0.5 0", "grid 16 16 2 1", "clip up {", "  frames 0", "  fps 1", "}", "clip down {", "  frames 1", "  fps 1", "}"].join("\n") + "\n");
}

/** A lamp post, 16 x 32: dark, then lit with a flicker in three frames. */
function lamp(): void {
  const frames = [0, 1, 2, 3].map((i) => {
    const f = new Pix(16, 32);
    f.rect(7, 12, 2, 18, "iron2");
    f.rect(7, 12, 1, 18, "iron4");
    f.rect(4, 29, 8, 3, "iron1");
    f.rect(4, 29, 8, 1, "iron3");
    // The cage.
    f.rect(4, 2, 8, 10, "brass1");
    f.rect(5, 3, 6, 8, i === 0 ? "soot2" : (["flame", "glow", "flame"] as const)[i - 1]!);
    if (i > 0) {
      f.rect(7, 5, 2, 4, "glow");
      f.set(6, 4 + (i % 2), "white");
    }
    f.rect(3, 1, 10, 1, "brass3");
    f.rect(6, 0, 4, 1, "brass4");
    f.rect(4, 12, 8, 1, "brass3");
    return f.outline("soot1");
  });
  writePixel("sprites/lamp.png", sheet(frames, 4));
  write("sprites/lamp.ogt", ["type flipbook", "version 1", "---", "# A checkpoint lamp (tools/make-assets.ts): dark, then lit and flickering.", 'texture "lamp.png"', "pivot 0.5 0", "grid 16 32 4 1", "clip dark {", "  frames 0", "  fps 1", "}", "clip lit {", "  frames 1 2 3 2", "  fps 8", "}"].join("\n") + "\n");
}

/** The exit hatch, 32 x 32: a round porthole that irises open. */
function hatch(): void {
  const frames = [0, 1, 2, 3].map((i) => {
    const f = new Pix(32, 32);
    f.rect(2, 28, 28, 4, "iron2");
    f.oval(16, 15, 14, 14, "brass2");
    f.oval(16, 15, 12, 12, "brass1");
    const open = [0, 4, 8, 11][i]!;
    f.oval(16, 15, 11, 11, "copper2");
    for (let k = 0; k < 8; k++) {
      const a = (k / 8) * Math.PI * 2 + i * 0.3;
      f.line(16, 15, 16 + Math.cos(a) * 11, 15 + Math.sin(a) * 11, "copper1");
    }
    if (open > 0) {
      f.oval(16, 15, open, open, "teal1");
      f.oval(16, 15, open * 0.7, open * 0.7, "teal3");
      f.oval(16, 15, open * 0.35, open * 0.35, "teal4");
    } else f.oval(16, 15, 2, 2, "brass4");
    for (let k = 0; k < 8; k++) {
      const a = (k / 8) * Math.PI * 2;
      f.set(16 + Math.cos(a) * 13, 15 + Math.sin(a) * 13, "brass4");
    }
    return f.outline("soot1");
  });
  writePixel("sprites/hatch.png", sheet(frames, 4));
  write("sprites/hatch.ogt", ["type flipbook", "version 1", "---", "# The exit hatch (tools/make-assets.ts): shut, two frames of its iris, open.", 'texture "hatch.png"', "pivot 0.5 0", "grid 32 32 4 1", "clip shut {", "  frames 0", "  fps 1", "}", "clip open {", "  frames 1 2 3", "  fps 10", "  mode once", "}"].join("\n") + "\n");
}

function crate(): void {
  const f = new Pix(16, 16);
  f.rect(0, 0, 16, 16, "wood2");
  for (let y = 0; y < 16; y += 5) f.rect(0, y, 16, 1, "wood1");
  f.line(2, 2, 13, 13, "wood3").line(2, 3, 12, 13, "wood3");
  f.rect(0, 0, 16, 2, "brass3").rect(0, 14, 16, 2, "brass2").rect(0, 0, 2, 16, "brass3").rect(14, 0, 2, 16, "brass2");
  for (const [x, y] of [[0, 0], [14, 0], [0, 14], [14, 14]] as const) f.rect(x, y, 2, 2, "brass5");
  f.rect(1, 1, 14, 1, "brass4");
  writePixel("sprites/crate.png", f);
}

function door(gate: boolean): Pix {
  const f = new Pix(16, 48);
  if (gate) {
    // A portcullis: bars and a cross band.
    for (let x = 1; x < 16; x += 4) {
      f.rect(x, 0, 3, 46, "iron3");
      f.rect(x, 0, 1, 46, "iron5");
    }
    for (const y of [8, 24, 40]) f.rect(0, y, 16, 3, "brass2");
    for (let x = 1; x < 16; x += 4) f.rect(x, 45, 3, 3, "iron4");
  } else {
    f.rect(1, 0, 14, 48, "iron2");
    f.rect(1, 0, 2, 48, "iron4");
    f.rect(13, 0, 2, 48, "iron1");
    for (const y of [4, 22, 40]) {
      f.rect(1, y, 14, 3, "brass2");
      f.rect(1, y, 14, 1, "brass4");
    }
    f.rect(6, 12, 4, 6, "soot1");
    f.rect(7, 13, 2, 4, "teal2");
  }
  return f.outline("soot1");
}

function lift(): void {
  const f = new Pix(48, 8);
  f.rect(0, 0, 48, 4, "brass3");
  f.rect(0, 0, 48, 1, "brass5");
  f.rect(0, 4, 48, 2, "brass1");
  for (let x = 3; x < 48; x += 6) f.set(x, 2, "brass1");
  f.rect(10, 6, 4, 2, "iron2").rect(34, 6, 4, 2, "iron2");
  writePixel("sprites/lift.png", f.outline("soot1"));
}

function sprites(): void {
  wick();
  tick();
  gear();
  plate();
  lamp();
  hatch();
  crate();
  writePixel("sprites/door.png", door(false));
  writePixel("sprites/gate.png", door(true));
  lift();
}

// ---------------------------------------------------------------------- fx

function fx(): void {
  const spark = new Pix(4, 4);
  spark.rect(1, 0, 2, 4, "white").rect(0, 1, 4, 2, "white");
  writePixel("fx/spark.png", spark);
  const puff = new Pix(8, 8);
  puff.oval(4, 4, 3.5, 3.5, "iron5").oval(3.5, 3.5, 2, 2, "white");
  writePixel("fx/puff.png", puff);
  const emitter = (file: string, doc: string, lines: string[]) => write(`fx/${file}.ogt`, ["type particles", "version 1", "---", `# ${doc} (tools/make-assets.ts).`, ...lines].join("\n") + "\n");
  emitter("dust", "Landing dust: a puff each way at the feet", [
    "maxParticles 24", "shape box 0.3 0.02 0", "spread 70", "rate 0", "burst 0 8", "duration 0.1", "looping false",
    "lifetime 0.25 0.45", "speed 0.8 1.8", "size 0.18 0.32", "sizeOverLife 0 0.6 1 1.2",
    "colorOverLife {", "  key 0 0.75 0.7 0.75 0.8", "  key 1 0.5 0.45 0.5 0", "}", "drag 5", "gravity 0 1 0",
    'texture "fx/puff.png"', "blend alpha", "render sprite 6",
  ]);
  emitter("gear", "A gear taken: brass sparks every way", [
    "maxParticles 32", "shape point", "spread 180", "rate 0", "burst 0 14", "duration 0.1", "looping false",
    "lifetime 0.2 0.45", "speed 2 5", "size 0.1 0.18", "sizeOverLife 0 1 1 0",
    "colorOverLife {", "  key 0 1 0.9 0.5 1", "  key 1 0.9 0.5 0.15 0", "}", "drag 3",
    'texture "fx/spark.png"', "blend additive", "glow 1.5", "render sprite 12",
  ]);
  emitter("stomp", "A tick stomped: sparks and soot", [
    "maxParticles 32", "shape point", "spread 180", "rate 0", "burst 0 18", "duration 0.1", "looping false",
    "lifetime 0.18 0.4", "speed 3 6", "size 0.12 0.2", "sizeOverLife 0 1 1 0",
    "colorOverLife {", "  key 0 1 0.85 0.6 1", "  key 1 0.8 0.35 0.2 0", "}", "drag 4", "gravity 0 -6 0",
    'texture "fx/spark.png"', "blend additive", "glow 1", "render sprite 12",
  ]);
  emitter("steam", "A vent's steam: slow puffs rising", [
    "maxParticles 48", "shape box 0.3 0.02 0", "spread 12", "rate 9", "lifetime 1.1 1.6", "speed 1 1.6",
    "size 0.3 0.5", "sizeOverLife 0 0.6 1 1.8",
    "colorOverLife {", "  key 0 0.8 0.85 0.85 0", "  key 0.2 0.8 0.85 0.85 0.45", "  key 1 0.7 0.75 0.8 0", "}",
    "noise 0.6 0.8 0.4", 'texture "fx/puff.png"', "blend alpha", "render sprite 3",
  ]);
  emitter("exit", "The open hatch: motes drifting out", [
    "maxParticles 40", "shape sphere 0.7", "spread 30", "rate 10", "lifetime 0.8 1.4", "speed 0.3 0.8",
    "size 0.08 0.14", "sizeOverLife 0 0 0.2 1 1 0",
    "colorOverLife {", "  key 0 0.6 1 0.9 1", "  key 1 0.4 0.9 0.8 0", "}", "gravity 0 0.6 0",
    'texture "fx/spark.png"', "blend additive", "glow 1.2", "render sprite 1",
  ]);
}

// ------------------------------------------------------------------- audio

function sfx(): void {
  const presets: [string, string, string, number, number, number, number][] = [
    ["jump", "jump", "a quick upward square chirp", 3, 1.25, 0.7, 0.75],
    ["land", "land", "a soft low thud", 1, 1.1, 0.8, 0.8],
    ["step", "footstep", "a short filtered noise tick", 4, 1.4, 0.6, 0.5],
    ["gear", "coin", "a bright two-note pickup chime", 2, 1.15, 1, 0.75],
    ["plate", "click", "a tiny UI tick", 5, 0.55, 1.6, 1],
    ["door", "door", "a low creak and thump", 0, 1, 1, 0.9],
    ["gate", "door", "a low creak and thump", 3, 0.75, 1.4, 0.9],
    ["stomp", "hit", "a punchy noise crack with a low body", 1, 1.2, 0.8, 0.95],
    ["hurt", "hurt", "a falling buzzy saw", 2, 1, 1, 0.9],
    ["lamp", "pickup", "a short rising blip for items", 1, 0.8, 1.2, 0.8],
    ["exit", "levelup", "a three-note rising fanfare", 2, 1, 1.3, 0.9],
    ["shove", "footstep", "a short filtered noise tick", 7, 0.6, 1, 0.6],
    ["ui-click", "click", "a tiny UI tick", 0, 1, 1, 0.8],
    ["ui-hover", "click", "a tiny UI tick", 1, 1.3, 0.6, 0.25],
    ["ui-select", "select", "a two-step UI confirm", 0, 1, 1, 0.8],
    ["ui-back", "back", "a falling UI cancel", 0, 1, 1, 0.8],
    ["ui-error", "error", "a low double buzz", 0, 1, 1, 0.8],
  ];
  for (const [file, preset, doc, seed, pitch, length, volume] of presets) {
    write(`audio/${file}.ogt`, ["type sfx", "version 1", "---", `# ${doc}: og.load.sound(world, "<this file>") plays it.`, `preset "${preset}"`, `seed ${seed}`, `pitch ${pitch}`, `length ${length}`, `volume ${volume}`].join("\n") + "\n");
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

/**
 * "The Vault Turns": 24 bars at 100 bpm in A minor, about 58 s, looping
 * seamlessly (tails wrap to the start). A music-box pluck walks each chord
 * in eighths, a low bowed pad holds it, a tick-tock of filtered clicks keeps
 * the clockwork on every beat from bar 3, a soft bass from bar 5 and a
 * seeded glockenspiel melody in bars 9 to 20. 44.1 kHz stereo.
 */
function music(): Uint8Array {
  const RATE = 44100;
  const BPM = 100;
  const BARS = 24;
  const beat = 60 / BPM;
  const N = Math.round(BARS * 4 * beat * RATE);
  const L = new Float32Array(N);
  const R = new Float32Array(N);
  const midi = (m: number) => 440 * 2 ** ((m - 69) / 12);
  // Am, F, C, G, Dm, Am, E7sus, E: two bars each, then the first four again.
  const CHORDS = [
    [45, 57, 60, 64, 69],
    [41, 57, 60, 65, 69],
    [48, 55, 60, 64, 67],
    [43, 55, 59, 62, 67],
    [38, 57, 62, 65, 69],
    [45, 57, 60, 64, 69],
    [40, 56, 59, 62, 64],
    [40, 56, 59, 64, 68],
  ];
  const SCALE = [69, 71, 72, 74, 76, 77, 79, 81, 84];
  const r = rng(1907);
  const add = (start: number, dur: number, fn: (t: number) => number, pan: number) => {
    const s0 = Math.floor(start * RATE);
    const len = Math.floor(dur * RATE);
    const gl = Math.cos(((pan + 1) * Math.PI) / 4);
    const gr = Math.sin(((pan + 1) * Math.PI) / 4);
    for (let i = 0; i < len; i++) {
      const v = fn(i / RATE);
      const k = (s0 + i) % N;
      L[k]! += v * gl;
      R[k]! += v * gr;
    }
  };
  const pad = (f: number, dur: number, amp: number) => (t: number) =>
    amp * Math.min(1, t / 0.9) * Math.min(1, (dur - t) / 0.9) * (Math.sin(2 * Math.PI * f * t) + 0.5 * Math.sin(2 * Math.PI * f * 2.003 * t + 0.4) + 0.25 * Math.sin(2 * Math.PI * f * 3.01 * t));
  const box = (f: number, amp: number) => (t: number) => amp * Math.min(1, t / 0.002) * Math.exp(-t * 5) * (Math.sin(2 * Math.PI * f * t) + 0.3 * Math.sin(2 * Math.PI * f * 4.2 * t) * Math.exp(-t * 14));
  const bass = (f: number, amp: number) => (t: number) => amp * Math.min(1, t / 0.01) * Math.exp(-t * 2.4) * (Math.sin(2 * Math.PI * f * t) + 0.3 * Math.sin(4 * Math.PI * f * t));
  const glock = (f: number, amp: number) => (t: number) => amp * Math.min(1, t / 0.002) * Math.exp(-t * 2.6) * (Math.sin(2 * Math.PI * f * t) + 0.4 * Math.sin(2 * Math.PI * f * 2.76 * t) * Math.exp(-t * 6));
  const click = (amp: number, seed: number, high: boolean) => {
    const g = rng(seed);
    let lp = 0;
    return (t: number) => {
      const w = g() * 2 - 1;
      lp += (w - lp) * (high ? 0.7 : 0.25);
      return amp * Math.exp(-t * (high ? 90 : 60)) * (high ? w - lp : lp);
    };
  };
  let note = 4;
  for (let bar = 0; bar < BARS; bar++) {
    const chord = CHORDS[Math.floor(bar / 2) % CHORDS.length]!;
    const t0 = bar * 4 * beat;
    if (bar % 2 === 0) chord.slice(1, 4).forEach((m, i) => add(t0, 8 * beat + 0.9, pad(midi(m - 12), 8 * beat + 0.9, 0.03), i === 1 ? 0 : i === 0 ? -0.5 : 0.5));
    for (let e = 0; e < 8; e++) {
      const m = chord[1 + ((e * 3 + bar) % 4)]! + (e % 4 === 3 ? 12 : 0);
      add(t0 + e * beat * 0.5, 1.2, box(midi(m), e % 2 === 0 ? 0.05 : 0.035), e % 2 === 0 ? -0.35 : 0.35);
    }
    if (bar >= 2) for (let q = 0; q < 4; q++) add(t0 + q * beat, 0.08, click(q % 2 === 0 ? 0.09 : 0.06, bar * 4 + q, q % 2 === 1), q % 2 === 0 ? -0.2 : 0.2);
    if (bar >= 4) for (const b of [0, 2.5]) add(t0 + b * beat, 1.4, bass(midi(chord[0]!), 0.15), 0);
    if (bar >= 8 && bar < 20) {
      for (let q = 0; q < 4; q++) {
        if (r() < 0.35) continue;
        note = clamp(note + Math.floor(r() * 5) - 2, 0, SCALE.length - 1);
        add(t0 + q * beat + (r() < 0.3 ? beat / 2 : 0), 2, glock(midi(SCALE[note]!), 0.05), (r() - 0.5) * 0.5);
      }
    }
  }
  let peak = 0;
  for (let i = 0; i < N; i++) peak = Math.max(peak, Math.abs(L[i]!), Math.abs(R[i]!));
  const gain = 0.7 / peak;
  const inter = new Float32Array(N * 2);
  for (let i = 0; i < N; i++) {
    inter[i * 2] = L[i]! * gain;
    inter[i * 2 + 1] = R[i]! * gain;
  }
  return wav(RATE, 2, inter);
}

// -------------------------------------------------------------------- skin

/**
 * The fantasy style pack (the catalogue's skin for detailed-pixel) copied
 * from the engine's templates/ui-kit as og-taste §1 says, recoloured: a
 * parchment pack under a brass accent and soot ground. Its CJK `fallback`
 * line is dropped (the game shows only Latin-1: English and Spanish).
 */
function skin(): void {
  const from = join(ENGINE, "templates", "ui-kit", "assets", "skins", "fantasy");
  const to = join(ASSETS, "skins", "brasswick");
  mkdirSync(to, { recursive: true });
  for (const f of ["atlas.png", "atlas.png.ogt", "icons.png", "icons.png.ogt", "Cinzel-Bold.ttf", "Cinzel-Bold.ttf.ogt", "Newsreader-Medium.ttf", "Newsreader-Medium.ttf.ogt", "OFL.txt", "LICENSES.md"]) copyFileSync(join(from, f), join(to, f));
  let text = readFileSync(join(from, "skin.ogt"), "utf8");
  const colours: Record<string, string> = {
    backdrop: "#14110d",
    well: "#1d1812",
    primary: "#7a4f12",
    primaryHover: "#6e470f",
    primaryActive: "#5f3d0c",
    highlight: "#6a440e",
    focus: "#1f6a5c",
  };
  for (const [k, v] of Object.entries(colours)) text = text.replace(new RegExp(`^(  ${k}) "#[0-9a-fA-F]+"`, "m"), `$1 "${v}"`);
  text = text.replace(/^name "fantasy"$/m, 'name "brasswick"');
  text = text.replace(/^# The fantasy style pack.*$/m, "# Brasswick's skin: the engine's fantasy style pack (templates/ui-kit, CC0 art, OFL fonts:\n# LICENSES.md), copied by tools/make-assets.ts, its ground soot and its accent brass; the CJK\n# fallback line dropped, the game shows Latin-1 only.");
  text = text.replace(/^  fallback .*\n/m, "");
  writeFileSync(join(to, "skin.ogt"), text);
}

// -------------------------------------------------------------------- main

tileset();
maps();
sprites();
fx();
sfx();
skin();
write("music/vault.wav.ogt", "type import\nversion 1\n---\n# The music (tools/make-assets.ts): cooked in 4-second chunks and\n# streamed, never resident whole (og-audio §5).\nsound {\n  stream true\n  chunk 4\n}\n");
if (!process.argv.includes("--no-music")) write("music/vault.wav", music());
console.log(`made Brasswick's assets under ${ASSETS}`);
