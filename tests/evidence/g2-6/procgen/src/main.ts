// The G2-6 procgen sample: three tilemaps, each generated with og.procgen
// and written with og.tilemap.fill, regenerated live. The camera visits one
// per three seconds. tests/evidence/g2-6/NOTES.md has the commands and what
// each frame shows.
//
// - 0-3 s, the terrain: 3D FBM simplex noise sliced at a z that moves with
//   time, banded into water, sand, grass, rock and snow, with woods on the
//   grass where Poisson-disk points fall;
// - 3-6 s, the dungeon: BSP rooms and corridors, a new seed every half
//   second; the walls are a terrain layer, re-tiled once per fill;
// - 6-9 s, the village: a 16 × 10 WFC of roads, houses, fields and trees
//   (each WFC tile drawn as 3 × 3 cells), a new seed every 0.75 s.
//
// Every seed is a fork of og.Rng (`world.resource(og.Rng)`'s value
// `.fork(label).seed`), so the world
// replays exactly.

import { og, defineComponent, defineSystem, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** What must replay: on the entity "show". */
const Show = defineComponent("Show", {
  /** 0 terrain, 1 dungeon, 2 village. */
  phase: u32(0),
  /** Dungeons and villages generated so far. */
  dungeons: u32(0),
  villages: u32(0),
  /** The last dungeon's rooms and the last village's WFC try. */
  rooms: u32(0),
  attempt: u32(0),
  /** Woods placed on the terrain. */
  woods: u32(0),
});

const TERRAIN = { w: 64, h: 40, at: { x: 0, y: 0 } };
const DUNGEON = { w: 48, h: 32, at: { x: 200, y: 0 } };
const VILLAGE = { w: 48, h: 30, at: { x: 400, y: 0 } };
const SECTION = 180; // ticks per map

let maps: Entity[] = [];

// ── the terrain ──────────────────────────────────────────────────────────

function land(h: number): number {
  if (h < -0.28) return 0; // deep water
  if (h < -0.12) return 1; // water
  if (h < -0.06) return 7; // shallows
  if (h < 0.0) return 2; // sand
  if (h < 0.24) return 3; // grass
  if (h < 0.4) return 5; // rock
  return 6; // snow
}

function terrain(world: World, seed: number, woods: number, tick: number): number {
  const heights = must(og.procgen.noiseGrid({
    noise: { seed, kind: "simplex", octaves: 5, frequency: 0.04, threeD: true },
    width: TERRAIN.w, height: TERRAIN.h, x: tick * 0.12, y: 0, z: tick * 0.01,
  })) as number[];
  const tiles = heights.map(land);
  const points = must(og.procgen.poissonDisk({ seed: woods, width: TERRAIN.w, height: TERRAIN.h, radius: 2.2 }));
  let placed = 0;
  for (const p of points) {
    const i = Math.floor(p.y) * TERRAIN.w + Math.floor(p.x);
    // Woods only on inland grass.
    if (tiles[i] === 3 && heights[i]! > 0.06) {
      tiles[i] = 4;
      placed += 1;
    }
  }
  // Refused only in tick 0, before the og.tilemap system adopted the map.
  const filled = og.tilemap.fill(world, { entity: maps[0]!, layer: "land", x: 0, y: 0, width: TERRAIN.w, tiles });
  return filled.ok ? placed : 0;
}

// ── the dungeon ──────────────────────────────────────────────────────────

function dungeon(world: World, seed: number): number {
  const d = must(og.procgen.bsp({ seed, width: DUNGEON.w, height: DUNGEON.h, minRoom: 4, maxRoom: 10, depth: 5, padding: 1, corridor: 1 }));
  // Floor slabs 16-18 under rooms (varied by the cell), 19 the stairs in the last room; walls fill the rest.
  const ground = d.cells.map((c, i) => (c === 0 ? -1 : c === 2 ? 16 : 16 + ((i * 7 + Math.floor(i / DUNGEON.w) * 3) % 5 === 0 ? 1 : 0)));
  const last = d.rooms[d.rooms.length - 1];
  if (last) ground[(last.y + (last.height >> 1)) * DUNGEON.w + last.x + (last.width >> 1)] = 19;
  must(og.tilemap.fill(world, { entity: maps[1]!, layer: "ground", x: 0, y: 0, width: DUNGEON.w, tiles: ground }));
  must(og.tilemap.fill(world, { entity: maps[1]!, layer: "walls", x: 0, y: 0, width: DUNGEON.w, tiles: d.cells.map((c) => (c === 0 ? 1 : 0)) }));
  return d.rooms.length;
}

// ── the village ──────────────────────────────────────────────────────────

// Sockets [up, right, down, left]: 1 a road crosses that edge.
const VILLAGE_TILES = [
  { weight: 3, up: 0, right: 0, down: 0, left: 0, rotations: false }, // 0 grass
  { weight: 1.6, up: 0, right: 0, down: 0, left: 0, rotations: false }, // 1 house
  { weight: 1, up: 0, right: 0, down: 0, left: 0, rotations: false }, // 2 field
  { weight: 0.8, up: 0, right: 0, down: 0, left: 0, rotations: false }, // 3 trees
  { weight: 2, up: 1, right: 0, down: 1, left: 0, rotations: true }, // 4 road
  { weight: 1, up: 1, right: 1, down: 0, left: 0, rotations: true }, // 5 bend
  { weight: 0.6, up: 1, right: 1, down: 1, left: 0, rotations: true }, // 6 junction
  { weight: 0.2, up: 1, right: 1, down: 1, left: 1, rotations: false }, // 7 crossing
  { weight: 0.15, up: 1, right: 0, down: 0, left: 0, rotations: true }, // 8 a well at a road's end
];

/** The 3 × 3 cells (row-major) a WFC tile draws: 8 grass, 9 road, 10 wall, 11 roof, 12 tree, 13 field, 14 door, 15 well. */
function block(tile: number, rotation: number): number[] {
  switch (tile) {
    case 0: return [8, 8, 8, 8, 8, 8, 8, 8, 8];
    case 1: return [11, 11, 11, 10, 14, 10, 8, 8, 8];
    case 2: return [13, 13, 13, 13, 13, 13, 8, 8, 8];
    case 3: return [12, 8, 12, 8, 12, 8, 12, 8, 12];
  }
  const s = VILLAGE_TILES[tile]!;
  const base = [s.up, s.right, s.down, s.left];
  // A tile turned r quarters clockwise has side d's socket from side d - r.
  const side = (d: number) => base[(d - rotation + 4) % 4]!;
  const out = [8, 8, 8, 8, tile === 8 ? 15 : 9, 8, 8, 8, 8];
  const edge = [1, 5, 7, 3]; // the edge-middle cell of up, right, down, left
  for (let d = 0; d < 4; d++) if (side(d) === 1) out[edge[d]!] = 9;
  return out;
}

function village(world: World, seed: number): number {
  const W = VILLAGE.w / 3;
  const H = VILLAGE.h / 3;
  const v = must(og.procgen.wfc({ seed, width: W, height: H, tiles: VILLAGE_TILES, wrap: false, attempts: 20 }));
  const tiles = new Array<number>(VILLAGE.w * VILLAGE.h).fill(8);
  for (let ty = 0; ty < H; ty++) {
    for (let tx = 0; tx < W; tx++) {
      const b = block(v.tiles[ty * W + tx]!, v.rotations[ty * W + tx]!);
      for (let k = 0; k < 9; k++) tiles[(ty * 3 + Math.floor(k / 3)) * VILLAGE.w + tx * 3 + (k % 3)] = b[k]!;
    }
  }
  must(og.tilemap.fill(world, { entity: maps[2]!, layer: "land", x: 0, y: 0, width: VILLAGE.w, tiles }));
  return v.attempt;
}

// ── the show ─────────────────────────────────────────────────────────────

export const Generate = defineSystem(
  "Generate",
  { reads: [og.Rng], writes: [Show, og.Camera] },
  (_q, _time, world) => {
    const tick = world.tick;
    const show = world.find("show")!;
    const s = must(world.get(show, Show));
    const rng = must(world.resource(og.Rng));
    const phase = Math.min(2, Math.floor(tick / SECTION));
    const at = tick - phase * SECTION;
    const next = { ...s, phase };
    if (phase === 0) {
      next.woods = terrain(world, rng.fork("terrain").seed, rng.fork("woods").seed, tick);
    } else if (phase === 1 && at % 30 === 0) {
      next.rooms = dungeon(world, rng.fork(`dungeon-${s.dungeons}`).seed);
      next.dungeons = s.dungeons + 1;
    } else if (phase === 2 && at % 45 === 0) {
      next.attempt = village(world, rng.fork(`village-${s.villages}`).seed);
      next.villages = s.villages + 1;
    }
    world.set(show, Show, next);
    const m = [TERRAIN, DUNGEON, VILLAGE][phase]!;
    const cam = must(world.resource(og.Camera));
    cam.position = { x: m.at.x + m.w / 2, y: m.at.y - m.h / 2, z: 50 };
    cam.height = m.h + 2;
  },
);

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Generate],
  start(world: World) {
    const settings = must(world.resource(og.RenderSettings));
    settings.environmentIntensity = 0;
    settings.toneMapped = false;
    const cam = must(world.resource(og.Camera));
    cam.orthographic = true;
    cam.background = { r: 0.012, g: 0.012, b: 0.016 };
    maps = [];
    for (const [name, m] of [["terrain", TERRAIN], ["dungeon", DUNGEON], ["village", VILLAGE]] as const) {
      const map = must(og.load.tilemap(world, `maps/${name}.ogt`)).map;
      const e = must(world.spawn({ name, position: { x: m.at.x, y: m.at.y, z: 0 } }));
      must(world.insert(e, og.Tilemap, { map, collision: false }));
      maps.push(e);
    }
    must(world.insert(must(world.spawn({ name: "show" })), Show));
  },
});
