// Navigation and AI through the TypeScript API on the native host (IDL
// modules `nav` and `ai`): a grid agent that routes round a wall and a
// restore that replays it, a navmesh built from a floor and boxes whose
// obstacle rebuilds tiles, an agent that plans without driving, and a guard
// on a behaviour tree that patrols, sees, chases at a resource's speed and
// bites through the game's command.
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, f32, f64, u32, NAV_ARRIVED, NAV_MOVING } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

function start(game: unknown): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** 20 × 10 cells of 1 m from (0, 0); a wall at column 10 from row 0 to 7, so the way round is rows 8–9. */
function walledGrid(world: World): number {
  const costs: number[] = [];
  for (let y = 0; y < 10; y++) for (let x = 0; x < 20; x++) costs.push(x === 10 && y < 8 ? 0 : 1);
  return must(og.nav.grid(world, { width: 20, height: 10, cellSize: 1, costs })) as unknown as number;
}

const Nav = defineResource("AtNav", { grid: f64(0) });

function walker(drive: boolean) {
  return (world: World) => {
    const grid = walledGrid(world);
    must(world.resource(Nav)).grid = grid;
    const e = must(world.spawn({ name: "walker", position: { x: 2.5, y: 0, z: 2.5 } }));
    world.insert(e, og.NavAgent, { nav: grid as never, target: { x: 17.5, y: 0, z: 2.5 }, seeking: true, speed: 6, acceleration: 60, drive });
  };
}

test("og.nav: a grid agent routes round a wall, arrives, and a restored snapshot replays the walk", () => {
  const engine = start(og.game({ resources: [Nav], start: walker(true) }));
  const w = engine.world;
  const grid = must(w.resource(Nav)).grid;
  // The search: unsmoothed, JPS and A* find paths of one length; smoothing never lengthens it; it goes round the wall's end.
  const ends = { nav: grid as never, from: { x: 2.5, y: 0, z: 2.5 }, to: { x: 17.5, y: 0, z: 2.5 } };
  const jps = must(og.nav.findPath(w, { ...ends, smooth: false }));
  const astar = must(og.nav.findPath(w, { ...ends, smooth: false, algorithm: "astar" as never }));
  const smooth = must(og.nav.findPath(w, ends));
  expect(smooth.length <= jps.length + 1e-4).toBeTrue();
  expect(smooth.length).toBeGreaterThan(Math.hypot(7.5, 5.5) * 2 - 0.5);
  expect(jps.complete).toBeTrue();
  expect(jps.algorithm).toBe("jps");
  expect(astar.algorithm).toBe("astar");
  expect(jps.length).toBeCloseTo(astar.length, 3);
  expect(jps.length).toBeGreaterThan(15 + 2);
  expect(Math.max(...jps.points.map((p) => p.z))).toBeGreaterThan(7.9);
  expect(og.nav.cellAt(w, grid as never, { x: 10.5, y: 0, z: 3.5 })).toEqual({ x: 10, y: 3, inside: true });

  engine.advance(20);
  const walker_ = w.find("walker")!;
  // The agent's own plan, mid-walk: round the wall's end too.
  const planned = og.nav.pathOf(w, walker_);
  expect(planned).not.toBeNull();
  expect(Math.max(...planned!.points.map((p) => p.z))).toBeGreaterThan(7.5);
  const snap = engine.probe.snapshot() as { id: string; hash: string };
  engine.advance(280);
  const a = must(w.get(walker_, og.NavAgent));
  const t = must(w.get(walker_, og.Transform));
  expect(a.status).toBe(NAV_ARRIVED);
  expect(t.position.x).toBeCloseTo(17.5, 0);
  expect(t.position.z).toBeCloseTo(2.5, 0);
  const after = engine.hash();
  expect(engine.probe.restore(snap.id)).toBeTrue();
  engine.advance(280);
  expect(engine.hash()).toBe(after);
});

test("og.nav: drive false plans and reports but leaves the motion to the game", () => {
  const engine = start(og.game({ resources: [Nav], start: walker(false) }));
  engine.advance(30);
  const e = engine.world.find("walker")!;
  const a = must(engine.world.get(e, og.NavAgent));
  expect(a.status).toBe(NAV_MOVING);
  expect(a.plans).toBeGreaterThan(0);
  expect(a.remaining).toBeGreaterThan(15);
  expect(must(engine.world.get(e, og.Transform)).position.x).toBe(2.5);
});

test("og.nav: an agent moved off its path (a teleport, a knock-back) plans again from where it is", () => {
  const engine = start(og.game({ resources: [Nav], start: walker(true) }));
  const w = engine.world;
  engine.advance(20);
  const e = w.find("walker")!;
  const before = must(w.get(e, og.NavAgent)).plans;
  must(w.set(e, og.Transform, { position: { x: 14.5, y: 0, z: 9.2 } }) as never);
  engine.advance(1);
  expect(must(w.get(e, og.NavAgent)).plans).toBe(before + 1);
  const leg = og.nav.pathOf(w, e)!;
  expect(leg.points[0]!.x).toBeCloseTo(14.5, 1);
  engine.advance(200);
  expect(must(w.get(e, og.NavAgent)).status).toBe(NAV_ARRIVED);
});

test("og.nav: a navmesh built from a floor and a box routes round it, and an obstacle rebuilds tiles", () => {
  const engine = start(og.game({}));
  const w = engine.world;
  const info = must(
    og.nav.build(w, {
      min: { x: 0, y: 0, z: 0 },
      max: { x: 24, y: 0, z: 24 },
      blocked: [{ min: { x: 10, y: 0, z: 0 }, max: { x: 12, y: 2, z: 18 } }],
      cellSize: 0.25,
      agentRadius: 0.4,
      tileSize: 32,
    }),
  );
  expect(info.polygons).toBeGreaterThan(0);
  expect(info.tiles).toBe(9);
  const from = { x: 4, y: 0, z: 4 };
  const to = { x: 20, y: 0, z: 4 };
  const round = must(og.nav.findPath(w, { nav: info.mesh, from, to }));
  expect(round.complete).toBeTrue();
  expect(round.algorithm).toBe("navmesh");
  expect(Math.max(...round.points.map((p) => p.z))).toBeGreaterThan(18);
  // A box in the gap: the tiles under it are rebuilt and the path is cut off.
  const rebuilt = must(og.nav.setObstacle(w, info.mesh, 1, { min: { x: 9, y: 0, z: 17.5 }, max: { x: 13, y: 2, z: 24 } }));
  expect(rebuilt.tiles.length).toBeGreaterThan(0);
  expect(rebuilt.tiles.length).toBeLessThan(9);
  const cut = must(og.nav.findPath(w, { nav: info.mesh, from, to }));
  expect(cut.complete).toBeFalse();
  must(og.nav.setObstacle(w, info.mesh, 1));
  expect(must(og.nav.findPath(w, { nav: info.mesh, from, to })).length).toBeCloseTo(round.length, 3);
  expect(og.nav.nearest(w, info.mesh, { x: 11, y: 0, z: 5 })).not.toBeNull();
});

const GUARD = `type behaviour
version 1
---
name "guard"
route beat {
  point 2 0 2
  point 6 0 2
}
tree {
  selector {
    sequence attack {
      check sees
      check targetDistance below 1.2
      command bite {
        cooldown 1
      }
    }
    sequence chase {
      check sees
      chase {
        speed Pace.chase
        reach 0.8
      }
    }
    patrol beat {
      wait 0.2
      speed 2
    }
  }
}
`;

const Pace = defineResource("Pace", { chase: f32(5), bites: u32(0), bitBy: f32(0) });

function bite(world: World, args: readonly string[]): void {
  const p = must(world.resource(Pace));
  p.bites += 1;
  p.bitBy = Number(args[0]);
}

function guarded(world: World): void {
  const b = must(og.ai.parse(world, GUARD));
  const g = must(world.spawn({ name: "guard", position: { x: 2, y: 0, z: 2 } }));
  world.insert(g, og.NavAgent, { speed: 2, acceleration: 40, stopDistance: 0.2 });
  world.insert(g, og.AiAgent, { behaviour: b.behaviour });
  world.insert(g, og.Perception, { sight: 6, fov: 360, team: 2, occlude: false });
  const thief = must(world.spawn({ name: "thief", position: { x: 20, y: 0, z: 20 } }));
  world.insert(thief, og.Perceivable, { team: 1 });
}

test("og.ai: a guard on a behaviour tree patrols, sees, chases at a resource's speed and bites through a command", () => {
  const engine = start(og.game({ resources: [Pace], start: guarded, commands: { bite } }));
  const w = engine.world;
  const guard = w.find("guard")! as Entity;
  const thief = w.find("thief")! as Entity;
  engine.advance(60);
  const patrol = og.ai.describe(w, guard) as { behaviour: string; node: string; path: string; sees: boolean; kind: string } | null;
  expect(patrol).not.toBeNull();
  expect(patrol!.behaviour).toBe("guard");
  expect(patrol!.kind).toBe("tree");
  expect(patrol!.path).toContain("patrol beat");
  expect(patrol!.sees).toBeFalse();
  expect(og.ai.get(w, guard, "sees")).toBe(false);
  expect(must(w.resource(Pace)).bites).toBe(0);

  // The thief steps into sight: the guard chases at Pace.chase and bites once in reach.
  must(w.set(thief, og.Transform, { position: { x: 8, y: 0, z: 2 } }));
  engine.advance(2);
  const chase = og.ai.describe(w, guard) as { node: string; sees: boolean; target: number };
  expect(chase.sees).toBeTrue();
  expect(chase.target).toBe(thief);
  const nav = must(w.get(guard, og.NavAgent));
  expect(nav.seeking).toBeTrue();
  expect(nav.speed).toBeCloseTo(5, 5);
  engine.advance(90);
  const pace = must(w.resource(Pace));
  expect(pace.bites).toBeGreaterThan(0);
  expect(pace.bites).toBeLessThan(3);
  expect(pace.bitBy).toBe(guard);
  // The blackboard: the game's own keys, and the built-in ones refused.
  must(og.ai.set(w, guard, "alarm", 3) as never);
  expect(og.ai.get(w, guard, "alarm")).toBe(3);
  expect(og.ai.set(w, guard, "sees", true).ok).toBeFalse();
});
