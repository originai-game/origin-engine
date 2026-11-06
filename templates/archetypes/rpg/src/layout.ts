// Greywatch's yard as data: every wall, prop, torch, spawn and spot, in
// metres (x east, z south, y up). src/world.ts builds the keep from it, and
// tools/make-nav.ts writes the navmesh's bake file (assets/nav/keep.ogt)
// from the same boxes, so the foes walk where the heroes can. Plain data:
// nothing here imports the engine.
//
// The pieces are KayKit Dungeon Remastered's (assets/models/keep/), drawn
// at SCALE: a wall is 4 units wide, 4 tall and 1 deep in the kit, so one
// tile of the yard is TILE metres. The far walls (north and west, the ones
// an isometric camera looks at) stand whole; the near ones (south and
// east) are a cut-away parapet, so nothing stands between the camera and
// the party.

/** The kit's units to metres. */
export const SCALE = 0.7;
/** One tile: a wall's width, a floor tile's side. */
export const TILE = 4 * SCALE;
/** The yard inside the walls: 7 tiles across (x), 9 deep (z). */
export const YARD = { minX: -3.5 * TILE, maxX: 3.5 * TILE, minZ: -4.5 * TILE, maxZ: 4.5 * TILE } as const;
/** A wall's thickness and height in metres. */
export const WALL = { depth: SCALE, height: 4 * SCALE } as const;
/** The cut-away parapet's height as a share of a wall's. */
export const LOW = 0.3;

export type Piece = "wall" | "wall_arched" | "wall_broken" | "wall_doorway" | "wall_gated" | "pillar";

export interface Wall {
  readonly piece: Piece;
  readonly x: number;
  readonly z: number;
  /** Radians about +y: 0 runs east-west, π/2 north-south. */
  readonly yaw: number;
  /** 1 a whole wall, LOW the parapet. */
  readonly height: number;
}

const north = -4.5 * TILE - WALL.depth / 2;
const south = 4.5 * TILE + WALL.depth / 2;
const west = -3.5 * TILE - WALL.depth / 2;
const east = 3.5 * TILE + WALL.depth / 2;
const NS = Math.PI / 2;

/** The north wall, west to east: the crypt's door in the middle, the breach where the dead came up. */
const NORTH: readonly Piece[] = ["wall", "wall_arched", "wall", "wall_doorway", "wall", "wall_broken", "wall"];
/** The west wall, north to south: the gate the party came in by, shut behind it, second from the south. */
const WEST: readonly Piece[] = ["wall", "wall", "wall_arched", "wall", "wall", "wall", "wall", "wall_gated", "wall"];

export const WALLS: readonly Wall[] = [
  ...NORTH.map((piece, i) => ({ piece, x: (i - 3) * TILE, z: north, yaw: 0, height: 1 })),
  ...WEST.map((piece, j) => ({ piece, x: west, z: (j - 4) * TILE, yaw: NS, height: 1 })),
  ...Array.from({ length: 7 }, (_, i) => ({ piece: "wall" as const, x: (i - 3) * TILE, z: south, yaw: 0, height: LOW })),
  ...Array.from({ length: 9 }, (_, j) => ({ piece: "wall" as const, x: east, z: (j - 4) * TILE, yaw: NS, height: LOW })),
  // The corners' towers: whole at the back, cut down at the front.
  { piece: "pillar", x: west, z: north, yaw: 0, height: 1.25 },
  { piece: "pillar", x: east, z: north, yaw: 0, height: 1.25 },
  { piece: "pillar", x: west, z: south, yaw: 0, height: 1 },
  { piece: "pillar", x: east, z: south, yaw: 0, height: LOW + 0.1 },
];

/** A pillar's side in metres. */
export const PILLAR = 1.5 * SCALE;

export type PropKind = "crates_stacked" | "barrel_large" | "barrel_small" | "pillar" | "rubble_half" | "chest";

export interface Prop {
  readonly kind: PropKind;
  readonly x: number;
  readonly z: number;
  readonly yaw: number;
  /** Its solid box (metres, centred on x, z): what the heroes bump and the navmesh leaves out. */
  readonly w: number;
  readonly d: number;
  readonly h: number;
}

/** The yard's furniture: cover for the fights and the crypt's chest. */
export const PROPS: readonly Prop[] = [
  { kind: "pillar", x: -4.2, z: -1.4, yaw: 0, w: PILLAR, d: PILLAR, h: 2.8 },
  { kind: "pillar", x: 4.2, z: -1.4, yaw: 0, w: PILLAR, d: PILLAR, h: 2.8 },
  { kind: "pillar", x: -4.2, z: -6.6, yaw: 0, w: PILLAR, d: PILLAR, h: 2.8 },
  { kind: "pillar", x: 4.2, z: -6.6, yaw: 0, w: PILLAR, d: PILLAR, h: 2.8 },
  { kind: "crates_stacked", x: -8.2, z: -10.7, yaw: 0.3, w: 1.5, d: 1.5, h: 1.5 },
  { kind: "barrel_large", x: -6.6, z: -11.2, yaw: 0, w: 1.2, d: 1.2, h: 1.4 },
  { kind: "barrel_small", x: -8.6, z: -8.8, yaw: 0, w: 0.7, d: 0.7, h: 0.7 },
  { kind: "rubble_half", x: 4.4, z: -11.4, yaw: 0, w: 2.4, d: 1.6, h: 1.6 },
  { kind: "barrel_large", x: 8.4, z: -10.9, yaw: 0, w: 1.2, d: 1.2, h: 1.4 },
  { kind: "barrel_small", x: 8.5, z: 1.8, yaw: 0, w: 0.7, d: 0.7, h: 0.7 },
  { kind: "barrel_small", x: 7.8, z: 2.8, yaw: 0, w: 0.7, d: 0.7, h: 0.7 },
  { kind: "crates_stacked", x: 8.2, z: 7.6, yaw: -0.2, w: 1.5, d: 1.5, h: 1.5 },
  { kind: "barrel_large", x: -1.6, z: 5.2, yaw: 0, w: 1.2, d: 1.2, h: 1.4 },
  { kind: "barrel_small", x: -0.5, z: 5.8, yaw: 0, w: 0.7, d: 0.7, h: 0.7 },
];

/** The chest in front of the crypt's door, where the seal lies: opened with Talk once its guard has fallen. */
export const CHEST = { x: 0, z: -10.8, yaw: 0, w: 1.2, d: 0.9, h: 0.7 } as const;

export interface Banner { readonly x: number; readonly z: number; readonly yaw: number; readonly shield: boolean }
/** Banners on the far walls' faces. */
export const BANNERS: readonly Banner[] = [
  { x: -5.6 - 1.4, z: north, yaw: 0, shield: false },
  { x: -2.8, z: north, yaw: 0, shield: true },
  { x: 2.8, z: north, yaw: 0, shield: true },
  { x: west, z: -5.6, yaw: NS, shield: false },
  { x: west, z: 2.8, yaw: NS, shield: true },
];

/** Torches on the far walls (a bracket, its fire and a light): flanking the crypt's door and the gate. */
export const TORCHES: readonly { x: number; z: number; yaw: number }[] = [
  { x: -1.5, z: north + WALL.depth / 2, yaw: 0 },
  { x: 1.5, z: north + WALL.depth / 2, yaw: 0 },
  { x: -8.4, z: north + WALL.depth / 2, yaw: 0 },
  { x: 7.0, z: north + WALL.depth / 2, yaw: 0 },
  { x: west + WALL.depth / 2, z: 6.9, yaw: NS },
  { x: west + WALL.depth / 2, z: 9.9, yaw: NS },
  { x: west + WALL.depth / 2, z: -2.8, yaw: NS },
];

/** Where each seat's hero starts, inside the gate. */
export const STARTS: readonly { x: number; z: number }[] = [
  { x: -7.2, z: 9.4 },
  { x: -5.8, z: 10.3 },
];

/** Castellan Maren, by the gate, facing the yard. */
export const MAREN = { x: -8.2, z: 6.3, yaw: Math.PI / 2 } as const;

export interface FoeSpawn {
  readonly x: number;
  readonly z: number;
  /** 0 a risen minion, 1 the captain. */
  readonly kind: number;
}

/** The risen: four minions in the yard and their captain by the chest. */
export const FOES: readonly FoeSpawn[] = [
  { x: -2.4, z: -0.6, kind: 0 },
  { x: 4.6, z: 2.4, kind: 0 },
  { x: -6.0, z: -5.4, kind: 0 },
  { x: 6.4, z: -4.4, kind: 0 },
  { x: 0.4, z: -7.6, kind: 1 },
];

/**
 * The gate's ward: the torchlit corner inside the gate, by Maren, that the
 * risen will not enter (its navmesh area, NAV_WARD, is excluded from their
 * paths: src/foes.ts). The party starts there and may fall back to it.
 */
export const WARD = { minX: YARD.minX, maxX: -4.4, minZ: 3.4, maxZ: YARD.maxZ } as const;
/** The ward's navmesh area id. */
export const NAV_WARD = 1;

/** The navmesh's bake bounds: the yard and a little beyond. */
export const NAV_BOUNDS = { min: [YARD.minX - 1, -1, YARD.minZ - 1], max: [YARD.maxX + 1, 2, YARD.maxZ + 1] } as const;

/** Every solid box of the yard (centre and size in metres): the walls', the props' and the chest's. */
export function solidBoxes(): { x: number; y: number; z: number; w: number; h: number; d: number }[] {
  const out: { x: number; y: number; z: number; w: number; h: number; d: number }[] = [];
  for (const w of WALLS) {
    const h = WALL.height * w.height;
    if (w.piece === "pillar") out.push({ x: w.x, y: h / 2, z: w.z, w: PILLAR, h, d: PILLAR });
    else if (w.yaw === 0) out.push({ x: w.x, y: h / 2, z: w.z, w: TILE, h, d: WALL.depth });
    else out.push({ x: w.x, y: h / 2, z: w.z, w: WALL.depth, h, d: TILE });
  }
  for (const p of PROPS) out.push({ x: p.x, y: p.h / 2, z: p.z, w: p.w, h: p.h, d: p.d });
  out.push({ x: CHEST.x, y: CHEST.h / 2, z: CHEST.z, w: CHEST.w, h: CHEST.h, d: CHEST.d });
  return out;
}
