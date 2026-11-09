// Navigation and AI through the TypeScript API on the native host (IDL
// modules `nav` and `ai`): a grid agent that routes round a wall and a
// restore that replays it, a navmesh built from a floor and boxes whose
// obstacle rebuilds tiles, an agent that plans without driving, and a guard
// on a behaviour tree that patrols, sees, chases at a resource's speed and
// bites through the game's command; a machine restarted by og.ai.restart.
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, f32, f64, u32, AI_INTERRUPTED, AI_SUCCESS, NAV_ARRIVED, NAV_MOVING, NAV_ON_LINK } from "@origingame/engine";
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
  // The bounds are the polygons' (a floor at y 0, kept 0.4 m from its edges), not the floor plus the agent's height.
  expect(Math.abs(info.max.y - info.min.y)).toBeLessThan(0.2);
  expect(info.max.x).toBeGreaterThan(23);
  expect(info.max.x).toBeLessThan(24.001);
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
  ReactiveFallback {
    ReactiveSequence attack {
      check sees
      check targetDistance below 1.2
      command bite {
        cooldown 1
      }
    }
    ReactiveSequence chase {
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

const MOOD = `type behaviour
version 1
---
name "mood"
machine {
  initial calm
  state calm {
    wait 100
    to angry {
      check Pace.bites atLeast Pace.chase
    }
  }
  state angry {
    wait 100
  }
}
`;

test("og.ai.restart: a machine goes back to its initial state at its next think", () => {
  const engine = start(og.game({ resources: [Pace], start: (world: World) => {
    const b = must(og.ai.parse(world, MOOD));
    const e = must(world.spawn({ name: "moody" }));
    world.insert(e, og.AiAgent, { behaviour: b.behaviour });
  } }));
  const w = engine.world;
  const moody = w.find("moody")! as Entity;
  engine.advance(2);
  expect(must(w.get(moody, og.AiAgent)).state).toBe(0);
  // Pace.bites reaches Pace.chase (a check against a field): angry, and it stays so.
  must(w.resource(Pace)).bites = 5;
  engine.advance(2);
  must(w.resource(Pace)).bites = 0;
  engine.advance(2);
  expect(must(w.get(moody, og.AiAgent)).state).toBe(1);
  must(og.ai.restart(w, moody));
  engine.advance(1);
  expect(must(w.get(moody, og.AiAgent)).state).toBe(0);
  expect((og.ai.describe(w, moody) as { state: string }).state).toBe("calm");
});

// G2-4: areas, filters and shapes through the API, and og.NavFilter
// registered only once a game uses it.
const Swamp = defineResource("NavSwamp", { mesh: f64(0), careful: f64(0) });

function swampYard(filtered: boolean) {
  return (world: World) => {
    const info = must(og.nav.build(world, { min: { x: 0, y: 0, z: 0 }, max: { x: 20, y: 0, z: 10 }, cellSize: 0.25, tileSize: 16 }));
    must(world.resource(Swamp)).mesh = info.mesh as unknown as number;
    // Water across x 9–11, open only from z 8 to 10.
    must(og.nav.setArea(world, info.mesh, 1, { shape: { kind: "box" as never, min: { x: 9, y: -1, z: -1 }, max: { x: 11, y: 1, z: 8 } }, area: 3 }));
    const e = must(world.spawn({ name: "careful", position: { x: 2, y: 0, z: 4 } }));
    world.insert(e, og.NavAgent, { nav: info.mesh, target: { x: 18, y: 0, z: 4 }, seeking: true, speed: 4 });
    if (filtered) world.insert(e, og.NavFilter, { excluded: [3] });
  };
}

function types(world: World): string[] {
  return must(world.openSave(must(world.snapshot())))
    .types()
    .map((t) => t.name);
}

test("og.nav: areas reroute filtered paths and agents, shapes carve, and og.NavFilter joins the type table only when used", () => {
  const plain = start(og.game({ resources: [Swamp], start: swampYard(false) }));
  const w = plain.world;
  expect(types(w)).not.toContain("og.NavFilter");
  const mesh = must(w.resource(Swamp)).mesh as never;
  const ends = { nav: mesh, from: { x: 2, y: 0, z: 4 }, to: { x: 18, y: 0, z: 4 } };
  const maxZ = (pts: readonly { z: number }[]) => Math.max(...pts.map((p) => p.z));
  expect(maxZ(must(og.nav.findPath(w, ends)).points)).toBeLessThan(6);
  expect(maxZ(must(og.nav.findPath(w, { ...ends, areaCosts: [1, 1, 1, 10] })).points)).toBeGreaterThan(7.9);
  expect(maxZ(must(og.nav.findPath(w, { ...ends, excluded: [3] })).points)).toBeGreaterThan(7.9);
  // A pillar is round, a convex outline needs three points, an area past 63 is refused.
  must(og.nav.carve(w, mesh, 7, { kind: "cylinder" as never, center: { x: 15, y: 0, z: 2 }, radius: 1, height: 2 }));
  expect(og.nav.nearest(w, mesh, { x: 15, y: 0, z: 2 }, { x: 0.05, y: 1, z: 0.05 })).toBeNull();
  expect(og.nav.nearest(w, mesh, { x: 16.3, y: 0, z: 3.3 }, { x: 0.05, y: 1, z: 0.05 })).not.toBeNull();
  expect(og.nav.carve(w, mesh, 8, { kind: "convex" as never, points: [{ x: 1, y: 0, z: 1 }, { x: 2, y: 0, z: 1 }] }).ok).toBeFalse();
  expect(og.nav.setArea(w, mesh, 2, { shape: { kind: "box" as never }, area: 64 }).ok).toBeFalse();
  must(og.nav.carve(w, mesh, 7));
  expect(og.nav.nearest(w, mesh, { x: 15, y: 0, z: 2 }, { x: 0.05, y: 1, z: 0.05 })).not.toBeNull();

  // The filtered agent goes round by the gap; a save of its world restores
  // in a session that never used og.NavFilter and goes on to the same world.
  const careful = start(og.game({ resources: [Swamp], start: swampYard(true) }));
  const cw = careful.world;
  expect(types(cw)).toContain("og.NavFilter");
  const walker_ = cw.find("careful")!;
  let top = 0;
  for (let t = 0; t < 30; t++) {
    expect(careful.advance(2).ok).toBeTrue();
    const p = must(cw.get(walker_, og.Transform)).position;
    expect(p.x > 9.3 && p.x < 10.7 && p.z < 7.7).toBeFalse();
    top = Math.max(top, p.z);
  }
  must(og.nav.carve(cw, must(cw.resource(Swamp)).mesh as never, 9, { kind: "orientedBox" as never, center: { x: 14, y: 1, z: 9 }, size: { x: 2, y: 2, z: 0.4 }, yaw: 0.7 }));
  const save = must(cw.snapshot());
  for (let t = 0; t < 150; t++) {
    expect(careful.advance(2).ok).toBeTrue();
    const p = must(cw.get(walker_, og.Transform)).position;
    expect(p.x > 9.3 && p.x < 10.7 && p.z < 7.7).toBeFalse();
    top = Math.max(top, p.z);
  }
  const after = careful.hash();
  expect(top).toBeGreaterThan(7.9);
  expect(must(cw.get(walker_, og.NavAgent)).status).toBe(NAV_ARRIVED);
  const fresh = start(og.game({ resources: [Swamp], start: swampYard(false) }));
  expect(types(fresh.world)).not.toContain("og.NavFilter");
  expect(fresh.world.restore(save).ok).toBeTrue();
  expect(fresh.advance(300).ok).toBeTrue();
  expect(fresh.hash()).toBe(after);
});

function linkYard(game: boolean) {
  return (world: World) => {
    const info = must(og.nav.build(world, { min: { x: 0, y: 0, z: 0 }, max: { x: 20, y: 0, z: 10 }, cellSize: 0.25, tileSize: 16 }));
    must(world.resource(Swamp)).mesh = info.mesh as unknown as number;
    // A chasm across x 9–11 from wall to wall: only the link crosses it.
    must(og.nav.carve(world, info.mesh, 1, { kind: "box" as never, min: { x: 9, y: -1, z: -1 }, max: { x: 11, y: 1, z: 11 } }));
    expect(must(og.nav.setLink(world, info.mesh, 1, { start: { x: 8, y: 0, z: 4 }, end: { x: 12, y: 0, z: 4 }, kind: "jump", game }))).toBeTrue();
    const e = must(world.spawn({ name: "jumper", position: { x: 2, y: 0, z: 4 } }));
    world.insert(e, og.NavAgent, { nav: info.mesh, target: { x: 18, y: 0, z: 4 }, seeking: true, speed: 4 });
  };
}

test("og.nav: paths and agents cross off-mesh links, a shut link is not planned, a game crossing waits for finishLink, and og.NavLinkReached joins the type table only with a link", () => {
  expect(types(start(og.game({ resources: [Swamp], start: swampYard(false) })).world)).not.toContain("og.NavLinkReached");

  const eng = start(og.game({ resources: [Swamp], start: linkYard(false) }));
  const w = eng.world;
  // Before any crossing emits an event, the saved declaration must still
  // activate the producer in a world that has never registered it.
  const cold = start(og.game({ resources: [Swamp], start: swampYard(false) }));
  expect(cold.world.restore(must(w.snapshot())).ok).toBeTrue();
  expect(cold.hash()).toBe(eng.hash());
  expect(types(cold.world)).toContain("og.NavLinkReached");
  expect(types(w)).toContain("og.NavLinkReached");
  const mesh = must(w.resource(Swamp)).mesh as never;
  const ends = { nav: mesh, from: { x: 2, y: 0, z: 4 }, to: { x: 18, y: 0, z: 4 } };
  const path = must(og.nav.findPath(w, ends));
  expect(path.complete).toBeTrue();
  expect(path.links).toEqual([1]);
  // Shut, the chasm has no way over; open again, it has.
  must(og.nav.setLinkOpen(w, mesh, 1, false));
  const shut = og.nav.findPath(w, ends);
  expect(!shut.ok || !shut.value.complete || shut.value.links.length === 0).toBeTrue();
  must(og.nav.setLinkOpen(w, mesh, 1, true));
  expect(og.nav.setLink(w, mesh, 2, { start: { x: 8, y: 0, z: 6 }, end: { x: 12, y: 0, z: 6 }, kind: "rope" as never }).ok).toBeFalse();

  const jumper = w.find("jumper")!;
  const reached: number[] = [];
  let onLink = 0;
  for (let t = 0; t < 300; t++) {
    expect(eng.advance(1).ok).toBeTrue();
    for (const ev of w.events(og.NavLinkReached)) reached.push(Number(ev.link));
    if (must(w.get(jumper, og.NavAgent)).status === NAV_ON_LINK) onLink += 1;
  }
  expect(reached).toEqual([1]);
  expect(onLink).toBeGreaterThan(0);
  expect(must(w.get(jumper, og.NavAgent)).status).toBe(NAV_ARRIVED);
  expect(must(w.get(jumper, og.Transform)).position.x).toBeGreaterThan(17);

  // `game`: the agent holds at the start until the game ends the crossing,
  // and a save taken mid-crossing restores into the same world.
  const held = start(og.game({ resources: [Swamp], start: linkYard(true) }));
  const hw = held.world;
  const h = hw.find("jumper")!;
  expect(held.advance(200).ok).toBeTrue();
  expect(must(hw.get(h, og.NavAgent)).status).toBe(NAV_ON_LINK);
  expect(must(hw.get(h, og.Transform)).position.x).toBeLessThan(9);
  const save = must(hw.snapshot());
  expect(og.nav.finishLink(hw, h)).toBeTrue();
  expect(held.advance(200).ok).toBeTrue();
  expect(must(hw.get(h, og.NavAgent)).status).toBe(NAV_ARRIVED);
  const after = held.hash();
  const fresh = start(og.game({ resources: [Swamp], start: swampYard(false) }));
  expect(fresh.world.restore(save).ok).toBeTrue();
  const fh = fresh.world.find("jumper")!;
  expect(og.nav.finishLink(fresh.world, fh)).toBeTrue();
  expect(fresh.advance(200).ok).toBeTrue();
  expect(fresh.hash()).toBe(after);
  expect(og.nav.finishLink(fresh.world, fh)).toBeFalse();
});

function ring(crowd: boolean) {
  return (world: World) => {
    const info = must(og.nav.build(world, { min: { x: 0, y: 0, z: 0 }, max: { x: 20, y: 0, z: 10 }, cellSize: 0.25, tileSize: 16 }));
    must(world.resource(Swamp)).mesh = info.mesh as unknown as number;
    const r = Math.SQRT1_2;
    const d = [[1, 0], [r, r], [0, 1], [-r, r], [-1, 0], [-r, -r], [0, -1], [r, -r]] as const;
    d.forEach(([x, z], k) => {
      const e = must(world.spawn({ name: `walker${k}`, position: { x: 10 + 4 * x, y: 0, z: 5 + 4 * z } }));
      world.insert(e, og.NavAgent, { nav: info.mesh, target: { x: 10 - 4 * x, y: 0, z: 5 - 4 * z }, seeking: true, speed: 2, radius: 0.4 });
      if (crowd) world.insert(e, og.Crowd, {});
    });
  };
}

test("og.Crowd: eight agents swap places across a ring without touching, deterministically, and og.Crowd joins the type table only when used", () => {
  expect(types(start(og.game({ resources: [Swamp], start: ring(false) })).world)).not.toContain("og.Crowd");
  const run = () => {
    const eng = start(og.game({ resources: [Swamp], start: ring(true) }));
    const w = eng.world;
    const walkers = Array.from({ length: 8 }, (_, k) => w.find(`walker${k}`)!);
    let closest = Infinity;
    let save = "";
    let mid = "";
    // Every agent is home by tick 846.
    for (let t = 0; t < 870; t++) {
      if (t === 90) save = must(w.snapshot());
      if (t === 300) mid = eng.hash();
      expect(eng.advance(1).ok).toBeTrue();
      const ps = walkers.map((e) => must(w.get(e, og.Transform)).position);
      for (let i = 0; i < 8; i++) for (let j = i + 1; j < 8; j++) closest = Math.min(closest, Math.hypot(ps[i]!.x - ps[j]!.x, ps[i]!.z - ps[j]!.z));
    }
    for (const e of walkers) {
      const a = must(w.get(e, og.NavAgent));
      const p = must(w.get(e, og.Transform)).position;
      expect(Math.hypot(p.x - a.target.x, p.z - a.target.z)).toBeLessThan(0.5);
    }
    return { eng, closest, save, mid, hash: eng.hash() };
  };
  const a = run();
  expect(types(a.eng.world)).toContain("og.Crowd");
  expect(a.closest).toBeGreaterThan(0.76);
  expect(run().hash).toBe(a.hash);
  // A save from mid-swap (tick 90) restores where og.Crowd was never used
  // and is the same world 210 ticks on, with the agents still crossing.
  const fresh = start(og.game({ resources: [Swamp], start: swampYard(false) }));
  expect(fresh.world.restore(a.save).ok).toBeTrue();
  expect(fresh.advance(210).ok).toBeTrue();
  expect(fresh.hash()).toBe(a.mid);
}, 30_000); // two 870-tick crowds: about 1 s alone, over 8 s on a loaded gate

const RELOADER = `type behaviour
version 1
---
name "reloader"
tree {
  ReactiveSequence {
    action reload 2 fast
    set reloaded true
  }
}
`;

test("og.ai: a game action runs until og.ai.finish, its events carry the name and result, and they join the type table only with an action", () => {
  expect(types(start(og.game({ resources: [Pace], start: (world: World) => void must(og.ai.parse(world, MOOD)) })).world)).not.toContain("og.AiActionStarted");

  let subtrees: readonly string[] | undefined;
  const eng = start(og.game({ start: (world: World) => {
    const b = must(og.ai.parse(world, RELOADER));
    subtrees = b.subtrees;
    const e = must(world.spawn({ name: "gunner" }));
    world.insert(e, og.AiAgent, { behaviour: b.behaviour });
  } }));
  const w = eng.world;
  // Parsing enables both producers before either queue holds an event.
  // Opening and restoring this save must preserve that activation.
  const initial = eng.hash();
  expect(w.restore(must(w.snapshot())).ok).toBeTrue();
  expect(eng.hash()).toBe(initial);
  expect(types(w)).toContain("og.AiActionStarted");
  expect(types(w)).toContain("og.AiActionStopped");
  const gunner = w.find("gunner")! as Entity;
  const started: string[] = [];
  const stopped: [string, number][] = [];
  const read = () => {
    for (const ev of w.events(og.AiActionStarted)) started.push(`${ev.name} ${ev.args}`);
    for (const ev of w.events(og.AiActionStopped)) stopped.push([ev.name, ev.result]);
  };
  for (let t = 0; t < 30; t++) {
    expect(eng.advance(1).ok).toBeTrue();
    read();
  }
  // Started once, and still running: nothing finished it.
  expect(started).toEqual(["reload 2 fast"]);
  expect(stopped).toEqual([]);
  expect(og.ai.finish(w, gunner, true, "jump")).toBeFalse();
  expect(og.ai.finish(w, gunner, true, "reload")).toBeTrue();
  for (let t = 0; t < 30; t++) {
    expect(eng.advance(1).ok).toBeTrue();
    read();
  }
  expect(stopped[0]).toEqual(["reload", AI_SUCCESS]);
  // The tree loops: the next reload starts; a restart interrupts it.
  expect(started.length).toBeGreaterThan(1);
  const before = stopped.length;
  must(og.ai.restart(w, gunner));
  for (let t = 0; t < 30; t++) {
    expect(eng.advance(1).ok).toBeTrue();
    read();
  }
  expect(stopped.slice(before).some(([, r]) => r === AI_INTERRUPTED)).toBeTrue();
  expect(subtrees).toEqual([]);
});
