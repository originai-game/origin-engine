// A floor's plan: rooms, the corridors between them, where the lamp-bearer
// starts, the stairs (floor 1) or the relic and its guardian (floor 2), the
// foes and the finds. A pure function of the dive's seed and the floor
// number: the plan comes from an og.Rng stream forked with both, so every
// system that needs it (the Hero carves it, the Foes and the Loot place
// their rows) computes the same plan, and a dive replays from its seed.

import type { RngStream } from "@origingame/engine";
import { FoeKind, LootKind } from "./components.ts";

/** The floor's size in cells (assets/maps/dive.ogt). */
export const W = 28;
export const H = 20;

export interface Room {
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
}

export interface Plan {
  /** 1 where the floor is stone, row-major from the top row. */
  readonly walls: Uint8Array;
  readonly rooms: readonly Room[];
  readonly start: { x: number; y: number };
  /** Floor 1's way down. */
  readonly stairs: { x: number; y: number } | null;
  /** Floor 2's relic, in the farthest room. */
  readonly relic: { x: number; y: number } | null;
  readonly foes: ReadonlyArray<{ kind: number; x: number; y: number }>;
  readonly loot: ReadonlyArray<{ kind: number; x: number; y: number; amount: number }>;
}

const centre = (r: Room) => ({ x: r.x + Math.floor(r.w / 2), y: r.y + Math.floor(r.h / 2) });

/**
 * Rooms placed at random without touching, joined in order of their
 * centres by L-shaped corridors (which way the L turns is a coin), then the
 * start in the first room, the goal in the room farthest from it by walking
 * distance, and the foes and finds in the others.
 */
export function planFloor(rng: RngStream, floor: number): Plan {
  const walls = new Uint8Array(W * H).fill(1);
  const rooms: Room[] = [];
  for (let attempt = 0; attempt < 80 && rooms.length < 8; attempt++) {
    const w = rng.integer(4, 7);
    const h = rng.integer(3, 5);
    const x = rng.integer(1, W - w - 1);
    const y = rng.integer(1, H - h - 1);
    const touches = rooms.some((r) => x <= r.x + r.w && r.x <= x + w && y <= r.y + r.h && r.y <= y + h);
    if (!touches) rooms.push({ x, y, w, h });
  }
  rooms.sort((a, b) => centre(a).x - centre(b).x || centre(a).y - centre(b).y);
  const open = (x: number, y: number) => {
    if (x > 0 && y > 0 && x < W - 1 && y < H - 1) walls[y * W + x] = 0;
  };
  for (const r of rooms) for (let y = r.y; y < r.y + r.h; y++) for (let x = r.x; x < r.x + r.w; x++) open(x, y);
  for (let i = 1; i < rooms.length; i++) {
    const a = centre(rooms[i - 1]!);
    const b = centre(rooms[i]!);
    const flat = rng.chance(0.5);
    const corner = flat ? { x: b.x, y: a.y } : { x: a.x, y: b.y };
    for (const [p, q] of [[a, corner], [corner, b]] as const) {
      for (let x = Math.min(p.x, q.x); x <= Math.max(p.x, q.x); x++) open(x, p.y);
      for (let y = Math.min(p.y, q.y); y <= Math.max(p.y, q.y); y++) open(q.x, y);
    }
  }
  const start = centre(rooms[0]!);
  // Walking distance from the start to every open cell (four ways).
  const dist = new Int32Array(W * H).fill(-1);
  const queue = [start.y * W + start.x];
  dist[queue[0]!] = 0;
  for (let k = 0; k < queue.length; k++) {
    const c = queue[k]!;
    for (const d of [1, -1, W, -W]) {
      const n = c + d;
      if (n >= 0 && n < W * H && walls[n] === 0 && dist[n]! < 0) {
        dist[n] = dist[c]! + 1;
        queue.push(n);
      }
    }
  }
  let far = 0;
  rooms.forEach((r, i) => {
    const c = centre(r);
    if (dist[c.y * W + c.x]! > dist[centre(rooms[far]!).y * W + centre(rooms[far]!).x]!) far = i;
  });
  const goal = centre(rooms[far]!);
  const taken = new Set<number>([start.y * W + start.x, goal.y * W + goal.x]);
  /** A free cell of a room, not its doorway rows: tries a few times, then the room's centre. */
  const spot = (r: Room) => {
    for (let k = 0; k < 12; k++) {
      const x = rng.integer(r.x, r.x + r.w - 1);
      const y = rng.integer(r.y, r.y + r.h - 1);
      if (!taken.has(y * W + x)) {
        taken.add(y * W + x);
        return { x, y };
      }
    }
    return null;
  };
  const foes: Array<{ kind: number; x: number; y: number }> = [];
  const loot: Array<{ kind: number; x: number; y: number; amount: number }> = [];
  const others = rooms.map((_, i) => i).filter((i) => i !== 0);
  others.forEach((i, k) => {
    const r = rooms[i]!;
    const count = floor === 2 ? 1 + (k % 2) : 1;
    for (let n = 0; n < count; n++) {
      const at = spot(r);
      if (at === null) continue;
      if (i === far && floor === 2) continue;
      const kind = floor === 1 ? (k % 3 === 2 ? FoeKind.Skeleton : FoeKind.Rat) : n === 0 ? FoeKind.Skeleton : FoeKind.Rat;
      foes.push({ kind, ...at });
    }
  });
  if (floor === 2) {
    // The guardian stands between the relic and the way in.
    const g = { x: goal.x - 1, y: goal.y };
    taken.add(g.y * W + g.x);
    foes.push({ kind: FoeKind.Guardian, ...g });
  }
  const finds = floor === 1 ? [LootKind.Potion, LootKind.Blade, LootKind.Oil, LootKind.Coin, LootKind.Coin, LootKind.Potion] : [LootKind.Potion, LootKind.Ward, LootKind.Coin, LootKind.Coin, LootKind.Potion];
  finds.forEach((kind, k) => {
    const r = rooms[others[k % others.length]!]!;
    const at = spot(r);
    if (at !== null) loot.push({ kind, ...at, amount: kind === LootKind.Coin ? rng.integer(5, 15) : 0 });
  });
  if (floor === 2) loot.push({ kind: LootKind.Relic, ...goal, amount: 0 });
  return {
    walls,
    rooms,
    start,
    stairs: floor === 1 ? goal : null,
    relic: floor === 2 ? goal : null,
    foes: foes.slice(0, 10),
    loot: loot.slice(0, 12),
  };
}

/** The stream a floor is planned from: the world's seed, the dive's and the floor's. */
export function floorStream(rng: { fork(label: string): RngStream }, seed: number, floor: number): RngStream {
  return rng.fork(`dive-${seed}-floor-${floor}`);
}
