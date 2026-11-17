// Where things stand on the coast, in metres (x right, z towards the
// viewer, the cover camera looking down -z). tools/terrain.ts makes the
// ground from the same numbers, so both agree without measuring.

/** The terrain's side and its height samples per side. */
export const SIZE = 480;
export const SAMPLES = 1025;
/** The heights the heightmap's black and white stand for; the sea is y 0. */
export const HEIGHT_RANGE: readonly [number, number] = [-80, 80];

/** The mainland's coast: the land lies at x below this, at each z. */
export function coastX(z: number): number {
  // Straight past the camera, then falling away to the left behind the
  // headland, towards the far hills.
  const bend = Math.min(1, Math.max(0, (-z - 52) / 60));
  return 13 - 62 * bend * bend * (3 - 2 * bend) - Math.max(0, -z - 112) * 0.35;
}

/** The headland the lighthouse stands on: an ellipse, turned by `turn`. */
export const HEADLAND = { x: 15, z: -85, rx: 33, rz: 22, turn: -0.22 } as const;

/** The cliff path's centre line, from behind the camera to the lighthouse door. */
export const PATH: readonly (readonly [number, number])[] = [
  [1.5, 40], [0.4, 22], [-0.6, 8], [-0.9, 0], [-1.3, -5], [-0.6, -12], [0.8, -20], [0.4, -30],
  [-1.5, -42], [-1, -54], [1, -64], [5, -72], [10, -77],
];

/** The cover camera's place on the path and where it looks. */
export const CAMERA = { x: 0.2, z: 3.2, eye: 1.9, lookX: 2.3, lookZ: -75, drop: 3.6, fov: 52 } as const;
export const WANDERER = { x: -1.45, z: 0.1 } as const;
export const LIGHTHOUSE = { x: 14, z: -82, height: 24 } as const;
export const COTTAGE = { x: 2.5, z: -80, length: 13 } as const;
export const RUIN = { x: 26, z: -86, length: 9 } as const;
