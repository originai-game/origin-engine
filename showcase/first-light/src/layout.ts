// Where things stand, in scene metres: the camera's knoll is the origin,
// the view runs down -z, +x is to the right, the lake's surface is y 0.
// tools/terrain.ts shapes the ground from these and the game places its
// models by them, so the two never drift apart. No engine import: the tool
// runs under bun alone.

const clamp01 = (t: number) => Math.min(1, Math.max(0, t));
const smooth = (a: number, b: number, t: number) => {
  const k = clamp01((t - a) / (b - a));
  return k * k * (3 - 2 * k);
};
/** The terrain: its side in metres, its samples a side, where its centre stands and the heights black and white mean. */
export const SIZE = 8192;
export const SAMPLES = 2049;
export const CENTER = { x: 600, z: -2600 };
export const HEIGHT = { low: -32, high: 2016 };

/** The knoll under the oak: a flat top a few metres across where the shepherd stands, falling away to the paddock and the shore. */
export const KNOLL = { x: 0, z: 12, top: 6, radius: 60, height: 15 };

/** The lake: its spine from the near bay out to the far valley, and its half width at each end. */
export const LAKE = {
  spine: [[-48, -174.5], [-82.2, -463.7], [-147.4, -929.3], [-352.2, -1653.9], [-676.2, -2707.8]] as const,
  widthNear: 135,
  widthFar: 330,
};

const from = (x: number, z: number, cx: number, cz: number) => Math.sqrt((x - cx) * (x - cx) + (z - cz) * (z - cz));

/** The village's shelf on the near right shore: low even ground rising gently from the water. */
export const VILLAGE = {
  shelf: (x: number, z: number) => 1 - smooth(45, 95, from(x, z, 62, -72)),
  height: (x: number, z: number) => 1.4 + clamp01((from(x, z, LAKE.spine[0][0], LAKE.spine[0][1]) - 140) / 70) * 5,
  /** Houses: x, z, the model and the heading its front faces, in degrees. */
  houses: [
    { x: 53.3, z: -34.7, model: "house-a", turn: 330 },
    { x: 32.2, z: -44.7, model: "house-b", turn: 10 },
    { x: 44.3, z: -46.2, model: "house-c", turn: 300 },
    { x: 34.5, z: -59.3, model: "house-b", turn: 350 },
    { x: 54.9, z: -70.6, model: "house-a", turn: 20 },
    { x: 62.8, z: -59.4, model: "house-b", turn: 310 },
    { x: 77.6, z: -96.2, model: "house-c", turn: 0 },
    { x: 84.6, z: -77.3, model: "house-a", turn: 340 },
    { x: 95.6, z: -107.6, model: "house-b", turn: 15 },
  ],
  /** The jetty starts on the shore near here and runs out along `heading`; the boat lies beside it. */
  jetty: { x: 34, z: -52, heading: 214 },
} as const;

/** The sheep's paddock on the slope between the knoll and the village: open pasture inside a dry stone wall. */
export const PADDOCK = {
  weight: (x: number, z: number) => 1 - smooth(14, 30, from(x, z, 21.3, -23.2)),
  /** The wall's corners, in order; the gate stands on the first side. */
  wall: [[9.8, -14.5], [17.9, -8.3], [34.6, -30.8], [20.7, -39.2]] as const,
  gate: { x: 16.3, z: -9 },
  /** Where the sheep stand. */
  flock: [[14.8, -18.9], [21.3, -23.2], [21.1, -17.4], [27, -27.9], [20.5, -30.5], [27.6, -22.4], [25.2, -32.6]] as const,
} as const;

/** The windmill's hill behind the village. */
export const MILL = { x: 144.4, z: -144.6, radius: 72, height: 22 };

/** The wheat field on the hill's flank below the mill. */
export const FIELD = {
  weight: (x: number, z: number) => 1 - smooth(16, 28, from(x, z, 112.7, -91.4)),
} as const;

/** Paths of packed earth: from the paddock's gate down through the village and up to the mill, and a spur to the jetty. */
export const PATHS = [
  { width: 2.6, points: [[14.7, -7.1], [24.9, -13.9], [35.9, -27], [38.5, -40.5], [45.9, -57.3], [62.8, -69.8], [81.5, -86.8], [108.3, -109.9], [123, -126.3]] },
  { width: 2.2, points: [[45.9, -57.3], [38, -53.5], [33, -52.5]] },
] as const;
