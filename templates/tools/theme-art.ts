// Reference renderer for theme nine-slice art. The cook's ui_art.rs keeps
// the same f64 operation order; --parity regenerates its digest fixture.
//   bun templates/tools/theme-art.ts SPEC.json OUT_DIR
//   bun templates/tools/theme-art.ts --sheet SPEC.json... OUT.png
//   bun templates/tools/theme-art.ts --parity [repo]
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { encodePngBest, type Image } from "../../packages/engine/src/node/png.ts";

type RGBA = number[];
type Box = [number, number, number, number];
type Ref = string | { mix: [Ref, Ref, number] };
// Specs are author JSON programs, with each layer owning its named parameters.
type Layer = Record<string, any>;
type Spec = {
  scale?: number;
  palette: Record<string, string | string[]>;
  material: Layer;
  shape: Layer;
  ornament?: Record<string, Layer>;
  panel?: Layer;
  divider?: Ref;
  badge_rim?: Ref;
};
type Field = Float64Array;
const clamp = (v: number) => Math.min(Math.max(v, 0), 1);
const ramp = (v: number) => {
  const t = clamp(v);
  return t * t * (3 - 2 * t);
};
const mix = (a: RGBA, b: RGBA, t: number) => a.map((v, i) => v * (1 - t) + b[i]! * t);
const hexc = (h: string): RGBA => [0, 2, 4, 6].map((i) => (i === 6 && h.length < 9 ? 1 : parseInt(h.slice(i + 1, i + 3), 16) / 255));
const WHITE = hexc("#ffffff"),
  BLACK = hexc("#000000");
class Colors {
  top: RGBA = [];
  bottom: RGBA = [];
  constructor(readonly palette: Spec["palette"]) {}
  get(ref: Ref): RGBA {
    if (typeof ref === "object") {
      const [a, b, t] = ref.mix;
      return mix(this.get(a), this.get(b), t);
    }
    if (ref === "top") return this.top;
    if (ref === "bottom") return this.bottom;
    if (ref.startsWith("#")) return hexc(ref);
    const v = this.palette[ref]!;
    return hexc(typeof v === "string" ? v : v[0]!);
  }
  pair(role: string): [RGBA, RGBA] {
    const p = this.palette[role] as string[];
    return [hexc(p[0]!), hexc(p[1]!)];
  }
}
class Canvas {
  insets: Box;
  w: number;
  h: number;
  nx: number;
  ny: number;
  cols: number[];
  rows: number[];
  rgba: Float64Array;
  tiled = false;
  constructor(
    readonly S: number,
    insets: Box,
    readonly band = [4, 4],
  ) {
    this.insets = insets.map(Math.ceil) as Box;
    const [t, r, b, l] = this.insets;
    this.w = l + r;
    this.h = t + b;
    this.nx = this.w * S + 1;
    this.ny = this.h * S + 1;
    const axis = (n: number, at: number, band: number) => [
      ...Array.from({ length: at * S }, (_, i) => i),
      ...Array(band * S).fill(at * S),
      ...Array.from({ length: (n - at) * S }, (_, i) => at * S + 1 + i),
    ];
    this.cols = axis(this.w, l, band[0]!);
    this.rows = axis(this.h, t, band[1]!);
    this.rgba = new Float64Array(this.cols.length * this.rows.length * 4);
  }
  field(f: (x: number, y: number, i: number) => number): Field {
    return Float64Array.from({ length: this.nx * this.ny }, (_, i) => f((i % this.nx) / this.S, Math.floor(i / this.nx) / this.S, i));
  }
  over(color: RGBA | ((x: number, y: number, i: number) => RGBA), alpha: (x: number, y: number, i: number) => number, whole = false): void {
    const w = this.cols.length;
    for (let j = 0; j < this.rows.length; j++)
      for (let k = 0; k < w; k++) {
        const x = this.cols[k]! / this.S,
          y = this.rows[j]! / this.S,
          i = this.rows[j]! * this.nx + this.cols[k]!;
        const px = whole ? (k < (this.insets[3] + this.band[0]!) * this.S ? k : k + 1) / this.S : x;
        const py = whole ? (j < (this.insets[0] + this.band[1]!) * this.S ? j : j + 1) / this.S : y;
        const c = typeof color === "function" ? color(px, py, i) : color;
        const a = clamp(alpha(px, py, i)) * c[3]!,
          p = (j * w + k) * 4,
          ba = this.rgba[p + 3]!;
        const oa = a + ba * (1 - a);
        for (let n = 0; n < 3; n++) this.rgba[p + n] = (c[n]! * a + this.rgba[p + n]! * ba * (1 - a)) / Math.max(oa, 1e-6);
        this.rgba[p + 3] = oa;
      }
  }
  image(): Image {
    return { width: this.cols.length, height: this.rows.length, rgba: Uint8Array.from(this.rgba, (v) => Math.floor(clamp(v) * 255 + 0.5)) };
  }
}
function rounded(x: number, y: number, box: Box, radius: number): number {
  const [x0, y0, x1, y1] = box,
    hw = (x1 - x0) / 2,
    hh = (y1 - y0) / 2,
    r = Math.min(radius, hw, hh);
  const qx = Math.abs(x - (x0 + x1) / 2) - hw + r,
    qy = Math.abs(y - (y0 + y1) / 2) - hh + r;
  const mx = Math.max(qx, 0),
    my = Math.max(qy, 0);
  return Math.sqrt(mx * mx + my * my) + Math.min(Math.max(qx, qy), 0) - r;
}
function hash01(x: number, y: number, seed: number): number {
  let h = (Math.imul(x, 374761393) + Math.imul(y, 668265263) + Math.imul(seed, 2246822519)) >>> 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177) >>> 0;
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
function noise(x: number, y: number, cell: number, seed: number): number {
  const u = x / cell,
    v = y / cell,
    ix = Math.floor(u),
    iy = Math.floor(v);
  let fx = u - ix,
    fy = v - iy;
  fx = fx * fx * (3 - 2 * fx);
  fy = fy * fy * (3 - 2 * fy);
  const a = hash01(ix, iy, seed),
    b = hash01(ix + 1, iy, seed),
    c = hash01(ix, iy + 1, seed),
    d = hash01(ix + 1, iy + 1, seed);
  const top = a + (b - a) * fx,
    bot = c + (d - c) * fx;
  return top + (bot - top) * fy;
}
class Shape {
  family: string;
  step: number;
  constructor(readonly spec: Layer) {
    this.family = spec.family ?? "rounded";
    this.step = spec.step ?? 1;
  }
  radius(role: string): number {
    return this.spec.radius?.[role] ?? (role === "badge" ? 12.5 : (this.spec.radius?.control ?? 6));
  }
  cut(role: string): number {
    const c = this.spec.cut ?? 0;
    return typeof c === "object" ? (c[role] ?? c.control ?? 0) : c;
  }
  corner(role: string): number {
    return Math.max(this.radius(role), ["cut", "cut4"].includes(this.family) ? this.cut(role) : 0);
  }
  span(cv: Canvas): number {
    return this.family === "pixel" ? Math.floor(this.step * cv.S + 0.5) : 1;
  }
  whole(w: number): number {
    return this.family === "pixel" ? Math.ceil(w / this.step) * this.step : w;
  }
  dist(cv: Canvas, box: Box, role: string, cut = 1): Field {
    return cv.field((x, y) => {
      if (this.family === "pixel") {
        x = (Math.floor(x / this.step) + 0.5) * this.step;
        y = (Math.floor(y / this.step) + 0.5) * this.step;
      }
      const r = this.radius(role),
        fam = this.family;
      let d = rounded(x, y, box, fam === "cut" ? Math.min(r, 2) : fam === "cut4" ? 0 : r);
      if (fam === "cut" || fam === "cut4") {
        const [x0, y0, x1, y1] = box,
          cx = (x0 + x1) / 2,
          cy = (y0 + y1) / 2;
        for (const c of fam === "cut" ? ["tl", "br"] : ["tl", "tr", "br", "bl"]) {
          const sx = c.includes("l") ? -1 : 1,
            sy = c.includes("t") ? -1 : 1;
          d = Math.max(d, (sx * (x - cx) + sy * (y - cy) - ((x1 - x0) / 2 + (y1 - y0) / 2 - this.cut(role) * cut)) * 0.7071067811865476);
        }
      }
      if (fam === "pixel") return (Math.floor(d / this.step) + 0.5) * this.step;
      if (fam === "brush") d += (noise(x, y, 5, this.spec.seed ?? 7) - 0.5) * 2 * (this.spec.wobble ?? 0);
      return d;
    });
  }
}
const cov = (cv: Canvas, d: number) => clamp(0.5 - d * cv.S);
const gradient = (top: RGBA, bottom: RGBA, y: number, y0: number, y1: number) =>
  mix(top, bottom, clamp((y - y0) / Math.max(y1 - y0, 1e-6)));
function lit(cv: Canvas, d: Field, k = 1): Field {
  return cv.field((_, __, i) => {
    const x = i % cv.nx,
      y = Math.floor(i / cv.nx),
      hx = Math.min(x + k, cv.nx - 1),
      lx = Math.max(x - k, 0),
      hy = Math.min(y + k, cv.ny - 1),
      ly = Math.max(y - k, 0);
    const gx = (d[y * cv.nx + hx]! - d[y * cv.nx + lx]!) / (hx - lx),
      gy = (d[hy * cv.nx + x]! - d[ly * cv.nx + x]!) / (hy - ly);
    const n = Math.sqrt(gx * gx + gy * gy) + 1e-9;
    return clamp(0.5 + 0.5 * ((0 - gy / n) * 0.92 - (gx / n) * 0.38));
  });
}
const litStep = (v: number) => ramp((v - 0.35) / 0.3);
function blur(cv: Canvas, a: Field, sigma: number): Field {
  if (sigma <= 0) return a;
  const r = Math.floor((Math.sqrt(4 * sigma * sigma + 1) - 1) / 2 + 0.5);
  for (const axis of [1, 0])
    for (let pass = 0; pass < 3; pass++) {
      const src = a;
      a = cv.field((_, __, i) => {
        const x = i % cv.nx,
          y = Math.floor(i / cv.nx);
        let sum = 0;
        for (let k = -r; k <= r; k++) {
          const xx = axis === 1 ? x + k : x,
            yy = axis === 0 ? y + k : y;
          if (xx >= 0 && xx < cv.nx && yy >= 0 && yy < cv.ny) sum += src[yy * cv.nx + xx]!;
        }
        return sum / (2 * r + 1);
      });
    }
  return a;
}
function grain(x: number, y: number, g: Layer, box: Box): number {
  const cell = g.cell ?? 16,
    seed = g.seed ?? 1;
  switch (g.kind) {
    case "noise":
      return noise(x, y, cell, seed) * 2 - 1;
    case "brushed":
      return noise(0.5, y, cell, seed) * 2 - 1;
    case "wood": {
      const t = y / cell + noise(0.5, y, cell * 3, seed) * 1.5,
        tri = Math.abs(t - Math.floor(t) - 0.5) * 2;
      return tri * tri * (3 - 2 * tri) * 2 - 1;
    }
    case "weave":
      return (Math.abs(x / cell - Math.floor(x / cell) - 0.5) - Math.abs(y / cell - Math.floor(y / cell) - 0.5)) * 2;
    case "stone":
      return (noise(x, y, cell, seed) * 0.65 + noise(x, y, cell * 0.37, seed + 1) * 0.35) * 2 - 1;
    case "wash": {
      const t = clamp((y - box[1]) / Math.max(box[3] - box[1], 1e-6));
      return noise(x, y, cell, seed) * 2 - 1 + (-0.3 * (1 - t) + 0.3 * t);
    }
    default:
      throw new Error(`unknown grain ${g.kind}`);
  }
}
function surface(cv: Canvas, C: Colors, M: Layer, d: Field, box: Box, sunk: boolean, k: number): Field {
  const [x0, y0, x1, y1] = box;
  let fd = d;
  if (M.rim) {
    const r = M.rim,
      L = lit(cv, d, k);
    cv.over(
      (_, __, i) => mix(C.get(r.lo), C.get(r.hi), L[i]!),
      (_, __, i) => cov(cv, d[i]!),
    );
    fd = d.map((v) => v + r.width);
  }
  const f = M.face ?? {},
    top = C.top.slice(),
    bottom = C.bottom.slice();
  top[3]! *= f.alpha ?? 1;
  bottom[3]! *= f.alpha ?? 1;
  cv.over(
    (_, y) =>
      f.split == null
        ? gradient(top, bottom, y, y0, y1)
        : y < y0 + (y1 - y0) * f.split
          ? top
          : gradient(mix(top, bottom, f.split), bottom, y, y0 + (y1 - y0) * f.split, y1),
    (_, __, i) => cov(cv, fd[i]!),
  );
  const g = M.grain;
  if (g) {
    cv.over(C.get(g.light ?? "#ffffff"), (x, y, i) => cov(cv, fd[i]!) * clamp(grain(x, y, g, box)) * g.strength);
    cv.over(C.get(g.color ?? "#000000"), (x, y, i) => cov(cv, fd[i]!) * clamp(0 - grain(x, y, g, box)) * g.strength);
  }
  const e = M.edge;
  if (e) cv.over(C.get(e.color), (_, __, i) => cov(cv, fd[i]!) * clamp(1 + (fd[i]! + 0.5) / e.width) * e.alpha);
  const bv = M.bevel;
  if (bv) {
    const L = lit(cv, fd, k),
      ring = (i: number) => cov(cv, fd[i]!) - cov(cv, fd[i]! + bv.width);
    cv.over(WHITE, (_, __, i) => ring(i) * litStep(L[i]!) * bv.hi);
    cv.over(BLACK, (_, __, i) => ring(i) * (1 - litStep(L[i]!)) * bv.lo);
    cv.over(C.get(bv.inner ?? "top"), (_, __, i) => cov(cv, fd[i]! + bv.width) * (bv.inner_alpha ?? 0));
  }
  const gl = M.gloss;
  if (gl && !sunk) {
    const ins = gl.inset ?? 3.5,
      h = (y1 - y0) * gl.band;
    cv.over(WHITE, (x, y, i) => {
      const gb = rounded(x, y, [x0 + ins, y0 + ins * 0.7, x1 - ins, y0 + ins * 0.7 + h * 0.62], Math.max((gl.radius ?? 12) - ins, 2));
      const a = gradient(gl.alpha.slice(0, 1), gl.alpha.slice(1), y, y0 + ins * 0.7, y0 + h)[0]!;
      return cov(cv, gb) * a * cov(cv, fd[i]!);
    });
  }
  const tl = M.toplight;
  if (tl)
    cv.over(WHITE, (_, y, i) => (cov(cv, fd[i]!) - cov(cv, fd[i]! + (tl.width ?? 1))) * clamp((y0 + tl.depth - y) / tl.depth) * tl.alpha);
  const hl = M.hairline;
  if (hl) cv.over(C.get(hl.color ?? "#ffffff"), (_, __, i) => (cov(cv, fd[i]!) - cov(cv, fd[i]! + (hl.width ?? 1))) * hl.alpha);
  const sh = M.shade;
  if (sh && !sunk) {
    const fr = (y1 - y0) * sh.frac;
    cv.over(BLACK, (_, y, i) => cov(cv, fd[i]!) * clamp((y - (y1 - fr)) / fr) * sh.alpha);
  }
  const sk = M.sunk;
  if (sk && sunk) {
    cv.over(BLACK, (_, y, i) => cov(cv, fd[i]!) * clamp((y0 + sk.depth - y) / sk.depth) * sk.alpha);
    cv.over(WHITE, (_, y, i) => (cov(cv, d[i]!) - cov(cv, d[i]! + 1)) * clamp((y - (y1 - 10)) / 10) * (sk.lip ?? 0));
  }
  return fd;
}
function stroke(x: number, y: number, ax: number, ay: number, bx: number, by: number, w: number): number {
  const px = x - ax,
    py = y - ay,
    vx = bx - ax,
    vy = by - ay,
    t = clamp((px * vx + py * vy) / (vx * vx + vy * vy));
  const dx = px - vx * t,
    dy = py - vy * t;
  return clamp(0.5 + (w / 2 - Math.sqrt(dx * dx + dy * dy)) * 4);
}
function ornament(cv: Canvas, C: Colors, d: Field, orn: Layer, box: Box): void {
  const kind = orn.kind,
    col = C.get(orn.color ?? "orn"),
    a = orn.alpha ?? 1,
    size = orn.size ?? 10;
  const [x0, y0, x1, y1] = box,
    inset = orn.inset ?? 4;
  const inside = (i: number) => cov(cv, d[i]!),
    ring = (i: number, w: number) => cov(cv, d[i]! + inset) - cov(cv, d[i]! + inset + w);
  const corners = [
    [x0, y0, 1, 1],
    [x1, y0, -1, 1],
    [x1, y1, -1, -1],
    [x0, y1, 1, -1],
  ];
  if (kind === "hairline") cv.over(col, (_, __, i) => ring(i, 0.75) * a);
  else if (kind === "stitch") {
    const band = Math.max(cv.band[0]!, 1),
      period = band / Math.max(1, Math.floor(band / (orn.period ?? 3) + 0.5));
    cv.over(
      col,
      (x, y, i) => {
        const sx = (i % cv.nx) / cv.S,
          sy = Math.floor(i / cv.nx) / cv.S;
        const back = (sx - (x0 + x1) / 2) * (sy - (y0 + y1) / 2) > 0,
          t = (back ? x - y : x + y) / period;
        return ring(i, 0.9) * (t - Math.floor(t) < 0.55 ? 1 : 0) * a;
      },
      true,
    );
    cv.tiled = true;
  } else if (["brackets", "studs", "filigree", "runes", "brush-corner"].includes(kind)) {
    if (kind === "filigree" || kind === "runes")
      cv.over(col, (_, __, i) => ring(i, kind === "filigree" ? 0.75 : 0.6) * (kind === "filigree" ? 0.5 : 0.35) * a);
    for (const [cx, cy, sx, sy] of kind === "brush-corner" ? [corners[0]!, corners[2]!] : corners) {
      const local = (f: (x: number, y: number, i: number) => number) => (x: number, y: number, i: number) =>
        f((x - cx!) * sx!, (y - cy!) * sy!, i);
      if (kind === "brackets")
        cv.over(
          col,
          local(
            (x, y, i) =>
              ((x >= 3 && x < 3 + size && y >= 3 && y < 4.5) || (y >= 3 && y < 3 + size && x >= 3 && x < 4.5) ? 1 : 0) * inside(i) * a,
          ),
        );
      else if (kind === "studs") {
        const r = (x: number, y: number) => Math.sqrt((x - (cx! + sx! * 6.5)) ** 2 + (y - (cy! + sy! * 6.5)) ** 2);
        cv.over(BLACK, (x, y, i) => clamp(2.6 - r(x, y)) * 0.5 * inside(i));
        cv.over(col, (x, y, i) => clamp(2 - r(x, y)) * a * inside(i));
        cv.over(WHITE, (x, y, i) => clamp(0.9 - r(x + 0.6, y + 0.6)) * 0.7 * inside(i));
      } else if (kind === "filigree") {
        cv.over(
          col,
          local((x, y) => clamp(3.2 - (Math.abs(x - 9) + Math.abs(y - 9))) * a),
        );
        cv.over(
          col,
          local((x, y) => clamp(0.6 - Math.abs(Math.sqrt((x - 9) ** 2 + (y - 9) ** 2) - 6.5)) * (x < 9 || y < 9 ? 1 : 0) * 0.9 * a),
        );
      } else if (kind === "runes")
        cv.over(
          col,
          local(
            (x, y, i) =>
              clamp(stroke(x, y, 7, 6, 7, 13, 1.1) + stroke(x, y, 7, 9, 10.5, 6.5, 1.1) + stroke(x, y, 7, 9, 10.5, 12, 1.1)) *
              a *
              inside(i),
          ),
        );
      else
        cv.over(
          col,
          local(
            (x, y, i) =>
              stroke(x, y, 4, 4 + size, 4 + size, 4, 0.5 + 3 * clamp(1 - (x + y - 6) / (2 * size))) *
              clamp((x + y - 4) * 2) *
              a *
              inside(i),
          ),
        );
    }
  } else if (kind === "notch")
    cv.over(col, (x, y, i) => (x - (x0 + 3) + y - (y0 + 3) < size * 0.7 && x >= x0 + 3 && y >= y0 + 3 ? 1 : 0) * cov(cv, d[i]! + 1) * a);
  else if (kind === "bar") {
    const ins = Math.min(size, (y1 - y0) * 0.3);
    cv.over(
      col,
      (x, y) =>
        clamp((x - (x0 + 2.5)) * 4 + 0.5) *
        clamp((x0 + 4 - x) * 4 + 0.5) *
        clamp((y - (y0 + ins)) * 4 + 0.5) *
        clamp((y1 - ins - y) * 4 + 0.5) *
        a,
    );
  } else if (kind === "tab")
    cv.over(
      col,
      (x, y) =>
        clamp((x - (x0 + 10)) * 4 + 0.5) *
        clamp((x0 + 10 + size * 2.4 - x) * 4 + 0.5) *
        clamp((y - y0) * 4 + 0.5) *
        clamp((y0 + 1.5 - y) * 4 + 0.5) *
        a,
    );
  else throw new Error(`unknown ornament ${kind}`);
}
function piece(
  cv: Canvas,
  C: Colors,
  M: Layer,
  shape: Shape,
  role: string,
  box: Box,
  face: [RGBA, RGBA],
  lip = 0,
  sunk = false,
  drop = true,
  orn?: Layer,
  cut = 1,
): Field {
  [C.top, C.bottom] = face;
  let body = shape.dist(cv, box, role, cut);
  const dr = M.drop;
  if (drop && dr) {
    const dy = Math.floor(dr.dy * cv.S + 0.5);
    const shifted = cv.field((_, __, i) => (Math.floor(i / cv.nx) >= dy ? cov(cv, body[i - dy * cv.nx]!) : 0));
    const a = blur(cv, shifted, dr.blur * cv.S);
    cv.over(C.get(dr.color ?? "shadow"), (_, __, i) => a[i]! * dr.alpha);
  }
  const gw = M.glow;
  if (gw && drop) {
    const a = blur(
      cv,
      body.map((v) => cov(cv, v)),
      gw.blur * cv.S,
    );
    cv.over(C.get(gw.color), (_, __, i) => a[i]! * gw.alpha * (1 - cov(cv, body[i]!)));
  }
  if (M.outline) cv.over(C.get(M.outline.color ?? "outline"), (_, __, i) => cov(cv, body[i]! - M.outline.width));
  let fbox = box;
  if (lip > 0) {
    const lp = M.lip;
    cv.over(
      (_, y) => gradient(C.get(lp.top), C.get(lp.bottom), y, box[3] - lip - 2, box[3]),
      (_, __, i) => cov(cv, body[i]!),
    );
    fbox = [box[0], box[1], box[2], box[3] - lip];
    body = shape.dist(cv, fbox, role, cut);
  }
  const fd = surface(cv, C, M, body, fbox, sunk, shape.span(cv));
  if (orn && orn.kind !== "none") ornament(cv, C, fd, orn, fbox);
  return fd;
}

export function draw(spec: Spec): { name: string; image: Image; entry: Layer }[] {
  const S = spec.scale ?? 4,
    C = new Colors(spec.palette),
    M = spec.material,
    shape = new Shape(spec.shape);
  const keys = "drop glow outline lip rim face grain edge bevel gloss toplight hairline shade sunk hover press about".split(" ");
  for (const k of Object.keys(M)) if (!keys.includes(k)) throw new Error(`material: unknown key ${k}`);
  const orns = spec.ornament ?? {},
    lip = M.lip?.height ?? 0,
    press = M.press?.dy ?? 2,
    hover = M.hover ?? { toward: "#ffffff", t: 0.12 };
  const out: ReturnType<typeof draw> = [];
  const put = (name: string, cv: Canvas, fill = true, tint = false) => {
    const entry: Layer = { src: `art/${name}.png`, slice: cv.insets.map((v) => v * S), width: cv.insets };
    if (!fill) entry.fill = false;
    if (tint) entry.tint = true;
    if (cv.tiled) entry.repeat = "tile";
    out.push({ name, image: cv.image(), entry });
  };
  const cr = shape.corner("control"),
    bi: Box = [Math.max(12, cr + 4), Math.max(14, cr + 6), Math.max(12, cr + 4) + lip, Math.max(14, cr + 6)],
    m = 2.5;
  for (const [name, role] of [
    ["button", "button"],
    ["button-primary", "primary"],
    ["button-danger", "danger"],
  ])
    for (const st of ["", "-hover", "-pressed"]) {
      const cv = new Canvas(S, bi);
      let [top, bottom] = C.pair(role!);
      if (st === "-hover") {
        const tw = C.get(hover.toward);
        top = mix(top, tw, hover.t);
        bottom = mix(bottom, tw, hover.t * 0.7);
      }
      const pressed = st === "-pressed",
        l2 = lip > 0 ? (pressed ? Math.max(lip - press, 1) : lip) : 0;
      const yt = m + (lip > 0 ? lip - l2 : pressed ? press * 0.4 : 0);
      piece(cv, C, M, shape, "control", [m, yt, cv.w - m, cv.h - m], [top, bottom], l2, false, !pressed, orns.button);
      put(`${name}${st}`, cv);
    }
  let cv = new Canvas(S, bi);
  piece(cv, C, M, shape, "control", [m, m, cv.w - m, cv.h - m], C.pair("disabled"), lip);
  put("button-disabled", cv);
  const flat = { ...M };
  delete flat.gloss;
  const pc = Math.max(24, shape.corner("panel") + 12),
    head = spec.panel?.header,
    hh = head?.height ?? 0;
  cv = new Canvas(S, [pc + hh, pc, pc + lip * 0.5, pc]);
  const fd = piece(cv, C, flat, shape, "panel", [3, 3, cv.w - 3, cv.h - 3], C.pair("panel"), lip * 0.5, false, true, orns.panel, 1.6);
  if (head) {
    const hc = C.get(head.color);
    cv.over(
      (_, y) => gradient(mix(hc, WHITE, head.lift ?? 0.15), hc, y, 3, 3 + hh),
      (_, y, i) => cov(cv, fd[i]!) * clamp((3 + hh - y) * S),
    );
    if (head.rule) cv.over(C.get(head.rule), (_, y, i) => cov(cv, fd[i]!) * clamp(1 - Math.abs(y - (3 + hh - 0.5)) / 0.9));
  }
  put("panel", cv);
  const pi = Math.max(12, Math.min(shape.corner("control"), 14) + 4);
  cv = new Canvas(S, [pi, pi + 2, pi, pi + 2]);
  piece(cv, C, flat, shape, "control", [2, 2, cv.w - 2, cv.h - 2 - lip * 0.5], C.pair("plate"), lip * 0.5, false, true, orns.plate);
  put("plate", cv);
  const ci = Math.max(9, shape.corner("chip") + 3);
  cv = new Canvas(S, [ci, ci + 1, ci, ci + 1]);
  piece(cv, C, M, shape, "chip", [2, 2, cv.w - 2, cv.h - 2 - lip * 0.4], C.pair("chip"), lip * 0.4, false, false, orns.chip, 0.6);
  put("chip", cv);
  const si = Math.max(16, shape.corner("slot") + 6);
  cv = new Canvas(S, [si, si, si, si]);
  piece(cv, C, M, shape, "slot", [2, 2, cv.w - 2, cv.h - 2], C.pair("slot"), 0, true, false, orns.slot, 0.8);
  put("slot", cv);
  const fi = Math.max(12, Math.min(shape.corner("control"), 10) + 4);
  for (const name of ["field", "field-focus", "field-invalid", "field-disabled", "field-readonly"]) {
    cv = new Canvas(S, [fi, fi, fi, fi]);
    const d = piece(
      cv,
      C,
      M,
      shape,
      "control",
      [2, 2, cv.w - 2, cv.h - 2],
      C.pair(name === "field-disabled" ? "disabled" : "field"),
      0,
      name !== "field-readonly",
      false,
      undefined,
      0.8,
    );
    if (name === "field-focus" || name === "field-invalid") {
      const rc = C.get(name === "field-focus" ? "accent" : "danger");
      if (name === "field-focus") cv.over(rc, (_, __, i) => (cov(cv, d[i]!) - cov(cv, d[i]! + 3)) * 0.35);
      else cv.over(rc, (_, __, i) => cov(cv, d[i]! + 2.5) - cov(cv, d[i]! + 3.5));
      cv.over(rc, (_, __, i) => cov(cv, d[i]!) - cov(cv, d[i]! + 1.5));
    }
    put(name, cv);
  }
  cv = new Canvas(S, [4, 64, 4, 64], [4, 0]);
  const ends = (x: number) => {
    const t = clamp(Math.min(x, cv.w - x) / 56);
    return t * t * (3 - 2 * t);
  };
  cv.over(C.get("shadow"), (x, y) => clamp(1 - Math.abs(y - 5.2) / 0.6) * ends(x) * 0.4);
  cv.over(C.get(spec.divider ?? "orn"), (x, y) => clamp(1 - Math.abs(y - 4) / 0.6) * ends(x));
  put("divider", cv);
  cv = new Canvas(S, [13, 14, 14, 14], [2, 0]);
  const ow = M.outline?.width ?? 1,
    bm = { ...M, outline: { color: spec.badge_rim ?? "#ffffff", width: shape.whole(Math.max(ow, 1.5)) } },
    bo = bm.outline.width + 0.5;
  piece(
    cv,
    C,
    bm,
    shape,
    "badge",
    [bo, bo, cv.w - bo, cv.h - bo],
    spec.palette.badge ? C.pair("badge") : [hexc("#d83834"), hexc("#a00c1a")],
    0,
    false,
    true,
    undefined,
    0.6,
  );
  put("badge", cv);
  const hard = !!M.bevel,
    metal = !!M.rim;
  cv = new Canvas(S, [si, si, si, si]);
  let body = shape.dist(cv, [2, 2, cv.w - 2, cv.h - 2], "slot", 0.8),
    L = lit(cv, body, shape.span(cv));
  const rw = Math.max(2.25, ow + 0.75),
    glen = hard ? 3 : 7;
  cv.over(
    (_, __, i) => {
      const g = hard ? 0.7 + 0.3 * litStep(L[i]!) : metal ? 0.6 + 0.4 * L[i]! : 0.78 + 0.22 * L[i]!;
      return [g, g, g, 1];
    },
    (_, __, i) => {
      const ring = cov(cv, body[i]!) - cov(cv, body[i]! + rw),
        inner = clamp(1 + (body[i]! + rw) / glen) * cov(cv, body[i]! + rw);
      return clamp(ring + inner * 0.45);
    },
  );
  put("slot-rim", cv, false, true);
  cv = new Canvas(S, [ci, ci + 1, ci, ci + 1]);
  body = shape.dist(cv, [1, 1, cv.w - 1, cv.h - 1], "chip", 0.6);
  L = lit(cv, body, shape.span(cv));
  cv.over([0.16, 0.16, 0.16, 1], (_, __, i) => cov(cv, body[i]!));
  cv.over(
    (_, __, i) => {
      const g = hard ? 0.17 + 0.36 * litStep(L[i]!) : metal ? 0.12 + 0.5 * L[i]! : 0.17 + 0.36 * L[i]!;
      return [g, g, g, 1];
    },
    (_, __, i) => cov(cv, body[i]! + 1),
  );
  cv.over(
    (_, y) => gradient([0.33, 0.33, 0.33, 1], [0.24, 0.24, 0.24, 1], y, 3, cv.h - 3),
    (_, __, i) => cov(cv, body[i]! + 1 + (hard ? M.bevel.width : 1)),
  );
  put("chip-bevel", cv, true, true);
  return out;
}
function digest(image: Image): string {
  const hdr = Buffer.alloc(8);
  hdr.writeUInt32LE(image.width);
  hdr.writeUInt32LE(image.height, 4);
  let h = 0xcbf29ce484222325n;
  for (const b of Buffer.concat([hdr, image.rgba])) h = BigInt.asUintN(64, (h ^ BigInt(b)) * 0x100000001b3n);
  return h.toString(16).padStart(16, "0");
}
const read = (p: string) => JSON.parse(readFileSync(p, "utf8"));
const args = process.argv.slice(2);
if (args[0] === "--parity") {
  const fixture: Record<string, Record<string, string>> = {},
    specs = read(join(import.meta.dir, "theme-art-specs.json"));
  for (const name of Object.keys(specs).sort())
    fixture[name] = Object.fromEntries(
      draw(specs[name])
        .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0))
        .map((p) => [p.name, digest(p.image)]),
    );
  const path = join(args[1] ?? ".", "crates/og-cook/tests/ui_art_parity.json");
  writeFileSync(path, JSON.stringify(fixture, null, 1) + "\n");
  console.log(path);
} else if (args[0] === "--sheet") {
  const rows = args.slice(1, -1).map((p) => draw(read(p))),
    pad = 8;
  const width = Math.max(...rows.map((r) => r.reduce((n, p) => n + p.image.width + pad, 0))) + pad;
  const height = rows.reduce((n, r) => n + Math.max(...r.map((p) => p.image.height)) + pad, 0) + pad;
  const rgba = new Uint8Array(width * height * 4);
  for (let i = 0; i < rgba.length; i += 4) rgba.set([128, 138, 148, 255], i);
  let y = pad;
  for (const row of rows) {
    let x = pad;
    for (const { image: im } of row) {
      for (let j = 0; j < im.height; j++)
        for (let i = 0; i < im.width; i++) {
          const src = (j * im.width + i) * 4,
            dst = ((y + j) * width + x + i) * 4,
            a = im.rgba[src + 3]! / 255;
          for (let k = 0; k < 3; k++) rgba[dst + k] = Math.round(im.rgba[src + k]! * a + rgba[dst + k]! * (1 - a));
        }
      x += im.width + pad;
    }
    y += Math.max(...row.map((p) => p.image.height)) + pad;
  }
  writeFileSync(args.at(-1)!, encodePngBest({ width, height, rgba }));
} else if (args.length === 2) {
  const out = args[1]!,
    entries: Layer = {};
  mkdirSync(join(out, "art"), { recursive: true });
  for (const { name, image, entry } of draw(read(args[0]!))) {
    writeFileSync(join(out, "art", `${name}.png`), encodePngBest(image));
    entries[name] = entry;
  }
  writeFileSync(join(out, "images.json"), JSON.stringify(entries, null, 1) + "\n");
} else throw new Error("expected SPEC.json OUT_DIR, --sheet SPEC.json... OUT.png, or --parity [repo]");
