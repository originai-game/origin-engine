// The engine modules through the TypeScript API on the native host: physics
// (bodies fall and rest, events reach systems, scene queries inside and
// outside systems), the UI (a tree built by systems, clicked through the
// probe's command channel, dumped, and part of the world state), camera
// controllers (og.systems.cameras), and the world calls that run inside
// systems through the system context.
import { beforeAll, expect, test } from "bun:test";
import { og, defineComponent, defineResource, defineSystem, f32, u32, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";
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

function floorAndBox(world: World): void {
  const floor = must(world.spawn({ name: "floor", position: { x: 0, y: -0.5, z: 0 } }));
  world.insert(floor, og.Collider, { shape: SHAPE_BOX, size: { x: 20, y: 1, z: 20 } });
  const box = must(world.spawn({ name: "box", position: { x: 0, y: 3, z: 0 } }));
  world.insert(box, og.RigidBody);
  world.insert(box, og.Collider, { shape: SHAPE_BOX });
}

const Seen = defineResource("MdSeen", { collisions: u32(), groundHits: u32(), groundDistance: f32() });
const Watch = defineSystem("MdWatch", { reads: [og.CollisionStarted], writes: [Seen] }, (_q, _t, world) => {
  const seen = must(world.resource(Seen));
  const started = world.events(og.CollisionStarted);
  if (started.length > 0) seen.collisions += started.length;
  const hit = must(og.physics.raycast(world, { origin: { x: 5, y: 5, z: 0 }, direction: { x: 0, y: -1, z: 0 } }));
  if (hit !== null && hit.entity === world.find("floor")) {
    seen.groundHits += 1;
    seen.groundDistance = hit.distance;
  }
});

test("a dynamic box falls, rests on a static collider, and its collision reaches a system", () => {
  const engine = start(og.game({ resources: [Seen], systems: [Watch], start: floorAndBox }));
  expect(engine.advance(180).ok).toBeTrue();
  const box = engine.world.find("box")!;
  const t = must(engine.world.get(box, og.Transform));
  expect(t.position.y).toBeGreaterThan(0.45);
  expect(t.position.y).toBeLessThan(0.55);
  expect(engine.world.has(box, og.Velocity)).toBeTrue();
  const seen = must(engine.world.resource(Seen));
  expect(seen.collisions).toBe(1);
  // og.physics.queries records the colliders each tick: raycasts work inside systems.
  expect(seen.groundHits).toBe(180);
  expect(seen.groundDistance).toBeCloseTo(5, 5);
  // Between ticks, a query sees the world as it is.
  const down = must(og.physics.raycast(engine.world, { origin: { x: 0, y: 10, z: 0 } }));
  expect(down?.entity).toBe(box);
  expect(down!.point.y).toBeCloseTo(t.position.y + 0.5, 3);
  const touching = must(og.physics.overlap(engine.world, { shape: { shape: SHAPE_BOX, size: { x: 0.5, y: 0.5, z: 0.5 } }, position: { x: 0, y: 0.5, z: 0 } }));
  expect(touching).toEqual([box]);
});

test("a restored snapshot continues the simulation exactly", () => {
  const engine = start(og.game({ resources: [Seen], systems: [Watch], start: floorAndBox }));
  engine.advance(20);
  const snap = engine.probe.snapshot() as { id: string; hash: string };
  engine.advance(60);
  const after = engine.hash();
  expect(engine.probe.restore(snap.id)).toBeTrue();
  expect(engine.hash()).toBe(snap.hash);
  engine.advance(60);
  expect(engine.hash()).toBe(after);
});

const Clicks = defineResource("MdClicks", { n: u32(), value: f32() });
const Panel = defineSystem("MdPanel", { writes: [Clicks] }, (_q, _t, world) => {
  const c = must(world.resource(Clicks));
  og.ui.panel("Box", { padding: { left: 10, top: 10, right: 10, bottom: 10 } }, () => {
    og.ui.label("Hello", `clicks ${c.n}`);
    if (og.ui.button("Go", "Go", { variant: "primary" }).clicked) c.n += 1;
  });
});

test("a UI tree built by a system is clicked through the command channel and dumped", () => {
  const engine = start(og.game({ resources: [Clicks], systems: [Panel] }));
  engine.advance(2);
  const tree = engine.probe.dump({ what: "ui" }) as unknown as { elements: Array<{ id: string; parent: string; text: string; width: number }> };
  const ids = tree.elements.map((e) => `${e.parent}/${e.id}`);
  expect(ids).toEqual(["/Box", "Box/Hello", "Box/Go"]);
  expect(tree.elements[2]!.width).toBeGreaterThan(0);
  const report = engine.probe.commands("0:touch_move_to:Box/Go;0.05:touch_start;0.1:touch_end;0.2:move_to_text:clicks 1;0.3:touch_start;0.35:touch_end") as unknown as { warnings: string[] };
  expect(report.warnings).toEqual([]);
  engine.advance(30);
  expect(must(engine.world.resource(Clicks)).n).toBe(1);
  const after = engine.probe.dump({ what: "ui" }) as unknown as { elements: Array<{ id: string; text: string }> };
  expect(after.elements.find((e) => e.id === "Hello")!.text).toBe("clicks 1");
  // Calls outside a frame do nothing and say so.
  og.ui.button("Stray", "Stray");
  expect(engine.takeLog().some((l) => l.text.includes("og.ui.button was called outside a system"))).toBeTrue();
});

test("the UI's interaction state is world state: a restore replays a held press", () => {
  const engine = start(og.game({ resources: [Clicks], systems: [Panel] }));
  engine.advance(2);
  engine.probe.commands("0:touch_move_to:Go;0.05:touch_start");
  engine.advance(10);
  const snap = engine.probe.snapshot() as { id: string; hash: string };
  engine.probe.commands("0:touch_end");
  engine.advance(5);
  expect(must(engine.world.resource(Clicks)).n).toBe(1);
  expect(engine.probe.restore(snap.id)).toBeTrue();
  expect(must(engine.world.resource(Clicks)).n).toBe(0);
  engine.probe.commands("0:touch_end");
  engine.advance(5);
  expect(must(engine.world.resource(Clicks)).n).toBe(1);
});

const Rig = defineComponent("MdRig", { k: u32() });
function rig(component: unknown, value: Record<string, unknown>) {
  return (world: World) => {
    const e = must(world.spawn({ name: "rig", position: { x: 1, y: 2, z: 3 } }));
    world.insert(e, Rig);
    world.insert(e, component as never, value as never);
  };
}

test("og.systems.cameras: an orbit camera sits at its distance and turns with autoRotate", () => {
  const engine = start(og.game({ systems: [...og.systems.cameras], start: rig(og.OrbitCamera, { distance: 5, polar: Math.PI / 2, autoRotate: true, autoRotateSpeed: 1.5 }) }));
  engine.advance(1);
  const cam = must(engine.world.resource(og.Camera));
  const d = (p: { x: number; y: number; z: number }) => Math.hypot(p.x - 1, p.y - 2, p.z - 3);
  expect(d(cam.position)).toBeCloseTo(5, 4);
  // One tick of autoRotate: azimuth 1.5 / 60 about +Y from the +Z side.
  expect(cam.position.x).toBeCloseTo(1 + 5 * Math.sin(1.5 / 60), 5);
  expect(cam.position.z).toBeCloseTo(3 + 5 * Math.cos(1.5 / 60), 5);
  const first = { ...cam.position };
  engine.advance(60);
  const later = must(engine.world.resource(og.Camera)).position;
  expect(d(later)).toBeCloseTo(5, 4);
  expect(later.y).toBeCloseTo(2, 4);
  expect(Math.hypot(later.x - first.x, later.z - first.z)).toBeGreaterThan(1);
  const o = must(engine.world.get(engine.world.find("rig")!, og.OrbitCamera));
  expect(o.azimuth).toBeCloseTo(1.5 * (61 / 60), 3);
});

test("og.systems.cameras: a follow camera eases to its offset and a 2D follower keeps its dead zone", () => {
  const follow = start(og.game({ systems: [...og.systems.cameras], start: rig(og.FollowCamera, { offset: { x: 0, y: 2, z: 6 }, smoothTime: 0 }) }));
  follow.advance(1);
  expect(must(follow.world.resource(og.Camera)).position).toEqual({ x: 1, y: 4, z: 9 });
  const flat = start(og.game({ systems: [...og.systems.cameras], start: rig(og.Follow2D, { deadZone: { x: 0.5, y: 10 }, smoothTime: 0 }) }));
  flat.advance(1);
  const p = must(flat.world.resource(og.Camera)).position;
  // From x = 0 the camera moves to within the dead zone of the rig (x 1); y stays inside it.
  expect(p.x).toBeCloseTo(0.5, 5);
  expect(p.z).toBe(must(flat.world.resource(og.Camera)).position.z);
});

const Made = defineComponent("MdMade", { k: f32() });
const Spawner = defineSystem("MdSpawner", { writes: [og.Transform, Made] }, (_q, time, world) => {
  if (time.tick !== 1) return;
  const e = must(world.spawn({ name: "made", position: { x: 4, y: 0, z: 0 } }));
  must(world.insert(e, Made, { k: 7 }));
});
const Reader = defineSystem("MdReader", { reads: [Made], writes: [Clicks], after: ["MdSpawner"] }, (q, _t, world) => {
  for (const a of q.archetypes()) {
    for (let i = 0; i < a.count; i++) {
      const got = must(world.get(a.entity(i), Made));
      must(world.resource(Clicks)).value = got.k;
    }
  }
});

test("spawn, insert, get and resource writes run inside systems through the system context", () => {
  const engine = start(og.game({ resources: [Clicks], systems: [Spawner, Reader] }));
  engine.advance(3);
  const made = engine.world.find("made");
  expect(made).not.toBeNull();
  expect(must(engine.world.get(made!, Made)).k).toBe(7);
  expect(must(engine.world.resource(Clicks)).value).toBe(7);
});
