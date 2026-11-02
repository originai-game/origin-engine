// The G2-6 run-time tilemaps sample (batch 2): two maps made in game code
// with og.tilemap.create, no map asset at all. tests/evidence/g2-6/NOTES.md
// has the commands and what each frame shows.
//
// - 0-4 s, an isometric island: 20 × 20 diamonds banded from FBM simplex
//   noise. At 2 s og.tilemap.resize grows it to 28 × 28 with the old cells
//   shifted 4 × 4 in, and the new ring is filled from the same noise field,
//   so the island grows a coast without moving. The cell under the centre
//   and its og.tilemap.neighbours are drawn orange.
// - 4-8 s, a pointy-top hex map, 30 × 20: rock hexes (solid) at the bottom
//   and in a few noise-placed outcrops, each its own hexagon collider. A
//   ball drops every 0.2 s and bounces off the hexagons' slanted sides. The
//   hex under the centre and its six neighbours are orange.
//
// Every seed is a fork of og.Rng (`world.resource(og.Rng)`'s value
// `.fork(label).seed`), so the world replays exactly.

import { og, defineComponent, defineSystem, u32, SHAPE_SPHERE } from "@origingame/engine";
import type { Entity, TilemapAsset, Texture, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** What must replay: on the entity "show". */
const Show = defineComponent("Show", {
  /** 0 the island, 1 the hex map. */
  phase: u32(0),
  /** The island's size now. */
  width: u32(0),
  /** Balls dropped. */
  balls: u32(0),
});

const ISLAND = { w: 20, h: 20, grown: 28, shift: 4, at: { x: 0, y: 0 } };
const HEX = { w: 30, h: 20, at: { x: 100, y: 0 } };
const SECTION = 240;

let island: { entity: Entity; map: TilemapAsset } | null = null;
let hex: { entity: Entity; map: TilemapAsset } | null = null;
let ball: Texture | null = null;

function band(h: number): number {
  if (h < -0.25) return 0; // deep water
  if (h < -0.05) return 1; // water
  if (h < 0.02) return 2; // sand
  if (h < 0.22) return 3; // grass
  if (h < 0.34) return 4; // forest
  if (h < 0.45) return 5; // rock
  return 6; // snow
}

/** The island's tiles for a map of `size` whose cell (0, 0) is noise cell (-off, -off). */
function islandTiles(seed: number, size: number, off: number): number[] {
  const heights = must(og.procgen.noiseGrid({
    noise: { seed, kind: "simplex", octaves: 4, frequency: 0.09 },
    width: size, height: size, x: -off, y: -off,
  })) as number[];
  const c = (ISLAND.grown - 1) / 2;
  return heights.map((h, i) => {
    // Fall off from the grown map's centre so the island has a coast.
    const x = (i % size) - off - c + ISLAND.shift;
    const y = Math.floor(i / size) - off - c + ISLAND.shift;
    const d = Math.sqrt(x * x + y * y) / (ISLAND.grown / 2);
    return band(h + 0.35 - d * 0.9);
  });
}

function highlight(world: World, m: { entity: Entity; map: TilemapAsset }, tiles: number[], width: number, x: number, y: number): number[] {
  const out = tiles.slice();
  out[y * width + x] = 7;
  for (const n of og.tilemap.neighbours(world, m.map, x, y)) out[n.y * width + n.x] = 7;
  return out;
}

function hexTiles(seed: number): number[] {
  const heights = must(og.procgen.noiseGrid({
    noise: { seed, kind: "simplex", octaves: 3, frequency: 0.18 },
    width: HEX.w, height: HEX.h, x: 0, y: 0,
  })) as number[];
  return heights.map((h, i) => {
    const y = Math.floor(i / HEX.w);
    if (y >= HEX.h - 2) return 5; // the floor
    if (y > 6 && h > 0.3) return 5; // outcrops
    return y < 5 ? 1 : 3; // sky-water, then grass behind
  });
}

export const Show_ = defineSystem(
  "Show",
  { reads: [og.Rng], writes: [Show, og.Camera, og.Transform, og.RigidBody, og.Collider, og.Sprite] },
  (_q, _time, world) => {
    const tick = world.tick;
    const show = world.find("show")!;
    const s = must(world.get(show, Show));
    const rng = must(world.resource(og.Rng));
    const phase = Math.min(1, Math.floor(tick / SECTION));
    const next = { ...s, phase };
    const cam = must(world.resource(og.Camera));
    if (phase === 0 && island) {
      const seed = rng.fork("island").seed;
      if (tick === 1) {
        const tiles = islandTiles(seed, ISLAND.w, 0);
        must(og.tilemap.fill(world, { entity: island.entity, layer: "land", x: 0, y: 0, width: ISLAND.w, tiles: highlight(world, island, tiles, ISLAND.w, 10, 10) }));
        next.width = ISLAND.w;
      } else if (tick === 120) {
        must(og.tilemap.resize(world, { map: island.map, width: ISLAND.grown, height: ISLAND.grown, x: ISLAND.shift, y: ISLAND.shift }));
        next.width = ISLAND.grown;
      } else if (tick === 121) {
        const tiles = islandTiles(seed, ISLAND.grown, ISLAND.shift);
        must(og.tilemap.fill(world, { entity: island.entity, layer: "land", x: 0, y: 0, width: ISLAND.grown, tiles: highlight(world, island, tiles, ISLAND.grown, 14, 14) }));
      }
      const mid = og.tilemap.cellCenter(world, island.entity, 13, 13);
      cam.position = { x: mid.x, y: mid.y, z: 50 };
      cam.height = 32;
    } else if (phase === 1 && hex && ball) {
      if (tick === SECTION) {
        const tiles = hexTiles(rng.fork("hex").seed);
        must(og.tilemap.fill(world, { entity: hex.entity, layer: "ground", x: 0, y: 0, width: HEX.w, tiles: highlight(world, hex, tiles, HEX.w, 15, 9) }));
      }
      const mid = og.tilemap.cellCenter(world, hex.entity, 15, 10);
      cam.position = { x: mid.x, y: mid.y, z: 50 };
      cam.height = 18;
      if ((tick - SECTION) % 12 === 0 && s.balls < 18) {
        // Dropped across the map at a forked offset: the same in every replay.
        const r = rng.fork(`ball-${s.balls}`).seed;
        const top = og.tilemap.cellCenter(world, hex.entity, 3 + (r % 24), 0);
        const b = must(world.spawn({ name: `ball-${s.balls}`, position: { x: top.x + ((r >> 8) % 100) / 200, y: top.y + 1, z: 1 } }));
        must(world.insert(b, og.RigidBody, { mass: 1 }));
        must(world.insert(b, og.Collider, { shape: SHAPE_SPHERE, radius: 0.35, restitution: 0.4 }));
        must(world.insert(b, og.Sprite, { texture: ball, width: 0.7, height: 0.7, layer: 3 }));
        next.balls = s.balls + 1;
      }
    }
    world.set(show, Show, next);
  },
);

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Show_],
  start(world: World) {
    const settings = must(world.resource(og.RenderSettings));
    settings.environmentIntensity = 0;
    settings.toneMapped = false;
    const cam = must(world.resource(og.Camera));
    cam.orthographic = true;
    cam.background = { r: 0.012, g: 0.012, b: 0.016 };
    ball = must(og.load.texture(world, "tiles/ball.png"));
    const iso = must(og.tilemap.create(world, {
      width: ISLAND.w, height: ISLAND.h, tileWidth: 32, tileHeight: 16, unit: 2,
      orientation: "isometric", tilesets: ["tiles/iso.ogt"], layers: [{ name: "land" }],
    }));
    const ie = must(world.spawn({ name: "island", position: { x: ISLAND.at.x, y: ISLAND.at.y, z: 0 } }));
    must(world.insert(ie, og.Tilemap, { map: iso.map, collision: false }));
    island = { entity: ie, map: iso.map };
    const hx = must(og.tilemap.create(world, {
      width: HEX.w, height: HEX.h, tileWidth: 32, tileHeight: 32, unit: 1,
      orientation: "hex", tilesets: ["tiles/hex.ogt"], layers: [{ name: "ground", collision: true }],
    }));
    const he = must(world.spawn({ name: "hex", position: { x: HEX.at.x, y: HEX.at.y, z: 0 } }));
    must(world.insert(he, og.Tilemap, { map: hx.map }));
    hex = { entity: he, map: hx.map };
    must(world.insert(must(world.spawn({ name: "show" })), Show));
  },
});
