// The Kestrel Stair's layout: one table of plain numbers that both the game
// (src/world.ts, src/foes.ts, ...) and the asset generator
// (tools/make-assets.ts writes the level's glTF, its navmesh and the foes'
// behaviours from it) import, so the level and the code cannot drift apart.
// Nothing here touches the engine. Metres, y up, the climb runs north (-z).

export interface Box {
  min: [number, number, number];
  max: [number, number, number];
}

/** The ground heights of the four levels of the climb. */
export const Y = { court: 0, terrace: 4, high: 8, summit: 12 } as const;

/** Where a new journey starts: the south end of the lower court, facing north. */
export const START = { x: 0, y: 0, z: 39 } as const;

/** The solid blocks the level is built of (their tops are the walkable floors). */
export const BLOCKS: readonly (Box & { mat: "paving" | "stone" | "grass" | "plaster" })[] = [
  // The lower court's paving.
  { min: [-22, -1, 12], max: [22, 0, 44], mat: "paving" },
  // The crypt terrace (its front is the retaining wall the ramps climb).
  { min: [-22, 0, -24], max: [22, 4, 12], mat: "grass" },
  // The high terrace, and the strip behind the summit gate.
  { min: [-22, 0, -66], max: [22, 8, -24], mat: "grass" },
  // The summit.
  { min: [-9, 8, -66], max: [9, 12, -54], mat: "stone" },
];

/** Ramps: a wedge from (x0..x1, y0 at z0) up to (y1 at z1). */
export const RAMPS: readonly { x0: number; x1: number; z0: number; z1: number; y0: number; y1: number; name: string }[] = [
  // The broad ramp from the court to the crypt terrace (18.4 degrees).
  { x0: -5, x1: 5, z0: 24, z1: 12, y0: 0, y1: 4, name: "ramp" },
  // The steep path on the east side (33.7 degrees: still walkable).
  { x0: 14, x1: 18, z0: 18, z1: 12, y0: 0, y1: 4, name: "steep" },
  // The scree on the west side (63 degrees: too steep, the hero slides back).
  { x0: -18, x1: -14, z0: 14, z1: 12, y0: 0, y1: 4, name: "scree" },
];

/** Stairs: `steps` steps of `rise` metres, from (z0, y0) north to z1. */
export const STAIRS: readonly { x0: number; x1: number; z0: number; z1: number; y0: number; steps: number; rise: number; name: string }[] = [
  { x0: -4, x1: 4, z0: -16, z1: -24, y0: 4, steps: 16, rise: 0.25, name: "stair" },
  { x0: -3, x1: 3, z0: -46, z1: -54, y0: 8, steps: 16, rise: 0.25, name: "summit-stair" },
];

/** The crypt: walls round a room on the crypt terrace, a door in the east wall, a roof. */
export const CRYPT = {
  min: [-21, 4, -14] as [number, number, number],
  max: [-9, 8.5, -2] as [number, number, number],
  wall: 0.6,
  door: { z0: -9.6, z1: -6.4, top: 7 },
  pillars: [[-17, -11], [-13, -11], [-17, -5], [-13, -5]] as [number, number][],
  altar: { x: -19.3, z: -8, top: 5.1 },
  chain: { x: -15, y: 8.5, z: -8, length: 2.6 },
} as const;

/** The ledges east of the crypt terrace: stone posts, each a jump higher than the last. */
export const LEDGES: readonly { x: number; z: number; top: number; size: number }[] = [
  { x: 16.5, z: -5, top: 5.0, size: 2.6 },
  { x: 19, z: -9.2, top: 6.0, size: 2.4 },
  { x: 16.2, z: -13.2, top: 7.0, size: 2.4 },
  { x: 19, z: -17.4, top: 8.0, size: 2.4 },
  { x: 16.2, z: -21.2, top: 9.2, size: 2.6 },
];

/** Walls: parapets round the edges, the gate wall across the high terrace. */
export const WALLS: readonly Box[] = [
  // The court: west, east and south.
  { min: [-22.6, 0, 12], max: [-22, 3, 44] },
  { min: [22, 0, 12], max: [22.6, 3, 44] },
  { min: [-22.6, 0, 44], max: [22.6, 3, 44.6] },
  // The crypt terrace's sides.
  { min: [-22.6, 4, -24], max: [-22, 6, 12] },
  { min: [22, 4, -24], max: [22.6, 6, 12] },
  // The high terrace's sides and the back.
  { min: [-22.6, 8, -66], max: [-22, 10, -24] },
  { min: [22, 8, -66], max: [22.6, 10, -24] },
  { min: [-22.6, 8, -66.6], max: [22.6, 10, -66] },
  // The gate wall, either side of the gate.
  { min: [-22, 8, -44.5], max: [-3, 12, -43.5] },
  { min: [3, 8, -44.5], max: [22, 12, -43.5] },
  // The summit stair's balustrades.
  { min: [-3.5, 8, -54], max: [-3, 13, -46] },
  { min: [3, 8, -54], max: [3.5, 13, -46] },
  // The summit's parapet (its stair side open).
  { min: [-9.5, 12, -66.5], max: [9.5, 13, -66] },
  { min: [-9.5, 12, -66], max: [-9, 13, -54] },
  { min: [9, 12, -66], max: [9.5, 13, -54] },
  { min: [-9, 12, -54.4], max: [-3.5, 13, -54] },
  { min: [3.5, 12, -54.4], max: [9, 13, -54] },
];

/** Ruined columns standing about the court and the terraces: obstacles the foes steer round. */
export const COLUMNS: readonly { x: number; y: number; z: number; h: number }[] = [
  { x: -12, y: 0, z: 34, h: 3.2 },
  { x: 12, y: 0, z: 34, h: 2.1 },
  { x: -12, y: 0, z: 22, h: 1.4 },
  { x: 9, y: 0, z: 22, h: 3.4 },
  { x: 6, y: 0, z: 27, h: 0.9 },
  { x: 8, y: 4, z: 2, h: 2.6 },
  { x: -4, y: 4, z: -6, h: 3.0 },
  { x: 11, y: 4, z: -12, h: 1.2 },
  { x: -10, y: 8, z: -32, h: 2.8 },
  { x: 12, y: 8, z: -30, h: 3.1 },
  { x: 0, y: 8, z: -36, h: 1.0 },
];

/** Braziers: a stone bowl with a fire, a light and a crackle you hear from where it stands. */
export const BRAZIERS: readonly { x: number; y: number; z: number }[] = [
  { x: -6, y: 4, z: 9 },
  { x: 6, y: 4, z: 9 },
  { x: -5, y: 8, z: -26.5 },
  { x: 5, y: 8, z: -26.5 },
  { x: -4, y: 8, z: -42 },
  { x: 4, y: 8, z: -42 },
];

/** Banners (cloth) hung from posts, stirring in the wind. */
export const BANNERS: readonly { x: number; y: number; z: number; yaw: number }[] = [
  { x: -8, y: 4, z: 11.2, yaw: 0 },
  { x: 8, y: 4, z: 11.2, yaw: 0 },
  { x: -7, y: 8, z: -24.8, yaw: 0 },
  { x: 7, y: 8, z: -24.8, yaw: 0 },
];

/** The summit gate: a portcullis between the gate walls, raised by three sigils. */
export const GATE: Box = { min: [-3, 8, -44.4], max: [3, 12, -43.6] };

/** The summit's bell, where the letter is delivered and the climb ends. */
export const BELL = { x: 0, y: 12, z: -60 } as const;

/** The waystones: chapter starts and save points. */
export const WAYSTONES: readonly { x: number; y: number; z: number; key: string }[] = [
  { x: 3, y: 0, z: 36, key: "court" },
  { x: 3, y: 4, z: 7, key: "terrace" },
  { x: 7, y: 8, z: -28, key: "high" },
  { x: 7, y: 8, z: -40, key: "gate" },
];

/** The three sigils: A on the crypt's altar, B on the last ledge, C carried by the terrace warden. */
export const SIGILS: readonly { x: number; y: number; z: number }[] = [
  { x: CRYPT.altar.x, y: CRYPT.altar.top, z: CRYPT.altar.z },
  { x: 16.2, y: 9.2, z: -21.2 },
  { x: 0, y: 0, z: 0 },
];

/** Healing herbs growing about the climb. */
export const HERBS: readonly { x: number; y: number; z: number }[] = [
  { x: -16, y: 0, z: 26 },
  { x: 18, y: 4, z: 4 },
  { x: -14, y: 4, z: -18 },
  { x: -16, y: 8, z: -40 },
];

/** The foes: their kind, where each starts and the round it patrols. */
export const FOES: readonly { kind: "imp" | "warden"; name: string; route: [number, number, number][] }[] = [
  { kind: "imp", name: "imp-0", route: [[-9, 0, 31], [9, 0, 31], [9, 0, 19], [-9, 0, 19]] },
  { kind: "imp", name: "imp-1", route: [[16, 0, 38], [16, 0, 16], [6, 0, 16]] },
  { kind: "warden", name: "warden-0", route: [[6, 4, -2], [6, 4, -14], [-4, 4, -12]] },
  { kind: "imp", name: "imp-2", route: [[-14, 8, -28], [-6, 8, -40], [-16, 8, -38]] },
  { kind: "warden", name: "warden-1", route: [[9, 8, -38], [-9, 8, -38]] },
];

/** The chapters (og.ui.levelSelect): each starts at a waystone with what the chapters before it guarantee. */
export const CHAPTERS: readonly { waystone: number; sigils: number; key: string }[] = [
  { waystone: 0, sigils: 0, key: "court" },
  { waystone: 1, sigils: 0, key: "terrace" },
  { waystone: 2, sigils: 0b111, key: "high" },
  { waystone: 3, sigils: 0b111, key: "gate" },
];

/** Which chapter a point belongs to (for the chapter cards' progress). */
export function chapterAt(x: number, y: number, z: number): number {
  if (y > 6.5 && z <= -35) return 3;
  if (y > 6.5 && z < -23) return 2;
  if (z < 12 && y > 2) return 1;
  return 0;
}

/** The crypt's inside: the reverb zone (a box the hero's head is in). */
export const CRYPT_ROOM: Box = { min: [-20.4, 3.5, -13.4], max: [-9.6, 8.5, -2.6] };

/**
 * The foes' thorns and stones when whole. The thresholds their behaviours
 * compare against live in Tuning (`check Foe.hp below Tuning.impFleeBelow`;
 * FINDINGS #3, fixed: a check reads a field on its right too).
 */
export const MIND = {
  impHp: 3,
  wardenHp: 6,
} as const;
