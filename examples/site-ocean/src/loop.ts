// The hero's 90 s loop, as data: the hour, the storm and the camera at each
// moment of it. It opens at golden hour (the poster), a storm rolls in over
// the sunset and breaks into lightning, clears into a moonlit night (the
// camera lifts to the moon and the stars), then dawn behind the headland,
// a bright noon, and the afternoon sinks back to the opening frame. Every
// value is a function of the loop's time alone, so the same moment always
// looks the same, and the end meets the start.

export const LOOP = 90;

type Key = [time: number, value: number];

const smooth = (t: number) => t * t * (3 - 2 * t);

/** Eased between keys (smoothstep per span: each key is a held beat), held past the ends. */
function eased(keys: readonly Key[], t: number): number {
  if (t <= keys[0]![0]) return keys[0]![1];
  for (let i = 1; i < keys.length; i++) {
    const [t1, v1] = keys[i]!;
    if (t <= t1) {
      const [t0, v0] = keys[i - 1]!;
      return v0 + (v1 - v0) * smooth((t - t0) / (t1 - t0));
    }
  }
  return keys[keys.length - 1]![1];
}

/** The hour, unwrapped (it gains 24 over the loop): dusk 17:37 at 0 with the sun 5° up. */
const HOUR: Key[] = [
  [0, 17.62],
  [10, 17.8],
  [18, 18.15],
  [30, 19.5],
  [42, 22.0],
  [54, 27.6],
  [62, 29.0],
  [68, 30.1],
  [76, 33.0],
  [82, 36.0],
  [90, 41.62],
];

/** The storm, 0 fair to 1 a gale with lightning. */
const STORM: Key[] = [
  [0, 0],
  [12, 0],
  [22, 0.6],
  [28, 1],
  [38, 1],
  [46, 0.15],
  [52, 0],
  [90, 0],
];

export function hourAt(t: number): number {
  return eased(HOUR, t) % 24;
}

export function stormAt(t: number): number {
  return eased(STORM, t);
}

/** A camera key: where the eye is, where it looks (yaw right of -z, pitch up, degrees), its fov. */
interface Shot {
  t: number;
  eye: [number, number, number];
  yaw: number;
  pitch: number;
  fov: number;
}

const SHOTS: Shot[] = [
  { t: 0, eye: [0, 2.2, 0], yaw: -3, pitch: 2.5, fov: 40 },
  { t: 10, eye: [2, 2.3, -3], yaw: 3, pitch: 2.8, fov: 40 },
  { t: 20, eye: [6, 3.4, -2], yaw: 10, pitch: 3.4, fov: 42 },
  { t: 30, eye: [9, 5.5, 2], yaw: 16, pitch: 4, fov: 44 },
  { t: 40, eye: [8, 4.5, 4], yaw: 12, pitch: 4.5, fov: 44 },
  { t: 48, eye: [5, 3, 3], yaw: 13, pitch: 12, fov: 44 },
  { t: 54, eye: [4, 2.6, 2], yaw: 18, pitch: 26, fov: 46 },
  { t: 60, eye: [2, 2.4, 1], yaw: 4, pitch: 7, fov: 42 },
  { t: 68, eye: [-2, 2.3, 0], yaw: -14, pitch: 3, fov: 40 },
  { t: 76, eye: [-3, 2.2, -1], yaw: -16, pitch: 2.6, fov: 40 },
  { t: 82, eye: [-2, 2.2, 0], yaw: -9, pitch: 2.5, fov: 40 },
];

/** Catmull-Rom through the shots, closed over the loop. */
export function shotAt(t: number): { eye: [number, number, number]; yaw: number; pitch: number; fov: number } {
  const n = SHOTS.length;
  let i = n - 1;
  for (let k = 0; k < n; k++) if (SHOTS[k]!.t <= t) i = k;
  const key = (k: number) => SHOTS[((k % n) + n) % n]!;
  const time = (k: number) => key(k).t + Math.floor(k / n) * LOOP;
  const t0 = time(i);
  const t1 = time(i + 1) <= t0 ? time(i + 1) + LOOP : time(i + 1);
  const u = (t - t0) / (t1 - t0);
  const cr = (a: number, b: number, c: number, d: number) =>
    0.5 * (2 * b + (-a + c) * u + (2 * a - 5 * b + 4 * c - d) * u * u + (-a + 3 * b - 3 * c + d) * u * u * u);
  const p = key(i - 1);
  const a = key(i);
  const b = key(i + 1);
  const q = key(i + 2);
  const f = (g: (s: Shot) => number) => cr(g(p), g(a), g(b), g(q));
  return {
    eye: [f((s) => s.eye[0]), f((s) => s.eye[1]), f((s) => s.eye[2])],
    yaw: f((s) => s.yaw),
    pitch: f((s) => s.pitch),
    fov: f((s) => s.fov),
  };
}

/** The ship's course: 3.6 m/s along +x, 170 m out; turned back once a loop while the camera looks at the moon. */
export const SHIP_Z = -170;
export const SHIP_SPEED = 3.6;
const WRAP = 54;
export function shipXAt(t: number): number {
  return -12 + SHIP_SPEED * (t < WRAP ? t : t - LOOP);
}
