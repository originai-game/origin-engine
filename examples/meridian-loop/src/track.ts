// The Meridian Loop's circuit as numbers, shared by the game (progress,
// sectors, the reset, the autopilot) and tools/make-assets.ts (the road,
// walls and ground of models/track.glb, the mini-map). Pure arithmetic: no
// engine import, the same samples on every host and in the generator.
//
// A closed centripetal Catmull-Rom spline through CONTROL (x, z, bank in
// degrees), resampled every STEP metres of arc. The road is WIDTH wide,
// banked into each turn by the control points' bank (the outside edge
// raised), flat except the back straight, where a kicker ramp lifts it to
// the lip, the road stops for the gap, and a landing ramp brings it down.

export const WIDTH = 14;
export const STEP = 2;
export const WALL_HEIGHT = 1.1;
export const WALL_THICK = 0.6;
/** Laps in a run. */
export const LAPS = 3;

/** x, z (metres), bank (degrees, into the turn). Driven in this order. */
export const CONTROL: readonly (readonly [number, number, number])[] = [
  [0, -40, 0],
  [0, 20, 0],
  [2, 90, 0],
  [14, 140, 10],
  [48, 178, 16],
  [100, 192, 18],
  [152, 178, 16],
  [186, 140, 10],
  [196, 90, 0],
  [197, 30, 0],
  [197, -30, 0],
  [196, -90, 0],
  [188, -134, 14],
  [164, -162, 24],
  [128, -172, 24],
  [100, -158, 14],
  [88, -128, 6],
  [72, -104, 12],
  [46, -94, 12],
  [22, -96, 12],
  [6, -82, 10],
  [1, -62, 4],
];

/** The jump on the back straight, in metres along it from the ramp's foot (heading -z at x ≈ 197). */
export const JUMP = {
  /** z where the kicker starts rising. */
  rampZ: 64,
  /** Its length and the lip's height. */
  rampLength: 34,
  lip: 3.4,
  /** The gap after the lip, where there is no road. */
  gap: 15,
  /** The landing ramp: its top height and length down to the ground. */
  land: 2.2,
  landLength: 48,
} as const;

export interface Sample {
  /** Arc length from the start line, metres. */
  s: number;
  x: number;
  y: number;
  z: number;
  /** Unit tangent (with the slope). */
  tx: number;
  ty: number;
  tz: number;
  /** Unit right across the road (banked) and the road's normal. */
  rx: number;
  ry: number;
  rz: number;
  nx: number;
  ny: number;
  nz: number;
  /** Bank in radians (signed: positive raises the left edge). */
  bank: number;
  /** No road here (the jump's gap). */
  gap: boolean;
}

function catmull(p0: number, p1: number, p2: number, p3: number, t0: number, t1: number, t2: number, t3: number, t: number): number {
  const a1 = ((t1 - t) / (t1 - t0)) * p0 + ((t - t0) / (t1 - t0)) * p1;
  const a2 = ((t2 - t) / (t2 - t1)) * p1 + ((t - t1) / (t2 - t1)) * p2;
  const a3 = ((t3 - t) / (t3 - t2)) * p2 + ((t - t2) / (t3 - t2)) * p3;
  const b1 = ((t2 - t) / (t2 - t0)) * a1 + ((t - t0) / (t2 - t0)) * a2;
  const b2 = ((t3 - t) / (t3 - t1)) * a2 + ((t - t1) / (t3 - t1)) * a3;
  return ((t2 - t) / (t2 - t1)) * b1 + ((t - t1) / (t2 - t1)) * b2;
}

/** The spline densely: [x, z, bank°] every 1/64 of a segment. */
function dense(): [number, number, number][] {
  const n = CONTROL.length;
  const out: [number, number, number][] = [];
  for (let i = 0; i < n; i++) {
    const p = [CONTROL[(i + n - 1) % n]!, CONTROL[i]!, CONTROL[(i + 1) % n]!, CONTROL[(i + 2) % n]!];
    const k = [0, 0, 0, 0];
    for (let j = 1; j < 4; j++) k[j] = k[j - 1]! + Math.sqrt(Math.hypot(p[j]![0] - p[j - 1]![0], p[j]![1] - p[j - 1]![1]));
    for (let m = 0; m < 64; m++) {
      const t = k[1]! + ((k[2]! - k[1]!) * m) / 64;
      out.push([0, 1, 2].map((c) => catmull(p[0]![c]!, p[1]![c]!, p[2]![c]!, p[3]![c]!, k[0]!, k[1]!, k[2]!, k[3]!, t)) as [number, number, number]);
    }
  }
  return out;
}

/** The road's height along the back straight at z (0 elsewhere), and whether z is in the gap. */
export function jumpHeight(x: number, z: number): { y: number; gap: boolean } {
  if (x < 180) return { y: 0, gap: false };
  const d = JUMP.rampZ - z;
  if (d < 0) return { y: 0, gap: false };
  if (d <= JUMP.rampLength) {
    // A kicker: gentle at the foot, steepest at the lip.
    const u = d / JUMP.rampLength;
    return { y: JUMP.lip * (0.35 * u + 0.65 * u * u), gap: false };
  }
  const e = d - JUMP.rampLength;
  if (e < JUMP.gap) return { y: 0, gap: true };
  const f = e - JUMP.gap;
  if (f <= JUMP.landLength) {
    const u = f / JUMP.landLength;
    return { y: JUMP.land * (1 - u) * (1 - u * 0.4) / 1, gap: false };
  }
  return { y: 0, gap: false };
}

function build(): { samples: Sample[]; length: number } {
  const d = dense();
  // Arc length over the dense points, then resampled every STEP metres.
  const acc = [0];
  for (let i = 1; i <= d.length; i++) {
    const a = d[i - 1]!;
    const b = d[i % d.length]!;
    acc.push(acc[i - 1]! + Math.hypot(b[0] - a[0], b[1] - a[1]));
  }
  const total = acc[d.length]!;
  const count = Math.round(total / STEP);
  const step = total / count;
  const flat: { x: number; z: number; bank: number; s: number }[] = [];
  let j = 0;
  for (let i = 0; i < count; i++) {
    const s = i * step;
    while (acc[j + 1]! < s) j++;
    const u = (s - acc[j]!) / (acc[j + 1]! - acc[j]!);
    const a = d[j]!;
    const b = d[(j + 1) % d.length]!;
    flat.push({ x: a[0] + (b[0] - a[0]) * u, z: a[1] + (b[1] - a[1]) * u, bank: a[2] + (b[2] - a[2]) * u, s });
  }
  // The bank's sign from the heading's change over 16 m either side (a turn's side, not the spline's wiggle),
  // then the signed bank smoothed over 12 m, so the road twists evenly into and out of every bend.
  const heading = flat.map((c, i) => {
    const q = flat[(i + 1) % count]!;
    return Math.atan2(q.x - c.x, q.z - c.z);
  });
  const signed = flat.map((c, i) => {
    let dh = heading[(i + 8) % count]! - heading[(i + count - 8) % count]!;
    while (dh > Math.PI) dh -= 2 * Math.PI;
    while (dh < -Math.PI) dh += 2 * Math.PI;
    return ((c.bank * Math.PI) / 180) * (dh > 0 ? 1 : -1);
  });
  const banks = signed.map((_, i) => {
    let sum = 0;
    for (let d = -6; d <= 6; d++) sum += signed[(i + d + count) % count]!;
    return sum / 13;
  });
  const samples: Sample[] = [];
  for (let i = 0; i < count; i++) {
    const p = flat[(i + count - 1) % count]!;
    const c = flat[i]!;
    const q = flat[(i + 1) % count]!;
    const hp = jumpHeight(p.x, p.z).y;
    const h = jumpHeight(c.x, c.z);
    const hq = jumpHeight(q.x, q.z).y;
    let tx = q.x - p.x;
    let ty = hq - hp;
    let tz = q.z - p.z;
    const tl = Math.hypot(tx, ty, tz);
    tx /= tl;
    ty /= tl;
    tz /= tl;
    const bank = banks[i]!;
    // Flat right (heading x up = tangent x (0,1,0)), then turned about the tangent by the bank.
    let rx = -tz;
    let rz = tx;
    const rl = Math.hypot(rx, rz);
    rx /= rl;
    rz /= rl;
    // n0 = r x t (up-ish); rotate r towards n0 by bank.
    const n0x = 0 * tz - rz * ty;
    const n0y = rz * tx - rx * tz;
    const n0z = rx * ty - 0 * tx;
    const cb = Math.cos(bank);
    const sb = Math.sin(bank);
    const Rx = rx * cb + n0x * sb;
    const Ry = 0 * cb + n0y * sb;
    const Rz = rz * cb + n0z * sb;
    const nx = Ry * tz - Rz * ty;
    const ny = Rz * tx - Rx * tz;
    const nz = Rx * ty - Ry * tx;
    const flip = ny < 0 ? -1 : 1;
    // A banked road is lifted so its low edge stays on the ground.
    const lift = Math.abs(Math.sin(bank)) * WIDTH * 0.5;
    samples.push({ s: c.s, x: c.x, y: h.y + lift, z: c.z, tx, ty, tz, rx: Rx, ry: Ry, rz: Rz, nx: nx * flip, ny: ny * flip, nz: nz * flip, bank, gap: h.gap });
  }
  return { samples, length: total };
}

const BUILT = build();
/** The centre line every STEP metres, from the start line, in driving order. */
export const SAMPLES: readonly Sample[] = BUILT.samples;
/** The lap's length in metres. */
export const LENGTH = BUILT.length;

/** The start line: the sample nearest z = 0 on the start straight. */
export const START = SAMPLES.reduce((best, s, i) => (Math.abs(s.x) < 20 && Math.abs(s.z) < Math.abs(SAMPLES[best]!.z) ? i : best), 0);

/** The two split lines, as sample indices from START (sector 1 ends after the sweeper, sector 2 after the jump). */
export const SPLITS: readonly number[] = [0.36, 0.64].map((f) => Math.round(f * SAMPLES.length));

/** Index i counted from the start line (0 at the line). */
export function fromStart(i: number): number {
  return (i - START + SAMPLES.length) % SAMPLES.length;
}

/** The sample at k samples past the start line. */
export function at(k: number): Sample {
  return SAMPLES[(START + ((k % SAMPLES.length) + SAMPLES.length)) % SAMPLES.length]!;
}

/** The nearest sample to (x, z), searching `span` samples either side of `hint` (or everything when hint < 0). */
export function nearest(x: number, z: number, hint: number, span = 12): number {
  const n = SAMPLES.length;
  let best = 0;
  let bestD = Infinity;
  const from = hint < 0 ? 0 : hint - span;
  const to = hint < 0 ? n - 1 : hint + span;
  for (let k = from; k <= to; k++) {
    const i = ((k % n) + n) % n;
    const s = SAMPLES[i]!;
    const dd = (s.x - x) * (s.x - x) + (s.z - z) * (s.z - z);
    if (dd < bestD) {
      bestD = dd;
      best = i;
    }
  }
  return best;
}

/** Signed distance across the road from the centre at sample i (positive to the right). */
export function across(i: number, x: number, z: number): number {
  const s = SAMPLES[i]!;
  const l = Math.hypot(s.rx, s.rz) || 1;
  return ((x - s.x) * s.rx + (z - s.z) * s.rz) / l;
}

/** The mini-map's square in world metres (x right, z up the map), round the whole circuit with a margin. */
export const MAP = (() => {
  let minX = Infinity, maxX = -Infinity, minZ = Infinity, maxZ = -Infinity;
  for (const s of SAMPLES) {
    minX = Math.min(minX, s.x);
    maxX = Math.max(maxX, s.x);
    minZ = Math.min(minZ, s.z);
    maxZ = Math.max(maxZ, s.z);
  }
  const size = Math.max(maxX - minX, maxZ - minZ) + 40;
  return { minX: (minX + maxX) / 2 - size / 2, minZ: (minZ + maxZ) / 2 - size / 2, size };
})();

/** Where (x, z) falls on the mini-map, 0..1 across and down. */
export function mapPoint(x: number, z: number): { x: number; y: number } {
  return { x: (x - MAP.minX) / MAP.size, y: (MAP.minZ + MAP.size - z) / MAP.size };
}
