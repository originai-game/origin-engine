// Draws the look packs' UI parts: the nine-slice frames, plates, menu
// entries, cursors, tabs, slider pieces and bar frames the built-in controls
// stand on (docs/ui.md §3.3). Each pack's parts are one function here, so a
// part is redrawn by editing it and running this again; the output is
// deterministic (no randomness but an integer hash).
//
//   bun templates/tools/ui-parts.ts [PACK...]   writes templates/styles/shared/ui/<pack>/
//
// For every part it writes `<part>.png` and its import declaration
// `<part>.png.ts`, and for the pack `templates/themes/<pack>/parts.ts`: each
// part's slice (image pixels), width (points as drawn) and fill, as the
// recipe's `pack.images`.
//
// Smooth packs are drawn at SS x their final size and box-filtered down, at
// SCALE pixels per point. The pixel pack is drawn on its art grid and
// enlarged by nearest neighbour, ART points to an art pixel.
import { mkdirSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { encodePngBest, type Image } from "../../packages/engine/src/node/png.ts";

const ROOT = resolve(import.meta.dir, "..", "..");
const OUT = join(ROOT, "templates", "styles", "shared", "ui");
const SCALE = 4; // image pixels per point
const SS = 3; // supersampling of the smooth packs

// ---------------------------------------------------------------------------
// helpers

type RGBA = [number, number, number, number];
/** A colour: hex, or a top-to-bottom pair of hex colours across the face. */
type Face = string | [string, string];
type Mask = Float32Array;

function rgba(hex: string, a = 1): RGBA {
  const h = hex.replace(/^#/, "");
  const v = [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16));
  if (h.length === 8) a *= parseInt(h.slice(6, 8), 16) / 255;
  return [v[0]!, v[1]!, v[2]!, Math.round(a * 255)];
}

function hash01(x: number, y: number, seed = 0): number {
  let h = Number((BigInt(x) * 374761393n + BigInt(y) * 668265263n + BigInt(seed) * 2147483647n) & 0xffffffffn);
  h = Math.imul(h ^ (h >>> 13), 1274126177) >>> 0;
  return ((h ^ (h >>> 16)) & 0xffff) / 65535;
}

const clamp01 = (v: number) => (v < 0 ? 0 : v > 1 ? 1 : v);

/** PIL's box radius for a three-pass extended box blur of standard deviation `sigma`. */
function boxRadius(sigma: number, passes = 3): number {
  const s2 = (sigma * sigma) / passes;
  const l = Math.floor((Math.sqrt(12 * s2 + 1) - 1) / 2);
  const a = ((2 * l + 1) * (l * (l + 1) - 3 * s2)) / (6 * (s2 - (l + 1) * (l + 1)));
  return l + a;
}

/** An RGBA float canvas drawn at `ss` times the final pixels; masks are float arrays at canvas size. */
class Canvas {
  readonly W: number;
  readonly H: number;
  readonly a: Float32Array;
  constructor(readonly w: number, readonly h: number, readonly ss = SS) {
    this.W = w * ss;
    this.H = h * ss;
    this.a = new Float32Array(this.W * this.H * 4);
  }

  mask(f: (x: number, y: number) => number): Mask {
    const m = new Float32Array(this.W * this.H);
    for (let j = 0; j < this.H; j++) for (let i = 0; i < this.W; i++) m[j * this.W + i] = f((i + 0.5) / this.ss, (j + 0.5) / this.ss);
    return m;
  }

  /** A rounded (r) or chamfered (cut) rectangle's coverage. */
  rrect(x0: number, y0: number, x1: number, y1: number, r: number, cut = 0): Mask {
    const cx = (x0 + x1) / 2, cy = (y0 + y1) / 2, hw = (x1 - x0) / 2, hh = (y1 - y0) / 2;
    return this.mask((x, y) => {
      const dx = Math.abs(x - cx) - (hw - r), dy = Math.abs(y - cy) - (hh - r);
      let d = Math.hypot(Math.max(dx, 0), Math.max(dy, 0)) + Math.min(Math.max(dx, dy), 0) - r;
      if (cut) d = Math.max(d, (Math.abs(x - cx) + Math.abs(y - cy) - (hw + hh - cut)) / Math.SQRT2);
      return clamp01(0.5 - d * this.ss);
    });
  }

  /** A polygon filled by scanline at whole canvas pixels, edges included. */
  poly(pts: [number, number][]): Mask {
    const m = new Float32Array(this.W * this.H);
    const p = pts.map(([x, y]) => [x * this.ss, y * this.ss] as const);
    for (let j = 0; j < this.H; j++) {
      const xs: number[] = [];
      for (let k = 0; k < p.length; k++) {
        const [ax, ay] = p[k]!, [bx, by] = p[(k + 1) % p.length]!;
        if (ay === by) continue;
        const lo = Math.min(ay, by), hi = Math.max(ay, by);
        if (j < lo || j >= hi) continue;
        xs.push(ax + ((j - ay) * (bx - ax)) / (by - ay));
      }
      xs.sort((a, b) => a - b);
      for (let k = 0; k + 1 < xs.length; k += 2) {
        for (let i = Math.max(0, Math.ceil(xs[k]!)); i <= Math.min(this.W - 1, Math.floor(xs[k + 1]!)); i++) m[j * this.W + i] = 1;
      }
    }
    return m;
  }

  ellipse(x0: number, y0: number, x1: number, y1: number): Mask {
    const X0 = x0 * this.ss, Y0 = y0 * this.ss, X1 = x1 * this.ss - 1, Y1 = y1 * this.ss - 1;
    const cx = (X0 + X1) / 2, cy = (Y0 + Y1) / 2, rx = (X1 - X0 + 1) / 2, ry = (Y1 - Y0 + 1) / 2;
    const m = new Float32Array(this.W * this.H);
    for (let j = 0; j < this.H; j++) for (let i = 0; i < this.W; i++) if (((i - cx) / rx) ** 2 + ((j - cy) / ry) ** 2 <= 1) m[j * this.W + i] = 1;
    return m;
  }

  /** Paints `color` through `mask`, source over; a pair is a gradient between rows y0 and y1 (final pixels). */
  paint(mask: Mask, color: string | RGBA | Float32Array, alpha = 1): void {
    const rows = color instanceof Float32Array ? color : null;
    const c = rows ? null : typeof color === "string" ? rgba(color) : (color as RGBA);
    for (let j = 0; j < this.H; j++) {
      const r = rows ? rows.subarray(j * 4, j * 4 + 4) : c!;
      for (let i = 0; i < this.W; i++) {
        const n = j * this.W + i;
        const mv = mask[n]!;
        if (mv === 0) continue;
        const ca = (r[3]! / 255) * mv * alpha;
        const da = this.a[n * 4 + 3]! / 255;
        const oa = ca + da * (1 - ca);
        for (let k = 0; k < 3; k++) {
          const num = r[k]! * ca + this.a[n * 4 + k]! * da * (1 - ca);
          this.a[n * 4 + k] = oa > 0 ? num / Math.max(oa, 1e-6) : 0;
        }
        this.a[n * 4 + 3] = oa * 255;
      }
    }
  }

  /** A top-to-bottom gradient's colour per canvas row. */
  vgrad(top: string, bottom: string, y0: number, y1: number): Float32Array {
    const a = rgba(top), b = rgba(bottom);
    const out = new Float32Array(this.H * 4);
    for (let j = 0; j < this.H; j++) {
      const t = clamp01(((j + 0.5) / this.ss - y0) / Math.max(y1 - y0, 1e-6));
      for (let k = 0; k < 4; k++) out[j * 4 + k] = a[k]! * (1 - t) + b[k]! * t;
    }
    return out;
  }

  /** `mask` under a Gaussian blur of `radius` final pixels (PIL's three-pass extended box blur, edges extended). */
  blur(mask: Mask, radius: number): Mask {
    const { W, H } = this;
    let cur = Float64Array.from(mask, (v) => Math.floor(clamp01(v) * 255));
    const r = boxRadius(radius * this.ss);
    const ri = Math.floor(r), fr = r - ri, norm = 1 / (2 * r + 1);
    const pass = (src: Float64Array, len: number, lines: number, at: (line: number, i: number) => number) => {
      const out = new Float64Array(src.length);
      const line = new Float64Array(len);
      for (let l = 0; l < lines; l++) {
        for (let i = 0; i < len; i++) line[i] = src[at(l, i)]!;
        const v = (i: number) => line[i < 0 ? 0 : i >= len ? len - 1 : i]!;
        let sum = 0;
        for (let i = -ri; i <= ri; i++) sum += v(i);
        for (let i = 0; i < len; i++) {
          out[at(l, i)] = (sum + fr * (v(i - ri - 1) + v(i + ri + 1))) * norm;
          sum += v(i + ri + 1) - v(i - ri);
        }
      }
      return out;
    };
    for (let p = 0; p < 3; p++) cur = pass(cur, W, H, (l, i) => l * W + i);
    for (let p = 0; p < 3; p++) cur = pass(cur, H, W, (l, i) => i * W + l);
    return Float32Array.from(cur, (v) => Math.round(v) / 255);
  }

  /** The final image: a premultiplied box filter of the supersampled canvas, so edges keep their colour. */
  image(): Image {
    const { w, h, ss } = this;
    const out = new Uint8Array(w * h * 4);
    const px = new Float64Array(4);
    for (let y = 0; y < h; y++)
      for (let x = 0; x < w; x++) {
        px.fill(0);
        for (let j = 0; j < ss; j++)
          for (let i = 0; i < ss; i++) {
            const n = ((y * ss + j) * this.W + x * ss + i) * 4;
            const al = this.a[n + 3]!;
            for (let k = 0; k < 3; k++) px[k] += (this.a[n + k]! * al) / 255;
            px[3] += al;
          }
        const al = px[3]! / (ss * ss);
        for (let k = 0; k < 3; k++) {
          const v = px[k]! / (ss * ss);
          out[(y * w + x) * 4 + k] = Math.floor(Math.min(255, Math.max(0, al > 0 ? (v * 255) / Math.max(al, 1e-6) : 0)));
        }
        out[(y * w + x) * 4 + 3] = Math.floor(Math.min(255, Math.max(0, al)));
      }
    return { width: w, height: h, rgba: out };
  }
}

/** `mask` dilated by `width` final pixels: alternating 4- and 8-neighbour steps, an octagon close to a round pen, so rounded corners stay round under an outline. */
function grow(c: Canvas, mask: Mask, width: number): Mask {
  const { W, H } = c;
  let m = Float32Array.from(mask);
  const at = (a: Mask, i: number, j: number) => (i < 0 || j < 0 || i >= W || j >= H ? 0 : a[j * W + i]!);
  for (let s = 0, n = Math.round(width * c.ss); s < n; s++) {
    const out = new Float32Array(W * H);
    for (let j = 0; j < H; j++)
      for (let i = 0; i < W; i++) {
        let v = Math.max(at(m, i, j), at(m, i, j - 1), at(m, i, j + 1), at(m, i - 1, j), at(m, i + 1, j));
        if (s % 2) v = Math.max(v, at(m, i - 1, j - 1), at(m, i + 1, j - 1), at(m, i - 1, j + 1), at(m, i + 1, j + 1));
        out[j * W + i] = v;
      }
    m = out;
  }
  return m;
}

function shift(c: Canvas, mask: Mask, dx: number, dy: number): Mask {
  const { W, H } = c;
  const sx = Math.round(dx * c.ss), sy = Math.round(dy * c.ss);
  const out = new Float32Array(W * H);
  for (let j = 0; j < H; j++)
    for (let i = 0; i < W; i++) {
      const si = i - sx, sj = j - sy;
      if (si >= 0 && sj >= 0 && si < W && sj < H) out[j * W + i] = mask[sj * W + si]!;
    }
  return out;
}

/** Per-pixel helpers on masks. */
const inv = (m: Mask) => m.map((v) => 1 - v);
const sub = (a: Mask, b: Mask) => a.map((v, i) => clamp01(v - b[i]!));
const max = (a: Mask, b: Mask) => a.map((v, i) => Math.max(v, b[i]!));
const mul = (a: Mask, b: Mask) => a.map((v, i) => v * b[i]!);
/** The band `width` final pixels wide inside `m`'s edge. */
const edge = (c: Canvas, m: Mask, width: number) => sub(m, inv(grow(c, inv(m), width)));

interface Part {
  im: Image;
  slice: number | number[];
  width: number | number[];
  fill: boolean;
  tile: boolean;
}

class Pack {
  readonly parts = new Map<string, Part>();
  constructor(readonly name: string) {}

  add(part: string, im: Image, slicePx: number | number[], fill = true, scale = SCALE, tile = false): void {
    const s = Array.isArray(slicePx) ? slicePx : [slicePx, slicePx, slicePx, slicePx];
    const one = new Set(s).size === 1;
    const w = (v: number) => Math.round((v / scale) * 1000) / 1000;
    this.parts.set(part, { im, slice: one ? s[0]! : s, width: one ? w(s[0]!) : s.map(w), fill, tile });
  }

  write(): void {
    const d = join(OUT, this.name);
    mkdirSync(d, { recursive: true });
    for (const f of readdirSync(d)) rmSync(join(d, f));
    const lines = [
      "// Written by templates/tools/ui-parts.ts: the pack's drawn parts as nine-slice images.",
      "import { ref } from '@origingame/engine/author';",
      "",
      "export const images = {",
    ];
    for (const part of [...this.parts.keys()].sort()) {
      const m = this.parts.get(part)!;
      writeFileSync(join(d, `${part}.png`), encodePngBest(m.im));
      writeFileSync(
        join(d, `${part}.png.ts`),
        `import {file} from '@origingame/engine/author';\n\nexport default file.import({\n  "kind": "texture",\n  "source": "shared/ui/${this.name}/${part}.png"\n});\n`,
      );
      const list = (v: number | number[]) => (Array.isArray(v) ? `[${v.join(", ")}]` : `${v}`);
      const fields = [`"src": ref("@styles/shared/ui/${this.name}/${part}.png")`, `"slice": ${list(m.slice)}`, `"width": ${list(m.width)}`];
      if (!m.fill) fields.push('"fill": false');
      if (m.tile) fields.push('"repeat": "tile"');
      lines.push(`  "${part}": { ${fields.join(", ")} },`);
    }
    lines.push("};", "");
    const theme = join(ROOT, "templates", "themes", this.name);
    mkdirSync(theme, { recursive: true });
    writeFileSync(join(theme, "parts.ts"), lines.join("\n"));
  }
}

// ---------------------------------------------------------------------------
// pixel: carved wood and parchment (Stardew Valley's frames, Blasphemous's
// cursor), drawn on an art grid of ART points per art pixel

const ART = 2; // points per art pixel

class Grid {
  readonly p: (string | null)[];
  constructor(readonly w: number, readonly h: number) {
    this.p = new Array(w * h).fill(null);
  }

  set(x: number, y: number, c: string | null): void {
    if (x >= 0 && x < this.w && y >= 0 && y < this.h && c !== null) this.p[y * this.w + x] = c;
  }

  rect(x0: number, y0: number, x1: number, y1: number, c: string): void {
    for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) this.set(x, y, c);
  }

  /** Art pixels from rows of characters, each looked up in `pal`. */
  draw(rows: string[], pal: Record<string, string>): void {
    rows.forEach((row, y) => [...row].forEach((ch, x) => pal[ch] && this.set(x, y, pal[ch]!)));
  }

  image(): Image {
    const k = ART * SCALE;
    const W = this.w * k, H = this.h * k;
    const out = new Uint8Array(W * H * 4);
    for (let y = 0; y < H; y++)
      for (let x = 0; x < W; x++) {
        const c = this.p[Math.floor(y / k) * this.w + Math.floor(x / k)];
        if (c) out.set(rgba(c), (y * W + x) * 4);
      }
    return { width: W, height: H, rgba: out };
  }
}

const PX = {
  ink: "#2a1408", // the outer outline
  "wood-dk": "#6b3415",
  wood: "#a3561f",
  "wood-md": "#c46d2a",
  "wood-lt": "#e89a44",
  "wood-hi": "#ffc777",
  paper: "#fbd28f",
  "paper-dk": "#e6ae62",
  "paper-lt": "#ffe6b3",
  gold: "#ffd23d",
  "gold-dk": "#c47a12",
  "gold-hi": "#fff4b0",
  steel: "#4a3a5c",
  "steel-dk": "#241a33",
  "steel-lt": "#7d6a96",
  night: "#1d1430",
  green: "#26762d",
  "green-dk": "#185420",
  "green-hi": "#62c45a",
  red: "#b3302b",
  "red-dk": "#6e1719",
  "red-hi": "#ec6a52",
};

/** A carved wood frame: ink outline, a 3-tone plank with grain, an inner ink line and a face with the frame's shadow on its top and left. */
function woodFrame(g: Grid, x0: number, y0: number, x1: number, y1: number, face: string, faceDk: string, border = 4, rivets = true): void {
  const ink = PX.ink;
  for (let x = x0 + 1; x < x1 - 1; x++) (g.set(x, y0, ink), g.set(x, y1 - 1, ink));
  for (let y = y0 + 1; y < y1 - 1; y++) (g.set(x0, y, ink), g.set(x1 - 1, y, ink));
  const b = border;
  g.rect(x0 + 1, y0 + 1, x1 - 1, y1 - 1, PX.wood);
  // bevel: light on top/left of the plank, dark on bottom/right
  for (let x = x0 + 1; x < x1 - 1; x++) (g.set(x, y0 + 1, PX["wood-hi"]), g.set(x, y0 + 2, PX["wood-lt"]), g.set(x, y1 - 2, PX["wood-dk"]));
  for (let y = y0 + 1; y < y1 - 1; y++) (g.set(x0 + 1, y, PX["wood-lt"]), g.set(x1 - 2, y, PX["wood-dk"]));
  // grain: short darker streaks along the plank, hashed
  for (let y = y0 + 3; y < y0 + b; y++) for (let x = x0 + 3; x < x1 - 3; x++) if (hash01(x, y, 3) < 0.1) g.set(x, y, PX["wood-md"]);
  for (let y = y1 - b; y < y1 - 2; y++) for (let x = x0 + 3; x < x1 - 3; x++) if (hash01(x, y, 5) < 0.1) g.set(x, y, PX["wood-md"]);
  // inner ink line
  for (let x = x0 + b; x < x1 - b; x++) (g.set(x, y0 + b, ink), g.set(x, y1 - b - 1, ink));
  for (let y = y0 + b; y < y1 - b; y++) (g.set(x0 + b, y, ink), g.set(x1 - b - 1, y, ink));
  // face: flat, with the frame's shadow inside the slice, so the stretched middle stays one colour
  g.rect(x0 + b + 1, y0 + b + 1, x1 - b - 1, y1 - b - 1, face);
  for (let x = x0 + b + 1; x < x1 - b - 1; x++) g.set(x, y0 + b + 1, faceDk);
  for (let y = y0 + b + 1; y < y1 - b - 1; y++) g.set(x0 + b + 1, y, faceDk);
  if (rivets)
    for (const [rx, ry] of [[x0 + 2, y0 + 2], [x1 - 4, y0 + 2], [x0 + 2, y1 - 4], [x1 - 4, y1 - 4]] as const) {
      g.rect(rx, ry, rx + 2, ry + 2, PX["gold-dk"]);
      g.set(rx, ry, PX["gold-hi"]);
    }
}

function pixel(): Pack {
  const p = new Pack("pixel");
  const k = ART * SCALE;
  // panel: 32x32 art px, sliced through the plank, the ink line and the face's shading; the centre parchment
  let g = new Grid(32, 32);
  woodFrame(g, 0, 0, 32, 32, PX.paper, PX["paper-dk"], 6);
  p.add("panel", g.image(), [8 * k, 7 * k, 7 * k, 8 * k]);
  // plate: a slimmer frame (a tooltip, a toast, a caption)
  g = new Grid(16, 16);
  woodFrame(g, 0, 0, 16, 16, PX.paper, PX["paper-dk"], 3, false);
  p.add("plate", g.image(), [5 * k, 4 * k, 4 * k, 5 * k]);

  const plateButton = (face: string, dk: string, hi: string, lip: string, pressed = false) => {
    const g = new Grid(16, 16);
    const ink = PX.ink;
    const top = pressed ? 1 : 0;
    for (let x = 1; x < 15; x++) (g.set(x, top, ink), g.set(x, 15, ink));
    for (let y = top + 1; y < 15; y++) (g.set(0, y, ink), g.set(15, y, ink));
    g.rect(1, top + 1, 15, 15, face);
    if (!pressed) {
      g.rect(1, 12, 15, 15, lip); // the plate's front lip
      for (let x = 1; x < 15; x++) g.set(x, 12, dk);
    }
    for (let x = 2; x < 14; x++) g.set(x, top + 1, hi); // top highlight
    for (let y = top + 2; y < (pressed ? 14 : 11); y++) g.set(1, y, hi);
    return g.image();
  };
  p.add("button", plateButton(PX["wood-lt"], PX["wood-md"], PX["wood-hi"], PX["wood-dk"]), 5 * k);
  p.add("button-down", plateButton(PX["wood-md"], PX.wood, PX["wood-lt"], PX["wood-dk"], true), 5 * k);
  p.add("button-primary", plateButton(PX.green, PX["green-dk"], PX["green-hi"], PX["green-dk"]), 5 * k);
  p.add("button-primary-down", plateButton(PX.green, PX["green-dk"], PX["green-hi"], PX["green-dk"], true), 5 * k);
  p.add("button-danger", plateButton(PX.red, PX["red-dk"], PX["red-hi"], PX["red-dk"]), 5 * k);
  p.add("button-danger-down", plateButton(PX.red, PX["red-dk"], PX["red-hi"], PX["red-dk"], true), 5 * k);

  // menu entry, current: a parchment ribbon lit under the words
  g = new Grid(12, 12);
  g.rect(1, 1, 11, 11, PX["paper-lt"]);
  for (let x = 1; x < 11; x++) (g.set(x, 0, PX["gold-dk"]), g.set(x, 11, PX["gold-dk"]), g.set(x, 10, PX["paper-dk"]));
  for (let y = 1; y < 11; y++) (g.set(0, y, PX["gold-dk"]), g.set(11, y, PX["gold-dk"]));
  p.add("entry-on", g.image(), 3 * k);

  // cursor: a sword pointing right (Blasphemous's and Stardew's pointer)
  g = new Grid(12, 9);
  g.draw(
    ["...........", ".##........", "#gg#.......", "#GgG######.", "#GgsSSSSSs#", "#GgG######.", "#gg#.......", ".##........", "..........."],
    { "#": PX.ink, g: PX["gold-dk"], G: PX.gold, s: PX["steel-lt"], S: "#e8e2f4" },
  );
  p.add("cursor", g.image(), 0);

  // tabs: a wood tab, raised and lit when selected
  const tab = (on: boolean) => {
    const g = new Grid(14, 12);
    const ink = PX.ink;
    const top = on ? 0 : 2;
    for (let x = 1; x < 13; x++) g.set(x, top, ink);
    for (let y = top + 1; y < 12; y++) (g.set(0, y, ink), g.set(13, y, ink));
    g.rect(1, top + 1, 13, 12, on ? PX.paper : PX.wood);
    for (let x = 2; x < 12; x++) g.set(x, top + 1, on ? PX["paper-lt"] : PX["wood-lt"]);
    if (!on) for (let x = 0; x < 14; x++) g.set(x, 11, ink);
    return g.image();
  };
  p.add("tab", tab(false), [4 * k, 4 * k, 2 * k, 4 * k]);
  p.add("tab-on", tab(true), [4 * k, 4 * k, 2 * k, 4 * k]);

  // slider/progress track: a sunk groove; range: gold; thumb: a wood knob
  g = new Grid(8, 8);
  g.rect(1, 1, 7, 7, PX["steel-dk"]);
  for (let x = 1; x < 7; x++) (g.set(x, 0, PX.ink), g.set(x, 7, PX["wood-hi"]), g.set(x, 1, "#0f0a18"));
  for (let y = 1; y < 7; y++) (g.set(0, y, PX.ink), g.set(7, y, PX.ink));
  p.add("track", g.image(), 3 * k);
  g = new Grid(6, 6);
  g.rect(0, 0, 6, 6, PX.gold);
  for (let x = 0; x < 6; x++) (g.set(x, 0, PX["gold-hi"]), g.set(x, 5, PX["gold-dk"]));
  p.add("range", g.image(), 2 * k);
  g = new Grid(10, 12);
  g.draw(
    ["..######..", ".#hhhhhh#.", "#hLLLLLLd#", "#hLLLLLLd#", "#hLLwwLLd#", "#hLLwwLLd#", "#hLLwwLLd#", "#hLLLLLLd#", "#hLLLLLLd#", "#ddddddddd", ".#dddddd#.", "..######.."],
    { "#": PX.ink, h: PX["wood-hi"], L: PX["wood-lt"], d: PX["wood-dk"], w: PX.wood },
  );
  p.add("thumb", g.image(), 0);

  // bar frame: a dark iron frame for a HUD bar
  g = new Grid(10, 8);
  for (let x = 1; x < 9; x++) (g.set(x, 0, PX.ink), g.set(x, 7, PX.ink));
  for (let y = 1; y < 7; y++) (g.set(0, y, PX.ink), g.set(9, y, PX.ink));
  g.rect(1, 1, 9, 7, PX.steel);
  for (let x = 1; x < 9; x++) (g.set(x, 1, PX["steel-lt"]), g.set(x, 6, PX["steel-dk"]));
  g.rect(2, 2, 8, 6, PX.night);
  p.add("bar", g.image(), 3 * k);

  // slot: an inventory cell sunk into the parchment
  g = new Grid(12, 12);
  g.rect(0, 0, 12, 12, PX["paper-dk"]);
  for (let x = 0; x < 12; x++) (g.set(x, 0, PX["wood-dk"]), g.set(x, 11, PX["paper-lt"]));
  for (let y = 0; y < 12; y++) (g.set(0, y, PX["wood-dk"]), g.set(11, y, PX["paper-lt"]));
  g.rect(1, 1, 11, 2, PX.wood);
  g.rect(1, 1, 2, 11, PX.wood);
  p.add("slot", g.image(), 3 * k);

  // checkbox: a sunk parchment box; checked, a green tick drawn in
  const check = (on: boolean) => {
    const g = new Grid(10, 10);
    g.rect(1, 1, 9, 9, PX["paper-lt"]);
    for (let i = 1; i < 9; i++) {
      g.set(i, 0, PX.ink), g.set(i, 9, PX.ink), g.set(0, i, PX.ink), g.set(9, i, PX.ink);
      g.set(i, 1, PX["paper-dk"]), g.set(1, i, PX["paper-dk"]);
    }
    if (on) {
      for (const [x, y] of [[2, 5], [3, 6], [4, 7], [5, 6], [6, 5], [7, 4], [8, 3], [3, 5], [4, 6], [5, 5], [6, 4], [7, 3]] as const) g.set(x, y, PX.green);
      for (const [x, y] of [[4, 8], [5, 7], [6, 6], [7, 5], [8, 4]] as const) g.set(x, y, PX["green-dk"]);
    }
    return g.image();
  };
  p.add("check", check(false), 0);
  p.add("check-on", check(true), 0);

  // header: a carved sign plank for a panel's title
  g = new Grid(20, 12);
  woodFrame(g, 0, 0, 20, 12, PX.wood, PX["wood-dk"], 3, false);
  p.add("header", g.image(), [5 * k, 4 * k, 4 * k, 5 * k]);
  return p;
}

// ---------------------------------------------------------------------------
// the smooth packs' shared shapes

interface Framed {
  ink?: string;
  inkW?: number;
  drop?: string | null;
  dropDx?: number;
  dropDy?: number;
  topLight?: string;
  topH?: number;
  inner?: string;
  innerW?: number;
  glow?: string;
  glowR?: number;
  glowA?: number;
  lip?: string;
  lipH?: number;
  soft?: string;
  softR?: number;
  softDy?: number;
  softA?: number;
  gloss?: string;
  halo?: string;
  haloW?: number;
  line?: string;
  lineW?: number;
}

/** The first and last canvas rows `m` covers, in final pixels. */
function rowSpan(c: Canvas, m: Mask): [number, number] | null {
  let first = -1, last = -1;
  for (let j = 0; j < c.H; j++)
    for (let i = 0; i < c.W; i++)
      if (m[j * c.W + i]! > 0) {
        if (first < 0) first = j;
        last = j;
        break;
      }
  return first < 0 ? null : [first / c.ss, last / c.ss];
}

/**
 * One piece: `shape(c)` returns its face's coverage on canvas `c`; the piece is
 * drawn as soft shadow, glow, halo, hard drop, ink outline, lip, face (a
 * colour or a top-to-bottom pair), gloss, a line on its edge, inner line and
 * top light, in that order.
 */
function framed(w: number, h: number, shape: (c: Canvas) => Mask, face: Face, o: Framed = {}): Image {
  const c = new Canvas(w, h);
  const m = shape(c);
  const { ink, inkW = 0, lip, lipH = 0 } = o;
  const body = ink && inkW ? grow(c, m, inkW) : m;
  const full = lip ? shift(c, body, 0, lipH) : body;
  if (o.soft) c.paint(shift(c, c.blur(max(full, body), o.softR ?? 0), 0, o.softDy ?? 0), o.soft, o.softA ?? 1);
  if (o.glow) c.paint(c.blur(body, o.glowR ?? 0), o.glow, o.glowA ?? 1);
  if (o.halo) c.paint(grow(c, max(full, body), o.haloW ?? 0), o.halo);
  if (o.drop) c.paint(shift(c, body, o.dropDx ?? 0, o.dropDy ?? 0), o.drop);
  if (lip) {
    c.paint(shift(c, body, 0, lipH), ink ?? lip);
    c.paint(shift(c, m, 0, lipH), lip);
  }
  if (ink && inkW) c.paint(body, ink);
  if (Array.isArray(face)) {
    const span = rowSpan(c, m) ?? [0, h];
    c.paint(m, c.vgrad(face[0], face[1], span[0], span[1]));
  } else c.paint(m, face);
  if (o.gloss) {
    const span = rowSpan(c, m);
    if (span) {
      const [y0, y1] = span;
      const innerM = inv(grow(c, inv(m), 2.5 * SCALE));
      const band = c.mask((_x, y) => clamp01(((y0 + (y1 - y0) * 0.46 - y) * c.ss) / 2));
      c.paint(mul(innerM, band), o.gloss);
    }
  }
  if (o.line) c.paint(edge(c, m, o.lineW ?? 0), o.line);
  if (o.inner) c.paint(edge(c, m, o.innerW ?? 0), o.inner);
  if (o.topLight !== undefined) {
    const topH = o.topH ?? 0;
    c.paint(mul(m, c.mask((_x, y) => (y < topH ? 1 : 0))), o.topLight);
  }
  return c.image();
}

/** A parallelogram leaning right by `lean` px over its height. */
const slab = (x0: number, y0: number, x1: number, y1: number, lean: number) => (c: Canvas) =>
  c.poly([[x0 + lean, y0], [x1, y0], [x1 - lean, y1], [x0, y1]]);

/** Points scaled by `s`. */
const pts = (s: number, ...xy: number[]) => Array.from({ length: xy.length / 2 }, (_, i) => [xy[2 * i]! * s, xy[2 * i + 1]! * s] as [number, number]);

// ---------------------------------------------------------------------------
// comic: ink outlines, hard offset shadows, slanted slabs, a hand-drawn
// arrow (Hi-Fi Rush's menus, Borderlands' item cards)

const CO = {
  ink: "#0d0c10",
  char: "#25222c",
  "char-lt": "#3b3745",
  "char-dk": "#17151c",
  paper: "#f7efdc",
  "paper-dk": "#e2d4b4",
  yellow: "#ffd21f",
  "yellow-dk": "#e0a800",
  red: "#d92631",
  "red-dk": "#8f1018",
  cyan: "#29b6e8",
  white: "#ffffff",
};

function comic(): Pack {
  const p = new Pack("comic");
  const S = SCALE;
  // panel: charcoal, a 3 pt ink rim, two cut corners and a hard ink drop
  const w = 64 * S, h = 64 * S, cut = 10 * S;
  const panelShape = (c: Canvas) => {
    const inset = 3 * S, d = 5 * S;
    const x0 = inset, y0 = inset, x1 = w - inset - d, y1 = h - inset - d;
    return c.poly([[x0 + cut, y0], [x1, y0], [x1, y1 - cut], [x1 - cut, y1], [x0, y1], [x0, y0 + cut]]);
  };
  p.add("panel", framed(w, h, panelShape, CO.char, { ink: CO.ink, inkW: 3 * S, drop: CO.ink, dropDx: 5 * S, dropDy: 5 * S, inner: CO["char-lt"], innerW: 1.5 * S }), 22 * S);
  // plate: a charcoal box (a tooltip, a toast, a HUD plate); caption: the same in cream
  const w2 = 40 * S, h2 = 40 * S;
  const box = (c: Canvas) => c.rrect(2.5 * S, 2.5 * S, w2 - 2.5 * S - 4 * S, h2 - 2.5 * S - 4 * S, 1.5 * S);
  p.add("plate", framed(w2, h2, box, CO.char, { ink: CO.ink, inkW: 2.5 * S, drop: CO.ink, dropDx: 4 * S, dropDy: 4 * S, inner: CO["char-lt"], innerW: 1 * S }), 14 * S);
  p.add("caption", framed(w2, h2, box, CO.paper, { ink: CO.ink, inkW: 2.5 * S, drop: CO.ink, dropDx: 4 * S, dropDy: 4 * S }), 14 * S);
  // slot: a charcoal cell with an ink rim
  p.add("slot", framed(w2, h2, (c) => c.rrect(2 * S, 2 * S, w2 - 2 * S, h2 - 2 * S, 1 * S), CO["char-dk"], { ink: CO.ink, inkW: 2 * S, inner: CO["char-lt"], innerW: 1 * S }), 10 * S);

  // slanted slabs: buttons, the current entry, tabs and the header band
  const bw = 48 * S, bh = 20 * S, lean = 5 * S;
  const slabPiece = (face: string, { d = 4 * S, drop = CO.ink as string | null, top = undefined as string | undefined } = {}) => {
    const inkW = 2.5 * S;
    return framed(bw, bh, slab(inkW, inkW, bw - inkW - d, bh - inkW - d, lean), face, { ink: CO.ink, inkW, drop, dropDx: d, dropDy: d, topLight: top, topH: (bh - d) * 0.42 });
  };
  const sl = [8 * S, 14 * S, 8 * S, 14 * S];
  p.add("button", slabPiece(CO.paper, { top: "#ffffff40" }), sl);
  p.add("button-down", slabPiece(CO["paper-dk"], { drop: null }), sl);
  p.add("button-primary", slabPiece(CO.yellow, { top: "#ffffff4d" }), sl);
  p.add("button-primary-down", slabPiece(CO["yellow-dk"], { drop: null }), sl);
  p.add("button-danger", slabPiece(CO.red, { top: "#ffffff26" }), sl);
  p.add("button-danger-down", slabPiece(CO["red-dk"], { drop: null }), sl);
  p.add("entry-on", slabPiece(CO.yellow, { d: 5 * S, drop: CO.red, top: "#ffffff40" }), sl);
  p.add("header", slabPiece(CO.red), sl);
  p.add("tab", slabPiece(CO["char-lt"], { drop: null }), sl);
  p.add("tab-on", slabPiece(CO.yellow, { d: 3 * S }), sl);

  // cursor: a hand-drawn arrow pointing right, white on an ink rim
  const arrow = (c: Canvas) => c.poly(pts(S, 2, 7, 13, 6.2, 12, 1.5, 25, 9.5, 12.5, 18, 13.2, 13, 2.5, 12.6));
  p.add("cursor", framed(28 * S, 20 * S, arrow, CO.white, { ink: CO.ink, inkW: 2 * S, drop: CO.ink, dropDx: 1.5 * S, dropDy: 2 * S }), 0);

  // slider and bars: an ink trough, a yellow range, a white diamond thumb
  const tw = 32 * S, th = 12 * S;
  p.add("track", framed(tw, th, (c) => c.rrect(2 * S, 2 * S, tw - 2 * S, th - 2 * S, 1 * S), CO["char-dk"], { ink: CO.ink, inkW: 2 * S }), 5 * S);
  p.add("range", framed(16 * S, 8 * S, (c) => c.rrect(0, 0, 16 * S, 8 * S, 0.5 * S), CO.yellow, { topLight: "#ffffff59", topH: 3 * S }), 3 * S);
  p.add("thumb", framed(24 * S, 24 * S, (c) => c.poly(pts(S, 12, 2, 21, 11, 12, 20, 3, 11)), CO.white, { ink: CO.ink, inkW: 2.5 * S, drop: CO.ink, dropDx: 1.5 * S, dropDy: 2 * S }), 0);
  p.add("bar", framed(tw, th + 2 * S, slab(2 * S, 2 * S, tw - 2 * S, th, 2 * S), CO["char-dk"], { ink: CO.ink, inkW: 2 * S, drop: CO.ink, dropDx: 2 * S, dropDy: 2 * S }), [6 * S, 7 * S, 6 * S, 7 * S]);

  const check = (on: boolean) => {
    const c = new Canvas(22 * S, 22 * S);
    const m = c.rrect(3 * S, 3 * S, 18 * S, 18 * S, 1 * S);
    c.paint(shift(c, grow(c, m, 2 * S), 2 * S, 2 * S), CO.ink);
    c.paint(grow(c, m, 2 * S), CO.ink);
    c.paint(m, CO.paper);
    if (on) {
      const tick = c.poly(pts(S, 5, 10, 8.5, 9, 10, 12.5, 17.5, 3, 20.5, 5, 10.5, 18.5));
      c.paint(grow(c, tick, 1.5 * S), CO.ink);
      c.paint(tick, CO.red);
    }
    return c.image();
  };
  p.add("check", check(false), 0);
  p.add("check-on", check(true), 0);
  return p;
}

// ---------------------------------------------------------------------------
// minimal: matte cream plates with 45-degree facets, a hairline, a soft
// shadow and gold for the current choice (Bad North's plates, Monument
// Valley's quiet type, ABZU's small marker)

const MI = {
  cream: "#f3ede2", "cream-dk": "#e2d8c6", line: "#c8bba3", slate: "#2d3440", "slate-lt": "#4a5466",
  gold: "#e3a63b", "gold-lt": "#f5c96d", red: "#9c3f34", shadow: "#1a2230", track: "#c8bba3",
};

function minimal(): Pack {
  const p = new Pack("minimal");
  const S = SCALE;
  const facet = (w: number, h: number, inset: number, cut: number) => (c: Canvas) => c.rrect(inset, inset, w - inset, h - inset - 2 * S, 0, cut);
  const w = 48 * S, h = 48 * S;
  p.add("panel", framed(w, h, facet(w, h, 4 * S, 8 * S), MI.cream + "f0", { line: MI.line, lineW: 1 * S, soft: MI.shadow, softR: 4 * S, softDy: 2 * S, softA: 0.28 }), 16 * S);
  const w2 = 32 * S, h2 = 32 * S;
  p.add("plate", framed(w2, h2, facet(w2, h2, 3 * S, 5 * S), MI.cream + "f0", { line: MI.line, lineW: 1 * S, soft: MI.shadow, softR: 3 * S, softDy: 1.5 * S, softA: 0.25 }), 10 * S);
  p.add("slot", framed(w2, h2, facet(w2, h2, 3 * S, 4 * S), MI["cream-dk"], { line: MI.line, lineW: 1 * S }), 10 * S);
  const bw = 40 * S, bh = 20 * S;
  const b = facet(bw, bh, 2.5 * S, 5 * S);
  const sl = [8 * S, 9 * S, 9 * S, 9 * S];
  const lifted = { soft: MI.shadow, softR: 2 * S, softDy: 1 * S };
  p.add("button", framed(bw, bh, b, MI.cream, { line: MI.line, lineW: 1 * S, ...lifted, softA: 0.22 }), sl);
  p.add("button-down", framed(bw, bh, b, MI["cream-dk"], { line: MI.line, lineW: 1 * S }), sl);
  p.add("button-primary", framed(bw, bh, b, MI.slate, { line: MI["slate-lt"], lineW: 1 * S, ...lifted, softA: 0.25 }), sl);
  p.add("button-primary-down", framed(bw, bh, b, MI["slate-lt"]), sl);
  p.add("button-danger", framed(bw, bh, b, MI.red, { ...lifted, softA: 0.25 }), sl);
  p.add("button-danger-down", framed(bw, bh, b, "#7d3128"), sl);
  p.add("entry", framed(bw, bh, b, MI.cream + "e6", { line: MI.line, lineW: 1 * S }), sl);
  p.add("entry-on", framed(bw, bh, b, MI.cream, { line: MI.gold, lineW: 2 * S, soft: MI.gold, softR: 3 * S, softA: 0.45 }), sl);
  // cursor: a small gold diamond
  p.add("cursor", framed(12 * S, 12 * S, (c) => c.poly(pts(S, 6, 1, 11, 6, 6, 11, 1, 6)), MI.gold), 0);
  p.add("track", framed(24 * S, 6 * S, (c) => c.rrect(0, 2 * S, 24 * S, 4 * S, 1 * S), MI.track), [0, 3 * S, 0, 3 * S]);
  p.add("range", framed(24 * S, 6 * S, (c) => c.rrect(0, 2 * S, 24 * S, 4 * S, 1 * S), MI.slate), [0, 3 * S, 0, 3 * S]);
  p.add("thumb", framed(20 * S, 20 * S, (c) => c.poly(pts(S, 10, 2, 18, 10, 10, 18, 2, 10)), MI.cream, { ink: MI.slate, inkW: 1.5 * S, ...lifted, softA: 0.3 }), 0);
  p.add("bar", framed(24 * S, 10 * S, (c) => c.rrect(1 * S, 1 * S, 23 * S, 9 * S, 0, 2 * S), MI.slate + "cc", { line: MI.line, lineW: 1 * S }), 4 * S);

  const check = (on: boolean) => {
    const c = new Canvas(20 * S, 20 * S);
    const m = c.rrect(2 * S, 2 * S, 18 * S, 18 * S, 0, 3 * S);
    c.paint(m, on ? MI.slate : MI.cream);
    c.paint(edge(c, m, 1.2 * S), MI.slate);
    if (on) c.paint(c.poly(pts(S, 6, 10, 8, 8.5, 9.5, 11, 14, 5.5, 15.5, 7, 9.5, 14)), MI.cream);
    return c.image();
  };
  p.add("check", check(false), 0);
  p.add("check-on", check(true), 0);
  return p;
}

// ---------------------------------------------------------------------------
// neon: glowing lines on black, chamfered corners, a lit underline for the
// current choice (Thumper's colour, Octahedron's line frames)

const NE = { black: "#07060f", cyan: "#36f1ff", "cyan-dk": "#0c6f7a", magenta: "#ff2bd6", yellow: "#ffe94a", glass: "#0b0a18e0", ink: "#04040a" };

function neon(): Pack {
  const p = new Pack("neon");
  const S = SCALE;
  const linePiece = (w: number, h: number, inset: number, cut: number, line: string, { face = NE.glass, lw = 1.5 * S, glowR = 4 * S, glowA = 0.75, fill = undefined as string | undefined } = {}) => {
    const c = new Canvas(w, h);
    const m = c.rrect(inset, inset, w - inset, h - inset, 0, cut);
    c.paint(m, face);
    if (fill) c.paint(m, fill);
    const e = edge(c, m, lw);
    c.paint(c.blur(e, glowR), line, glowA);
    c.paint(c.blur(e, glowR / 3), line, glowA);
    c.paint(e, line);
    c.paint(e, "#ffffff", 0.35);
    return c.image();
  };
  p.add("panel", linePiece(48 * S, 48 * S, 8 * S, 8 * S, NE.cyan), 18 * S);
  p.add("plate", linePiece(32 * S, 32 * S, 6 * S, 5 * S, NE.cyan, { glowA: 0.5 }), 12 * S);
  p.add("slot", linePiece(32 * S, 32 * S, 6 * S, 4 * S, NE["cyan-dk"], { glowA: 0.4 }), 12 * S);
  const bw = 48 * S, bh = 26 * S, sl = 11 * S;
  p.add("button", linePiece(bw, bh, 6 * S, 6 * S, NE.cyan), sl);
  p.add("button-down", linePiece(bw, bh, 6 * S, 6 * S, NE.cyan, { fill: NE.cyan + "40" }), sl);
  p.add("button-primary", linePiece(bw, bh, 6 * S, 6 * S, NE.cyan, { face: NE.cyan, glowA: 1 }), sl);
  p.add("button-primary-down", linePiece(bw, bh, 6 * S, 6 * S, NE.cyan, { face: "#8ff8ff", glowA: 1 }), sl);
  p.add("button-danger", linePiece(bw, bh, 6 * S, 6 * S, NE.magenta), sl);
  p.add("button-danger-down", linePiece(bw, bh, 6 * S, 6 * S, NE.magenta, { fill: NE.magenta + "40" }), sl);

  // the current entry: a magenta wash fading right over a lit underline
  const ew = 64 * S, eh = 24 * S;
  let c = new Canvas(ew, eh);
  c.paint(c.mask((x, y) => (y > 3 * S && y < eh - 5 * S ? clamp01(1 - x / ew) : 0)), NE.magenta, 0.28);
  const under = c.mask((x, y) => (y > eh - 6 * S && y < eh - 4.5 * S ? clamp01(1.15 - x / ew) : 0));
  c.paint(c.blur(under, 3 * S), NE.magenta, 0.9);
  c.paint(under, NE.magenta);
  c.paint(under, "#ffffff", 0.4);
  p.add("entry-on", c.image(), [0, 16 * S, 0, 16 * S]);
  // cursor: a lit chevron
  c = new Canvas(16 * S, 20 * S);
  const chev = c.poly(pts(S, 3, 3, 7, 3, 13, 10, 7, 17, 3, 17, 9, 10));
  c.paint(c.blur(chev, 3 * S), NE.yellow, 0.9);
  c.paint(chev, NE.yellow);
  p.add("cursor", c.image(), 0);

  const glowBar = (w: number, h: number, y0: number, y1: number, col: string) => {
    const c = new Canvas(w, h);
    const m = c.mask((_x, y) => (y > y0 && y < y1 ? 1 : 0));
    c.paint(c.blur(m, 3 * S), col, 0.8);
    c.paint(m, col);
    return c.image();
  };
  p.add("tab-on", glowBar(24 * S, 24 * S, 19 * S, 21 * S, NE.cyan), [0, 4 * S, 6 * S, 4 * S]);
  p.add("track", glowBar(24 * S, 16 * S, 7.25 * S, 8.75 * S, NE["cyan-dk"]), [0, 4 * S, 0, 4 * S]);
  p.add("range", glowBar(24 * S, 16 * S, 6.75 * S, 9.25 * S, NE.cyan), [0, 4 * S, 0, 4 * S]);
  c = new Canvas(24 * S, 24 * S);
  const ring = c.poly(pts(S, 12, 4, 20, 12, 12, 20, 4, 12));
  const hole = c.poly(pts(S, 12, 7, 17, 12, 12, 17, 7, 12));
  c.paint(c.blur(ring, 3 * S), NE.cyan, 0.9);
  c.paint(sub(ring, hole), NE.cyan);
  c.paint(hole, NE.black);
  p.add("thumb", c.image(), 0);
  p.add("bar", linePiece(32 * S, 18 * S, 4 * S, 3 * S, NE.cyan, { lw: 1 * S, glowR: 3 * S, glowA: 0.5 }), 7 * S);

  const check = (on: boolean) => {
    const c = new Canvas(24 * S, 24 * S);
    const e = edge(c, c.rrect(5 * S, 5 * S, 19 * S, 19 * S, 0, 2 * S), 1.5 * S);
    c.paint(c.blur(e, 3 * S), NE.cyan, 0.7);
    c.paint(e, NE.cyan);
    if (on) {
      const inner = c.rrect(8.5 * S, 8.5 * S, 15.5 * S, 15.5 * S, 0);
      c.paint(c.blur(inner, 2 * S), NE.cyan, 0.9);
      c.paint(inner, NE.cyan);
    }
    return c.image();
  };
  p.add("check", check(false), 0);
  p.add("check-on", check(true), 0);
  return p;
}

// ---------------------------------------------------------------------------
// candy: glossy toy plastic, fat pills with a deep lip and a soft shadow,
// a yellow pill under a white rim for the current choice (Fall Guys,
// Astro's Playroom)

const CA = {
  white: "#ffffff", lip: "#c3cae8", navy: "#2a1f5c", shadow: "#2a1f5c", pink: "#e0185f", "pink-lip": "#98093c",
  yellow: "#ffd23a", "yellow-lip": "#d68a00", red: "#d93a2c", "red-lip": "#8f1c12", lav: "#e4e7fb", "lav-lip": "#b9bfe6",
  groove: "#cdd2f0", "groove-dk": "#aab1e0",
};

function candy(): Pack {
  const p = new Pack("candy");
  const S = SCALE;
  const lip = 5 * S;
  const pill = (w: number, h: number, r: number, inset = 4 * S) => (c: Canvas) => c.rrect(inset, inset, w - inset, h - inset - lip - 2 * S, r);
  const w = 72 * S, h = 72 * S;
  p.add("panel", framed(w, h, pill(w, h, 22 * S, 6 * S), [CA.white, "#f3f4fe"], { lip: CA.lip, lipH: lip, soft: CA.shadow, softR: 6 * S, softDy: 4 * S, softA: 0.3 }), [30 * S, 30 * S, 34 * S, 30 * S]);
  const w2 = 44 * S, h2 = 44 * S;
  p.add("plate", framed(w2, h2, pill(w2, h2, 12 * S), CA.white, { lip: CA.lip, lipH: 4 * S, soft: CA.shadow, softR: 4 * S, softDy: 3 * S, softA: 0.25 }), [18 * S, 18 * S, 22 * S, 18 * S]);
  p.add("slot", framed(w2, h2, (c) => c.rrect(4 * S, 4 * S, w2 - 4 * S, h2 - 4 * S, 10 * S), [CA["groove-dk"], CA.groove]), 16 * S);
  const bw = 64 * S, bh = 36 * S;
  const sl = [16 * S, 18 * S, 20 * S, 18 * S];
  const btn = (face: Face, lipc: string, down = false, halo?: string) =>
    down
      ? framed(bw, bh, (c) => c.rrect(4 * S, 7 * S, bw - 4 * S, bh - 6 * S, 13 * S), face, { lip: lipc, lipH: 2 * S, gloss: "#ffffff40" })
      : framed(bw, bh, pill(bw, bh, 13 * S), face, { lip: lipc, lipH: lip, gloss: "#ffffff55", soft: CA.shadow, softR: 3 * S, softDy: 2 * S, softA: 0.25, halo, haloW: 3 * S });
  p.add("button", btn([CA.white, "#eef0fd"], CA.lip), sl);
  p.add("button-down", btn("#eef0fd", CA.lip, true), sl);
  p.add("button-primary", btn(["#f03a7a", CA.pink], CA["pink-lip"]), sl);
  p.add("button-primary-down", btn(CA.pink, CA["pink-lip"], true), sl);
  p.add("button-danger", btn(["#e85440", CA.red], CA["red-lip"]), sl);
  p.add("button-danger-down", btn(CA.red, CA["red-lip"], true), sl);
  p.add("entry", btn([CA.white, "#eef0fd"], CA.lip), sl);
  p.add("entry-on", btn(["#ffe066", CA.yellow], CA["yellow-lip"], false, CA.white), sl);
  p.add("tab", btn([CA.lav, "#d9ddf8"], CA["lav-lip"]), sl);
  p.add("tab-on", btn(["#f03a7a", CA.pink], CA["pink-lip"]), sl);
  const tw = 40 * S, th = 20 * S;
  const groove = (c: Canvas) => c.rrect(2 * S, 2 * S, tw - 2 * S, th - 2 * S, 8 * S);
  p.add("track", framed(tw, th, groove, [CA["groove-dk"], CA.groove]), [0, 10 * S, 0, 10 * S]);
  p.add("range", framed(tw, th, groove, ["#f03a7a", CA.pink], { gloss: "#ffffff59" }), [0, 10 * S, 0, 10 * S]);
  p.add("thumb", framed(32 * S, 32 * S, (c) => c.ellipse(4 * S, 3 * S, 28 * S, 25 * S), [CA.white, "#eef0fd"], { lip: CA.lip, lipH: 3 * S, gloss: "#ffffff80", soft: CA.shadow, softR: 2.5 * S, softDy: 2 * S, softA: 0.35 }), 0);
  p.add("bar", framed(tw, 24 * S, (c) => c.rrect(2 * S, 2 * S, tw - 2 * S, 20 * S, 9 * S), CA.navy + "d9", { lip: CA.lip, lipH: 2 * S }), [0, 11 * S, 0, 11 * S]);

  const check = (on: boolean) => {
    const sh = (c: Canvas) => c.rrect(4 * S, 3 * S, 26 * S, 24 * S, 7 * S);
    if (!on) return framed(30 * S, 30 * S, sh, CA.white, { lip: CA.lip, lipH: 3 * S, line: CA.lip, lineW: 1.5 * S });
    const c = new Canvas(30 * S, 30 * S);
    const m = sh(c);
    c.paint(shift(c, m, 0, 3 * S), CA["pink-lip"]);
    c.paint(m, CA.pink);
    c.paint(c.poly(pts(S, 9, 13, 12, 11, 14, 15, 20.5, 7.5, 23, 10, 14, 20)), CA.white);
    return c.image();
  };
  p.add("check", check(false), 0);
  p.add("check-on", check(true), 0);
  return p;
}

// ---------------------------------------------------------------------------
// cartoon: a thick warm-dark outline round everything, glossy coloured
// buttons with a lip, a cream panel under a coloured header (Brawl Stars,
// Castle Crashers, Rayman Legends)

const CT: Record<string, string> = {
  ink: "#22142e", cream: "#fff6e3", "cream-dk": "#f1dfbd", "cream-lip": "#d9bd8c",
  blue: "#2a6fe0", "blue-hi": "#58a2ff", "blue-lip": "#173f99",
  yellow: "#ffc61a", "yellow-hi": "#ffe86a", "yellow-lip": "#c47b00",
  red: "#d42c24", "red-hi": "#ff6f5c", "red-lip": "#8c130f",
  orange: "#ff7a1a", "orange-hi": "#ffb04a", "orange-lip": "#b84a00",
  green: "#2fae45", "green-hi": "#7ee36a", "green-lip": "#1b7230",
  slate: "#a8b6d6", "slate-hi": "#c9d4ec", "slate-lip": "#6f7fa6", groove: "#3a2f52",
};

function cartoon(): Pack {
  const p = new Pack("cartoon");
  const S = SCALE;
  const ow = 3 * S;
  const ink = CT.ink!;
  const rr = (w: number, h: number, r: number, inset: number, lipH: number) => (c: Canvas) => c.rrect(inset, inset, w - inset, h - inset - lipH, r);
  const w = 64 * S, h = 64 * S;
  p.add("panel", framed(w, h, rr(w, h, 16 * S, 5 * S, 6 * S), [CT.cream!, CT["cream-dk"]!], { ink, inkW: ow, lip: CT["cream-lip"], lipH: 5 * S, soft: ink, softR: 3 * S, softDy: 3 * S, softA: 0.35 }), [24 * S, 24 * S, 28 * S, 24 * S]);
  const w2 = 40 * S, h2 = 40 * S;
  p.add("plate", framed(w2, h2, rr(w2, h2, 10 * S, 4 * S, 4 * S), CT.cream!, { ink, inkW: 2.5 * S, lip: CT["cream-lip"], lipH: 3 * S }), [15 * S, 15 * S, 18 * S, 15 * S]);
  p.add("slot", framed(w2, h2, rr(w2, h2, 8 * S, 4 * S, 0), [CT["cream-lip"]!, CT["cream-dk"]!], { ink, inkW: 2 * S }), 13 * S);
  const bw = 64 * S, bh = 36 * S;
  const sl = [14 * S, 18 * S, 19 * S, 18 * S];
  const btn = (col: string, { down = false, halo = undefined as string | undefined, r = 11 * S } = {}) => {
    const hi = CT[col + "-hi"]!, face = CT[col]!, lipc = CT[col + "-lip"]!;
    return down
      ? framed(bw, bh, (c) => c.rrect(4 * S, 7 * S, bw - 4 * S, bh - 6 * S, r), [face, hi], { ink, inkW: ow, lip: lipc, lipH: 2 * S })
      : framed(bw, bh, rr(bw, bh, r, 4 * S, 5 * S), [hi, face], { ink, inkW: ow, lip: lipc, lipH: 5 * S, gloss: "#ffffff4d", halo, haloW: 3 * S });
  };
  p.add("button", btn("blue"), sl);
  p.add("button-down", btn("blue", { down: true }), sl);
  p.add("button-primary", btn("yellow"), sl);
  p.add("button-primary-down", btn("yellow", { down: true }), sl);
  p.add("button-danger", btn("red"), sl);
  p.add("button-danger-down", btn("red", { down: true }), sl);
  p.add("entry", btn("blue"), sl);
  p.add("entry-on", btn("orange", { halo: "#ffffff" }), sl);
  p.add("header", btn("orange", { r: 9 * S }), sl);
  p.add("tab", btn("slate", { r: 9 * S }), sl);
  p.add("tab-on", btn("yellow", { r: 9 * S }), sl);
  const tw = 40 * S, th = 20 * S;
  const groove = (c: Canvas) => c.rrect(3 * S, 3 * S, tw - 3 * S, th - 3 * S, 7 * S);
  p.add("track", framed(tw, th, groove, CT.groove!, { ink, inkW: 2 * S }), [0, 10 * S, 0, 10 * S]);
  p.add("range", framed(tw, th, groove, [CT["green-hi"]!, CT.green!], { ink, inkW: 2 * S, gloss: "#ffffff4d" }), [0, 10 * S, 0, 10 * S]);
  p.add("thumb", framed(32 * S, 32 * S, (c) => c.ellipse(5 * S, 4 * S, 27 * S, 24 * S), ["#ffffff", "#e8e2f0"], { ink, inkW: 2.5 * S, lip: "#b9acd0", lipH: 3 * S, gloss: "#ffffff66" }), 0);
  p.add("bar", framed(tw, 24 * S, (c) => c.rrect(3 * S, 3 * S, tw - 3 * S, 21 * S, 8 * S), CT.groove!, { ink, inkW: 2.5 * S }), [0, 11 * S, 0, 11 * S]);

  const check = (on: boolean) => {
    const c = new Canvas(28 * S, 28 * S);
    const m = c.rrect(4 * S, 4 * S, 24 * S, 24 * S, 5 * S);
    c.paint(grow(c, m, 2.5 * S), ink);
    c.paint(m, on ? CT.green! : CT.cream!);
    if (on) {
      const tick = c.poly(pts(S, 8, 13, 11, 11, 13, 15, 19.5, 7, 22, 9.5, 13, 20));
      c.paint(grow(c, tick, 1.5 * S), ink);
      c.paint(tick, "#ffffff");
    }
    return c.image();
  };
  p.add("check", check(false), 0);
  p.add("check-on", check(true), 0);
  return p;
}

const PACKS: Record<string, () => Pack> = { pixel, comic, minimal, neon, candy, cartoon };

for (const name of process.argv.length > 2 ? process.argv.slice(2) : Object.keys(PACKS)) {
  const make = PACKS[name];
  if (!make) throw new Error(`ui-parts: no pack ${name} (${Object.keys(PACKS).join(", ")})`);
  make().write();
  console.log(`ui-parts: ${name}`);
}
