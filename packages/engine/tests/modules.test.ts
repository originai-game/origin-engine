// The engine modules through the TypeScript API on the native host: physics
// (bodies fall and rest, events reach systems, scene queries inside and
// outside systems), the UI (a mounted document clicked through the probe's
// command channel, dumped, its press part of the world state), camera
// controllers (og.systems.cameras), and the world calls that run inside
// systems through the system context.
import { beforeAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { cookProject } from "./support/og-bin.ts";

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
  world.insert(floor, og.Collider, { shape: "box", size: { x: 20, y: 1, z: 20 } });
  const box = must(world.spawn({ name: "box", position: { x: 0, y: 3, z: 0 } }));
  world.insert(box, og.RigidBody);
  world.insert(box, og.Collider, { shape: "box" });
}

const Seen = defineResource("MdSeen", { collisions: u32(), groundHits: u32(), groundDistance: f32() });
const Watch = defineSystem("MdWatch", { reads: [og.CollisionStarted], writes: [Seen] }, (_q, _t, world) => {
  const seen = must(world.resource(Seen));
  const started = world.events(og.CollisionStarted);
  if (started.length > 0) seen.collisions += started.length;
  const hit = must(og.physics.castRay(world, { origin: { x: 5, y: 5, z: 0 }, direction: { x: 0, y: -1, z: 0 } }));
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
  const down = must(og.physics.castRay(engine.world, { origin: { x: 0, y: 10, z: 0 } }));
  expect(down?.entity).toBe(box);
  expect(down!.point.y).toBeCloseTo(t.position.y + 0.5, 3);
  const touching = must(og.physics.overlap(engine.world, { shape: { shape: "box", size: { x: 0.5, y: 0.5, z: 0.5 } }, position: { x: 0, y: 0.5, z: 0 } }));
  expect(touching).toEqual([box]);
});

test("a compound of child colliders is one body, and raycastAll and projectPoint see its parts", () => {
  const engine = start(
    og.game({
      start: (world: World) => {
        floorAndBox(world);
        // A dumbbell 5 m to the right: a body with two child spheres.
        const bell = must(world.spawn({ name: "bell", position: { x: 5, y: 3, z: 0 } }));
        world.insert(bell, og.RigidBody);
        for (const [name, x] of [["left", -1], ["right", 1]] as const) {
          const ball = must(world.spawn({ name, position: { x, y: 0, z: 0 } }));
          world.insert(ball, og.Collider, { shape: "sphere", radius: 0.5 });
          world.insert(ball, og.Parent, { entity: bell });
        }
      },
    }),
  );
  expect(engine.advance(180).ok).toBeTrue();
  const w = engine.world;
  const [bell, left, right, box, floor] = ["bell", "left", "right", "box", "floor"].map((n) => w.find(n)!);
  // It rests on both spheres, its centre at their radius.
  expect(must(w.get(bell, og.Transform)).position.y).toBeCloseTo(0.5, 2);
  // A ray along the floor's top at the spheres' centre height crosses the box, then both spheres.
  const hits = must(og.physics.castRayAll(w, { origin: { x: -5, y: 0.5, z: 0 }, direction: { x: 1, y: 0, z: 0 } }));
  expect(hits.map((h) => h.entity)).toEqual([box, left, right]);
  expect(hits.map((h) => h.body)).toEqual([box, bell, bell]);
  expect(hits[1]!.distance).toBeCloseTo(8.5, 2);
  expect(must(og.physics.castRayAll(w, { origin: { x: -5, y: 0.5, z: 0 }, direction: { x: 1, y: 0, z: 0 }, maxHits: 1 })).length).toBe(1);
  // A point above the right sphere projects onto its top.
  const p = must(og.physics.projectPoint(w, { point: { x: 6, y: 3, z: 0 } }));
  expect(p?.entity).toBe(right);
  expect(p!.point.y).toBeCloseTo(1, 2);
  expect(p!.normal.y).toBeCloseTo(1, 3);
  expect(p!.inside).toBeFalse();
  expect(must(og.physics.projectPoint(w, { point: { x: 0, y: -0.25, z: 0 } }))?.entity).toBe(floor);
  expect(must(og.physics.projectPoint(w, { point: { x: 6, y: 3, z: 0 }, maxDistance: 1 }))).toBeNull();
});

const Slept = defineResource("MdSlept", { slept: u32(), woke: u32() });
const WatchSleep = defineSystem("MdWatchSleep", { reads: [og.BodySlept, og.BodyWoke], writes: [Slept] }, (_q, _t, world) => {
  const seen = must(world.resource(Slept));
  seen.slept += world.events(og.BodySlept).length;
  seen.woke += world.events(og.BodyWoke).length;
});

test("with sleepTicks a resting box sleeps, says so, and wakes when pushed", () => {
  const engine = start(
    og.game({
      resources: [Slept],
      systems: [WatchSleep],
      start: (world: World) => {
        must(world.resource(og.Physics)).sleepTicks = 30;
        floorAndBox(world);
      },
    }),
  );
  expect(engine.advance(240).ok).toBeTrue();
  const w = engine.world;
  const box = w.find("box")!;
  expect(must(w.get(box, og.RigidBody)).sleeping).toBeTrue();
  expect(must(w.resource(Slept))).toMatchObject({ slept: 1, woke: 0 });
  must(og.physics.applyImpulse(w, box, { x: 2, y: 0, z: 0 }));
  expect(engine.advance(3).ok).toBeTrue();
  expect(must(w.get(box, og.RigidBody)).sleeping).toBeFalse();
  expect(must(w.resource(Slept)).woke).toBe(1);
  expect(must(w.get(box, og.Transform)).position.x).toBeGreaterThan(0.01);
});

const Broke = defineResource("MdBroke", { count: u32(), joint: u32() });
const WatchBreak = defineSystem("MdWatchBreak", { reads: [og.JointBroken], writes: [Broke] }, (_q, _t, world) => {
  const seen = must(world.resource(Broke));
  for (const e of world.events(og.JointBroken)) {
    seen.count += 1;
    seen.joint = e.joint === world.find("heavy") ? 1 : 2;
  }
});

test("a conveyor hook carries a box, and a joint past its breakForce breaks once", () => {
  const engine = start(
    og.game({
      resources: [Broke],
      systems: [WatchBreak],
      start: (world: World) => {
        const floor = must(world.spawn({ name: "floor", position: { x: 0, y: -0.5, z: 0 } }));
        world.insert(floor, og.Collider, { shape: "box", size: { x: 40, y: 1, z: 40 } });
        world.insert(floor, og.ContactHook, { surfaceVelocity: { x: 1, y: 0, z: 0 } });
        const box = must(world.spawn({ name: "box", position: { x: 0, y: 0.25, z: 0 } }));
        world.insert(box, og.RigidBody);
        world.insert(box, og.Collider, { shape: "box", size: { x: 0.5, y: 0.5, z: 0.5 } });
        // 30 kg on a 200 N ball joint 1 m under a world point: about 294 N.
        const heavy = must(world.spawn({ name: "heavy", position: { x: 10, y: 4, z: 0 } }));
        world.insert(heavy, og.RigidBody, { mass: 30 });
        world.insert(heavy, og.Collider, { shape: "sphere", radius: 0.2 });
        world.insert(heavy, og.Joint, { kind: "ball", anchor: { x: 0, y: 1, z: 0 }, otherAnchor: { x: 10, y: 5, z: 0 }, breakForce: 200 });
      },
    }),
  );
  expect(engine.advance(120).ok).toBeTrue();
  const w = engine.world;
  expect(must(w.get(w.find("box")!, og.Velocity)).linear.x).toBeCloseTo(1, 1);
  expect(must(w.resource(Broke))).toMatchObject({ count: 1, joint: 1 });
  expect(must(w.get(w.find("heavy")!, og.Joint)).broken).toBeTrue();
});

test("a drivetrain truck on six wheel children shifts up through its gears", () => {
  const engine = start(
    og.game({
      start: (world: World) => {
        const floor = must(world.spawn({ name: "floor", position: { x: 0, y: -0.5, z: 0 } }));
        world.insert(floor, og.Collider, { shape: "box", size: { x: 400, y: 1, z: 400 } });
        const truck = must(world.spawn({ name: "truck", position: { x: 0, y: 1.2, z: 0 } }));
        world.insert(truck, og.RigidBody, { mass: 1500 });
        world.insert(truck, og.Collider, { shape: "box", size: { x: 1.8, y: 0.6, z: 5 } });
        world.insert(truck, og.Vehicle, { model: "drivetrain" });
        let n = 0;
        for (const z of [2, 0, -2]) {
          for (const x of [0.8, -0.8]) {
            const wheel = must(world.spawn({ name: `wheel-${n++}`, parent: truck, position: { x, y: -0.2, z } }));
            world.insert(wheel, og.VehicleWheel, { steer: z > 0 ? 1 : 0, drive: z <= 0 });
          }
        }
      },
    }),
  );
  expect(engine.advance(60).ok).toBeTrue();
  const w = engine.world;
  const truck = w.find("truck")!;
  w.set(truck, og.Vehicle, { throttle: 1 });
  expect(engine.advance(480).ok).toBeTrue();
  const v = must(w.get(truck, og.Vehicle));
  expect(v.contacts).toBe(6);
  expect(v.gear).toBeGreaterThanOrEqual(2);
  expect(v.rpm).toBeGreaterThan(900);
  expect(v.speed).toBeGreaterThan(5);
  const wheels = must(og.physics.vehicleWheels(w, truck));
  const ids = Array.from({ length: 6 }, (_, i) => w.find(`wheel-${i}`)!).sort((a, b) => Number(a) - Number(b));
  expect(wheels.map((x) => x.entity)).toEqual(ids);
});

test("a soft ball lands on the floor and keeps its volume", () => {
  const engine = start(
    og.game({
      start: (world: World) => {
        const floor = must(world.spawn({ name: "floor", position: { x: 0, y: -0.5, z: 0 } }));
        world.insert(floor, og.Collider, { shape: "box", size: { x: 40, y: 1, z: 40 } });
        const ball = must(world.spawn({ name: "jelly", position: { x: 0, y: 3, z: 0 } }));
        world.insert(ball, og.SoftBody, { shape: "sphere", radius: 0.5, stiffness: 0.3 });
      },
    }),
  );
  expect(engine.advance(240).ok).toBeTrue();
  const w = engine.world;
  const ball = w.find("jelly")!;
  const s = must(w.get(ball, og.SoftBody));
  expect(Math.abs(s.volume - 1)).toBeLessThan(0.05);
  const surface = must(og.physics.softBodyPoints(w, ball));
  expect(surface.points.length).toBe(162);
  expect(surface.indices.length).toBe(960);
  const low = Math.min(...surface.points.map((p) => p.y));
  expect(low).toBeGreaterThan(-0.02);
  expect(low).toBeLessThan(0.05);
  expect(s.center.y).toBeGreaterThan(0.3);
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
  if (og.ui.docEvents(world).some((e) => e.kind === "click" && e.name === "Go")) c.n += 1;
  must(og.ui.scope(world, "md").writable("clicks", 0).set(c.n));
});

/** A cooked project holding the panel document: a count and a Go button. */
let panelRoot = "";
function panelProject(): string {
  if (panelRoot) return panelRoot;
  const root = mkdtempSync(join(tmpdir(), "og-md-panel-"));
  mkdirSync(join(root, "assets"));
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Panel"\nid "md-panel"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  writeFileSync(join(root, "assets/panel.svelte"), `<meta name="component" content="panel">
<column id="Box" style="padding: 10px"><text id="Hello">clicks {$clicks}</text><button id="Go" onclick={go}>Go</button></column>
`);
  cookProject(root, root);
  panelRoot = root;
  return root;
}

/** An engine whose UI is the panel document, its count a store the system writes. */
function panel(): Engine {
  const r = createEngine(og.game({
    resources: [Clicks],
    systems: [Panel],
    start(world) {
      must(og.load.document(world, "panel.svelte"));
    },
    ui(world) {
      const scope = og.ui.scope(world, "md");
      must(scope.mount("panel", "panel", { clicks: scope.writable("clicks", 0) }));
    },
  }) as unknown as GameToken, { root: panelProject() });
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

test("a document's button is clicked through the command channel and the tree dumped", () => {
  const engine = panel();
  engine.advance(2);
  const tree = engine.probe.dump({ what: "ui" }) as unknown as { elements: Array<{ id: string; text: string; x: number; y: number; width: number; height: number }> };
  const go = tree.elements.find((e) => e.id === "Go")!;
  expect(go.width).toBeGreaterThan(0);
  expect(tree.elements.find((e) => e.id === "Hello")!.text).toBe("clicks 0");
  const report = engine.probe.commands(`0:touch_move:${go.x + go.width / 2},${go.y + go.height / 2};0.05:touch_start;0.1:touch_end`) as unknown as { warnings: string[] };
  expect(report.warnings).toEqual([]);
  engine.advance(30);
  expect(must(engine.world.resource(Clicks)).n).toBe(1);
  const after = engine.probe.dump({ what: "ui" }) as unknown as { elements: Array<{ id: string; text: string }> };
  expect(after.elements.find((e) => e.id === "Hello")!.text).toBe("clicks 1");
});

test("the UI's interaction state is world state: a restore replays a held press", () => {
  const engine = panel();
  engine.advance(2);
  const go = (engine.probe.dump({ what: "ui" }) as unknown as { elements: Array<{ id: string; x: number; y: number; width: number; height: number }> }).elements.find((e) => e.id === "Go")!;
  engine.probe.commands(`0:touch_move:${go.x + go.width / 2},${go.y + go.height / 2};0.05:touch_start`);
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
  // followRotation: the offset is behind the entity. Turned a quarter about +Y it faces +X, so six behind is -X.
  const h = Math.SQRT1_2;
  const behind = start(
    og.game({
      systems: [...og.systems.cameras],
      start: (world: World) => {
        const e = must(world.spawn({ name: "rig", position: { x: 1, y: 2, z: 3 }, quaternion: { x: 0, y: h, z: 0, w: h } }));
        world.insert(e, og.FollowCamera, { offset: { x: 0, y: 2, z: 6 }, followRotation: true, smoothTime: 0 });
      },
    }),
  );
  behind.advance(1);
  const at = must(behind.world.resource(og.Camera)).position;
  expect(at.x).toBeCloseTo(-5, 5);
  expect(at.y).toBeCloseTo(4, 5);
  expect(at.z).toBeCloseTo(3, 5);
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
