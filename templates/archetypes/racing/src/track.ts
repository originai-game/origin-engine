// The circuit as numbers: the centre line, the road's edges, the sectors,
// the grid and the racing line the rivals drive. Pure arithmetic (+ - * /
// and sqrt, which every host rounds alike), no engine import: the same
// samples on every peer of a room, in every replay and in tools/.
//
// A closed centripetal Catmull-Rom spline through CONTROL, resampled every
// STEP metres of arc into SAMPLES. Each sample has its arc length, its
// point, its unit tangent and right, and its curvature (positive bends
// left). The racing line is an elastic band relaxed inside the road (each
// point pulled to its neighbours' middle, kept a car's width from the
// walls): the minimum-curvature line, wide on the way in, the apex on the
// inside, wide on the way out. Its speed comes from its curvature and the
// braking needed for what comes next.

/** The road's width, metres between the kerbs' outer edges. */
export const WIDTH = 13;
/** How far from the centre line a car's middle may go before it meets the barrier. */
export const WALL = WIDTH / 2 + 1.2;
/** Metres of arc between samples. */
export const STEP = 2;
/** Laps in a race. */
export const LAPS = 3;
/** Checkpoints a lap passes, as fractions of a lap (the line is the last). */
export const SECTORS = [1 / 3, 2 / 3] as const;

/**
 * The circuit's control points (x, z in metres), driven in this order:
 * the start straight runs north (-z), a fast right sweeper, the back
 * esses, the hairpin, the long left and the run back to the line.
 */
export const CONTROL: readonly (readonly [number, number])[] = [
  [0, 70],
  [0, 10],
  [0, -50],
  [8, -92],
  [38, -118],
  [80, -120],
  [112, -100],
  [118, -66],
  [100, -38],
  [104, -8],
  [132, 12],
  [150, 44],
  [140, 80],
  [108, 88],
  [86, 66],
  [70, 40],
  [44, 46],
  [38, 84],
  [36, 110],
  [28, 136],
  [8, 142],
  [-4, 124],
];

export interface Sample {
  /** Arc length from the start line, metres. */
  s: number;
  x: number;
  z: number;
  /** Unit tangent (the driving direction). */
  tx: number;
  tz: number;
  /** Unit right across the road. */
  rx: number;
  rz: number;
  /** Signed curvature, 1/m: positive bends left. */
  k: number;
}

function catmull(p0: readonly [number, number], p1: readonly [number, number], p2: readonly [number, number], p3: readonly [number, number], t: number): [number, number] {
  const knot = (a: readonly [number, number], b: readonly [number, number]) => Math.sqrt(Math.sqrt((b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2));
  const t0 = 0;
  const t1 = t0 + knot(p0, p1);
  const t2 = t1 + knot(p1, p2);
  const t3 = t2 + knot(p2, p3);
  const u = t1 + (t2 - t1) * t;
  const lerp = (a: readonly [number, number], b: readonly [number, number], ta: number, tb: number): [number, number] => {
    const w = (u - ta) / (tb - ta);
    return [a[0] + (b[0] - a[0]) * w, a[1] + (b[1] - a[1]) * w];
  };
  const a1 = lerp(p0, p1, t0, t1);
  const a2 = lerp(p1, p2, t1, t2);
  const a3 = lerp(p2, p3, t2, t3);
  const b1 = lerp(a1, a2, t0, t2);
  const b2 = lerp(a2, a3, t1, t3);
  return lerp(b1, b2, t1, t2);
}

function build(): Sample[] {
  const n = CONTROL.length;
  const dense: [number, number][] = [];
  for (let i = 0; i < n; i++) {
    const at = (j: number) => CONTROL[(j + n) % n]!;
    for (let k = 0; k < 64; k++) dense.push(catmull(at(i - 1), at(i), at(i + 1), at(i + 2), k / 64));
  }
  dense.push(dense[0]!);
  const lengths = [0];
  for (let i = 1; i < dense.length; i++) lengths.push(lengths[i - 1]! + Math.sqrt((dense[i]![0] - dense[i - 1]![0]) ** 2 + (dense[i]![1] - dense[i - 1]![1]) ** 2));
  const total = lengths[lengths.length - 1]!;
  const count = Math.round(total / STEP);
  const step = total / count;
  const pts: [number, number][] = [];
  let j = 0;
  for (let i = 0; i < count; i++) {
    const s = i * step;
    while (lengths[j + 1]! < s) j++;
    const w = (s - lengths[j]!) / (lengths[j + 1]! - lengths[j]!);
    pts.push([dense[j]![0] + (dense[j + 1]![0] - dense[j]![0]) * w, dense[j]![1] + (dense[j + 1]![1] - dense[j]![1]) * w]);
  }
  const out: Sample[] = [];
  for (let i = 0; i < count; i++) {
    const a = pts[(i - 1 + count) % count]!;
    const b = pts[(i + 1) % count]!;
    const dx = b[0] - a[0];
    const dz = b[1] - a[1];
    const len = Math.sqrt(dx * dx + dz * dz);
    out.push({ s: i * step, x: pts[i]![0], z: pts[i]![1], tx: dx / len, tz: dz / len, rx: -dz / len, rz: dx / len, k: 0 });
  }
  for (let i = 0; i < count; i++) {
    const a = out[(i - 1 + count) % count]!;
    const b = out[(i + 1) % count]!;
    // Left is (tz, -tx) seen from above with +y up: a tangent turning that way bends left.
    out[i]!.k = (a.tz * b.tx - a.tx * b.tz) / (2 * step);
  }
  return out;
}

export const SAMPLES: readonly Sample[] = build();
export const COUNT = SAMPLES.length;
/** The lap's length, metres. */
export const LENGTH = COUNT * (SAMPLES[1]!.s - SAMPLES[0]!.s);

/** The sample nearest a point, searched around a hint (the car's last one) when given. */
export function nearest(x: number, z: number, hint = -1): number {
  let best = 0;
  let bestD = Infinity;
  const from = hint < 0 ? 0 : hint - 12;
  const to = hint < 0 ? COUNT : hint + 13;
  for (let j = from; j < to; j++) {
    const i = ((j % COUNT) + COUNT) % COUNT;
    const p = SAMPLES[i]!;
    const d = (p.x - x) ** 2 + (p.z - z) ** 2;
    if (d < bestD) {
      bestD = d;
      best = i;
    }
  }
  return best;
}

/** Where a point is on the circuit: along (metres from the line, 0 to LENGTH) and across (metres right of the centre). */
export function locate(x: number, z: number, hint = -1): { index: number; along: number; across: number } {
  const i = nearest(x, z, hint);
  const p = SAMPLES[i]!;
  const dx = x - p.x;
  const dz = z - p.z;
  let along = p.s + dx * p.tx + dz * p.tz;
  if (along < 0) along += LENGTH;
  if (along >= LENGTH) along -= LENGTH;
  return { index: i, along, across: dx * p.rx + dz * p.rz };
}

/** A point on the circuit: `along` metres from the line, `across` right of the centre, and the heading there (radians, 0 along -z, growing to the left). */
export function pointAt(along: number, across: number): { x: number; z: number; tx: number; tz: number } {
  const u = (((along / LENGTH) % 1) + 1) % 1 * COUNT;
  const i = Math.floor(u) % COUNT;
  const w = u - Math.floor(u);
  const a = SAMPLES[i]!;
  const b = SAMPLES[(i + 1) % COUNT]!;
  const tx = a.tx + (b.tx - a.tx) * w;
  const tz = a.tz + (b.tz - a.tz) * w;
  const n = Math.sqrt(tx * tx + tz * tz);
  const x = a.x + (b.x - a.x) * w;
  const z = a.z + (b.z - a.z) * w;
  return { x: x - (tz / n) * across, z: z + (tx / n) * across, tx: tx / n, tz: tz / n };
}

/**
 * The racing line: each sample's offset right of the centre (metres), an
 * elastic band relaxed to its least curvature inside `margin` of the
 * walls, and the speed it can be driven at (m/s) with `grip` m/s² of
 * cornering and `braking` m/s² of braking, capped at `top`.
 */
export function racingLine(margin: number, grip: number, braking: number, top: number): { offset: Float64Array; speed: Float64Array } {
  const n = COUNT;
  const limit = WIDTH / 2 - margin;
  const off = new Float64Array(n);
  const px = new Float64Array(n);
  const pz = new Float64Array(n);
  for (let it = 0; it < 400; it++) {
    for (let i = 0; i < n; i++) {
      const p = SAMPLES[i]!;
      px[i] = p.x + p.rx * off[i]!;
      pz[i] = p.z + p.rz * off[i]!;
    }
    for (let i = 0; i < n; i++) {
      const p = SAMPLES[i]!;
      const a = (i - 1 + n) % n;
      const b = (i + 1) % n;
      const mx = (px[a]! + px[b]!) / 2;
      const mz = (pz[a]! + pz[b]!) / 2;
      const want = (mx - p.x) * p.rx + (mz - p.z) * p.rz;
      const o = off[i]! + (want - off[i]!) * 0.5;
      off[i] = o < -limit ? -limit : o > limit ? limit : o;
    }
  }
  for (let i = 0; i < n; i++) {
    const p = SAMPLES[i]!;
    px[i] = p.x + p.rx * off[i]!;
    pz[i] = p.z + p.rz * off[i]!;
  }
  const speed = new Float64Array(n);
  for (let i = 0; i < n; i++) {
    // The curvature of the line over a few samples either side (a circle through three points).
    const a = (i - 3 + n) % n;
    const b = (i + 3) % n;
    const ax = px[a]! - px[i]!, az = pz[a]! - pz[i]!;
    const bx = px[b]! - px[i]!, bz = pz[b]! - pz[i]!;
    const cx = px[b]! - px[a]!, cz = pz[b]! - pz[a]!;
    const cross = Math.abs(ax * bz - az * bx);
    const k = (2 * cross) / Math.sqrt((ax * ax + az * az) * (bx * bx + bz * bz) * (cx * cx + cz * cz));
    const v = k > 1e-6 ? Math.sqrt(grip / k) : top;
    speed[i] = v < top ? v : top;
  }
  // Braking: no sample faster than the next can be braked to (twice round the loop).
  for (let pass = 0; pass < 2; pass++) {
    for (let j = 2 * n - 1; j >= 0; j--) {
      const i = j % n;
      const next = speed[(i + 1) % n]!;
      const can = Math.sqrt(next * next + 2 * braking * STEP);
      if (speed[i]! > can) speed[i] = can;
    }
  }
  return { offset: off, speed };
}

/** The grid: slot `i`'s spot behind the line, two abreast, 7 m apart, staggered. */
export function gridSlot(i: number): { along: number; across: number } {
  const row = Math.floor(i / 2);
  return { along: LENGTH - 8 - row * 7 - (i % 2) * 3.5, across: i % 2 === 0 ? -2.6 : 2.6 };
}
