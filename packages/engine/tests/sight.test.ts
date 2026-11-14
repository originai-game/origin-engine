// Fog of war through the TypeScript API on the native host: an og.SightMap
// over an og.Volume, og.Sight eyes and og.SightBlocker walls. A team sees
// the cells within its eyes' reach that no wall hides; what it saw stays
// explored after it walks on; a door that opens lets sight through; reveal
// and forget change what is explored; a system reads the same answer as
// the world between ticks; and the map is world state: the same on every
// run, and back after a restore.
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const at = (x: number, z: number) => ({ x, y: 0, z });

/** A 32 m map of 1 m cells; a scout of team 0 west of a wall along x -4, a sentry of team 1 in the south-east. */
function field(world: World): void {
  const map = must(world.spawn({ name: "map" }));
  world.insert(map, og.Volume, { size: { x: 32, y: 4, z: 32 } });
  world.insert(map, og.SightMap, { cell: 1, team: 0 });
  const scout = must(world.spawn({ name: "scout", position: at(-8.5, 0.5) }));
  world.insert(scout, og.Sight, { team: 0, radius: 9 });
  const sentry = must(world.spawn({ name: "sentry", position: at(8.5, 8.5) }));
  world.insert(sentry, og.Sight, { team: 1, radius: 4 });
  const wall = must(world.spawn({ name: "wall", position: at(-3.5, 0) }));
  world.insert(wall, og.Volume, { size: { x: 1, y: 4, z: 8 } });
  world.insert(wall, og.SightBlocker);
}

function start(systems: unknown[] = []): Engine {
  const r = createEngine(og.game({ start: field, systems } as never) as GameToken, {});
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const sees = (w: World, team: number, x: number, z: number) => must(og.sight.visible(w, team, at(x, z)));
const knows = (w: World, team: number, x: number, z: number) => must(og.sight.explored(w, team, at(x, z)));

test("a team sees within its reach, a wall hides what is behind it, and what was seen stays explored", () => {
  const engine = start();
  expect(engine.advance(2).ok).toBeTrue();
  const w = engine.world;
  expect(sees(w, 0, -8.5, 0.5)).toBeTrue();
  expect(sees(w, 0, -8.5, 6.5)).toBeTrue();
  expect(sees(w, 0, -8.5, 10.5)).toBeFalse(); // past its radius
  expect(sees(w, 0, -3.5, 0.5)).toBeTrue(); // the wall's face
  expect(sees(w, 0, -2.5, 0.5)).toBeFalse(); // behind the wall, within reach
  expect(sees(w, 0, -2.5, 5.5)).toBeTrue(); // past the wall's end
  // Each team its own cells.
  expect(sees(w, 1, 8.5, 8.5)).toBeTrue();
  expect(sees(w, 0, 8.5, 8.5)).toBeFalse();
  expect(sees(w, 1, -8.5, 0.5)).toBeFalse();
  // Outside the map nobody sees.
  expect(sees(w, 0, -20, 0.5)).toBeFalse();

  // The door opens: sight passes.
  must(w.set(w.find("wall")!, og.SightBlocker, { enabled: false }));
  expect(engine.advance(1).ok).toBeTrue();
  expect(sees(engine.world, 0, -2.5, 0.5)).toBeTrue();

  // The scout walks on: where it was is explored, no longer seen.
  must(engine.world.set(engine.world.find("scout")!, og.Transform, { position: at(8.5, -8.5) }));
  expect(engine.advance(1).ok).toBeTrue();
  const v = engine.world;
  expect(sees(v, 0, -8.5, 0.5)).toBeFalse();
  expect(knows(v, 0, -8.5, 0.5)).toBeTrue();
  expect(sees(v, 0, 8.5, -8.5)).toBeTrue();
  expect(knows(v, 0, 12.5, 12.5)).toBeFalse();

  // A map bought, then a memory wiped.
  must(og.sight.reveal(v, 0, at(12.5, 12.5), 2));
  expect(knows(v, 0, 12.5, 12.5)).toBeTrue();
  expect(sees(v, 0, 12.5, 12.5)).toBeFalse();
  must(og.sight.forget(v, 0));
  expect(knows(v, 0, -8.5, 0.5)).toBeFalse();
  expect(knows(v, 0, 8.5, -8.5)).toBeTrue(); // still seen
  expect(knows(v, 1, 8.5, 8.5)).toBeTrue(); // another team's memory stays

  expect(og.sight.reveal(v, 40, at(0, 0), 1).ok).toBeFalse();
  expect(og.sight.visible(v, 0, { x: Number.NaN, y: 0, z: 0 }).ok).toBeFalse();
});

test("a system reads what the tick's start cast, and a world without a map answers false", () => {
  const seen: boolean[] = [];
  const Ask = defineSystem("Ask", {}, (_q: unknown, _t: unknown, world: World) => {
    seen.push(must(og.sight.visible(world, 0, at(-8.5, 0.5))), must(og.sight.visible(world, 0, at(-2.5, 0.5))));
  });
  const engine = start([Ask]);
  expect(engine.advance(3).ok).toBeTrue();
  expect(seen).toEqual([true, false, true, false, true, false]);

  const r = createEngine(og.game({ start: () => {} }) as GameToken, {});
  if (!r.ok) throw new Error(r.reason);
  expect(r.value.advance(1).ok).toBeTrue();
  expect(must(og.sight.visible(r.value.world, 0, at(0, 0)))).toBeFalse();
  expect(og.sight.reveal(r.value.world, 0, at(0, 0), 1).ok).toBeFalse();
});

test("what each team explored is world state: the same on every run and back after a restore", () => {
  const run = () => {
    const engine = start();
    expect(engine.advance(5).ok).toBeTrue();
    const snap = engine.probe.snapshot() as { id: string; hash: string };
    const before = engine.hash();
    must(engine.world.set(engine.world.find("scout")!, og.Transform, { position: at(8.5, -8.5) }));
    expect(engine.advance(5).ok).toBeTrue();
    const moved = engine.hash();
    expect(moved).not.toBe(before);
    expect(knows(engine.world, 0, 8.5, -8.5)).toBeTrue();
    engine.probe.restore(snap.id);
    expect(engine.hash()).toBe(before);
    expect(knows(engine.world, 0, 8.5, -8.5)).toBeFalse();
    expect(knows(engine.world, 0, -8.5, 0.5)).toBeTrue();
    return { before, moved };
  };
  expect(run()).toEqual(run());
  // What a team has explored is in the hash: revealing ground moves it.
  const a = start();
  const b = start();
  expect(a.advance(2).ok && b.advance(2).ok).toBeTrue();
  expect(a.hash()).toBe(b.hash());
  must(og.sight.reveal(b.world, 0, at(12.5, 12.5), 2));
  expect(a.hash()).not.toBe(b.hash());
});
