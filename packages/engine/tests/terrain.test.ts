// Terrain through the TypeScript API on the native host (IDL module
// `terrain`, phase 2): a cooked terrain (tests/fixtures/terrain, a 6 m hill
// on a gentle slope, 129 × 129 samples over 64 m) loads with its layer's
// texture and its height field registered with physics; og.terrain.sample
// answers the surface a ball comes to rest on and a character walks up;
// an edit raises the ground under the renderer, physics and sample alike;
// a layer reads back and retunes; og.Terrain draws it; og.TerrainScatter
// grows copies of a mesh over it near the camera; refusals name the fix.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, SHAPE_HEIGHTFIELD, SHAPE_SPHERE, SHAPE_CAPSULE } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { decodeCapture, pixel } from "./support/png.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const fixture = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "terrain");

// At module level, not in a hook: a hook's 5 s would not cover a cook on a
// loaded machine (b2532cd's gate).
{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", fixture]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(fixture, "cooked"))) throw new Error(`og cook ${fixture} wrote no cooked/`);
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The fixture's ground: the hill (6 m, 14 m wide) on a 0.05 slope, 2 m up. */
function ground(x: number, z: number): number {
  return 6 * Math.exp(-(x * x + z * z) / (14 * 14)) + 0.05 * x + 2;
}

type Info = { data: number; geometry: number; columns: number; rows: number; minHeight: number; maxHeight: number; layers: number; size: { x: number; y: number }; collision: boolean };

/** Points the camera from `eye` at `target` and lights the scene with a sun. */
function view(w: World, eye: { x: number; y: number; z: number }, target: { x: number; y: number; z: number }): void {
  const cam = must(w.resource(og.Camera));
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, target);
  cam.fov = 50;
  cam.far = 500;
  const sun = must(w.spawn({ name: "sun" }));
  w.insert(sun, og.DirectionalLight, { intensity: 3 });
  w.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -1, y: -2, z: -1 }) });
}

/** An engine on the fixture with the terrain placed at `origin`, collidable. */
function start(origin = { x: 0, y: 0, z: 0 }, size?: [number, number]): { engine: Engine; info: Info; terrain: Entity } {
  let info: Info | undefined;
  let terrain: Entity | undefined;
  const game = og.game({
    start(world: World) {
      info = must(og.terrain.load(world, "terrain/hill.ogt")) as unknown as Info;
      terrain = must(world.spawn({ name: "hill", position: origin }));
      world.insert(terrain, og.Terrain, { data: info.data as never });
      world.insert(terrain, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry as never });
    },
  });
  const r = createEngine(game as GameToken, { root: fixture });
  if (!r.ok) throw new Error(r.reason);
  if (size) {
    const attached = JSON.parse(native().og_node_attach_offscreen(r.value.id, size[0], size[1])) as { ok: boolean; reason?: string };
    if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  }
  expect(r.value.advance(1).ok).toBeTrue();
  return { engine: r.value, info: info!, terrain: terrain! };
}

test("og.terrain.load answers the terrain, and sample its surface", () => {
  const { engine, info } = start({ x: 10, y: -1, z: 5 });
  const w = engine.world;
  expect([info.columns, info.rows, info.layers, info.collision]).toEqual([129, 129, 1, true]);
  expect([info.size.x, info.size.y]).toEqual([64, 64]);
  expect(info.minHeight).toBeCloseTo(0.4, 2);
  expect(info.maxHeight).toBeCloseTo(8.02, 1);
  // Loading again answers the same terrain.
  expect((must(og.terrain.load(w, "terrain/hill.ogt")) as unknown as Info).data).toBe(info.data);
  // The surface under world points, the terrain standing at (10, -1, 5).
  const at = (x: number, z: number) => must(og.terrain.sample(w, { data: info.data as never, origin: { x: 10, y: -1, z: 5 }, x, z }));
  const top = at(10, 5);
  expect(top.height).toBeCloseTo(ground(0, 0) - 1, 2);
  expect(top.normal.y).toBeGreaterThan(0.99);
  expect(top.inside).toBeTrue();
  const flank = at(10 + 7, 5 + 3);
  expect(flank.height).toBeCloseTo(ground(7, 3) - 1, 1);
  expect(flank.slope).toBeGreaterThan(10);
  expect(flank.normal.x).toBeGreaterThan(0.1); // the hill falls away towards +x
  expect(at(10 + 40, 5).inside).toBeFalse();
  const box = must(og.terrain.bounds(w, info.data as never, { x: 10, y: -1, z: 5 }));
  expect([box.min.x, box.min.z]).toEqual([-22, -27]);
  expect(box.min.y).toBeCloseTo(-0.6, 2);
  expect(box.max.x).toBe(42);
  expect(box.max.y).toBeCloseTo(7.02, 1);
});

test("a turned and scaled terrain: sample, bounds, edit and physics follow its entity's transform", () => {
  // A quarter turn about y and (3, 1.5, 1): the hill's x runs along world -z, three times as long, half again as tall.
  const turn = { x: 0, y: Math.SQRT1_2, z: 0, w: Math.SQRT1_2 };
  const place = { origin: { x: 5, y: 1, z: -3 }, quaternion: turn, scale: { x: 3, y: 1.5, z: 1 } };
  let info: Info | undefined;
  const game = og.game({
    start(world: World) {
      info = must(og.terrain.load(world, "terrain/hill.ogt")) as unknown as Info;
      const e = must(world.spawn({ name: "hill", position: place.origin, quaternion: turn, scale: place.scale }));
      world.insert(e, og.Terrain, { data: info.data as never });
      world.insert(e, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry as never });
    },
  });
  const r = createEngine(game as GameToken, { root: fixture });
  if (!r.ok) throw new Error(r.reason);
  const engine = r.value;
  expect(engine.advance(1).ok).toBeTrue();
  const w = engine.world;
  const data = info!.data as never;
  // World (x, z) to the terrain's own space: undo the turn, then the scale.
  const local = (x: number, z: number) => ({ x: -(z - place.origin.z) / 3, z: x - place.origin.x });
  const expected = (x: number, z: number) => {
    const l = local(x, z);
    return place.origin.y + 1.5 * ground(l.x, l.z);
  };
  const at = (x: number, z: number) => must(og.terrain.sample(w, { data, ...place, x, z }));
  expect(at(5, -3).height).toBeCloseTo(expected(5, -3), 1);
  // 60 m along world -z is 20 m along the hill's x: on it, turned; 60 m along world x is off it.
  const along = at(5, -63);
  expect(along.inside).toBeTrue();
  expect(along.height).toBeCloseTo(expected(5, -63), 1);
  expect(at(65, -3).inside).toBeFalse();
  // The flank 7 m along the hill's x falls away towards it: in the world, towards -z.
  const flank = at(5 + 3, -3 - 21);
  expect(flank.height).toBeCloseTo(expected(5 + 3, -3 - 21), 1);
  expect(flank.normal.z).toBeLessThan(-0.05);
  // The box around the turned corners.
  const box = must(og.terrain.bounds(w, data, place.origin, turn, place.scale));
  expect(box.min.x).toBeCloseTo(-27, 3);
  expect(box.max.x).toBeCloseTo(37, 3);
  expect(box.min.z).toBeCloseTo(-99, 3);
  expect(box.max.z).toBeCloseTo(93, 3);
  expect(box.max.y).toBeCloseTo(1 + 1.5 * info!.maxHeight, 2);
  // Physics collides with the same turned and scaled surface.
  const hit = must(og.physics.raycast(w, { origin: { x: 5, y: 50, z: -63 }, direction: { x: 0, y: -1, z: 0 } }));
  expect(hit?.point.y).toBeCloseTo(expected(5, -63), 1);
  // An edit raises 3 world metres at a world point.
  must(og.terrain.edit(w, data, { ...place, mode: "raise" as never, center: { x: 5, y: 0, z: -63 }, radius: 6, strength: 3, falloff: 0.5 }));
  expect(at(5, -63).height).toBeCloseTo(expected(5, -63) + 3, 1);
  expect(engine.advance(1).ok).toBeTrue();
  const raised = must(og.physics.raycast(w, { origin: { x: 5, y: 50, z: -63 }, direction: { x: 0, y: -1, z: 0 } }));
  expect(raised?.point.y).toBeCloseTo(expected(5, -63) + 3, 1);
});

test("a ball rests on the terrain and a character walks up the hill", () => {
  const { engine } = start();
  const w = engine.world;
  const ball = must(w.spawn({ name: "ball", position: { x: -20, y: 12, z: 10 } }));
  w.insert(ball, og.RigidBody, {});
  w.insert(ball, og.Collider, { shape: SHAPE_SPHERE, radius: 0.5 });
  const hero = must(w.spawn({ name: "hero", position: { x: -24, y: ground(-24, 0) + 1.2, z: 0 } }));
  w.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.35, height: 1 });
  w.insert(hero, og.CharacterController, { maxSlope: 50 });
  expect(engine.advance(30).ok).toBeTrue();
  w.set(hero, og.CharacterController, { move: { x: 3, y: 0, z: 0 } });
  let lowest = Infinity;
  for (let i = 0; i < 240; i++) {
    expect(engine.advance(1).ok).toBeTrue();
    const p = must(w.get(hero, og.Transform)).position;
    lowest = Math.min(lowest, p.y - 0.85 - ground(p.x, p.z));
  }
  const p = must(w.get(hero, og.Transform)).position;
  expect(p.x).toBeGreaterThan(-15); // it went most of its 12 m
  expect(ground(p.x, p.z)).toBeGreaterThan(ground(-24, 0) + 2); // up the hill's flank
  expect(p.y - 0.85).toBeGreaterThan(ground(p.x, p.z) - 0.15);
  expect(lowest).toBeGreaterThan(-0.2);
  expect(must(w.get(hero, og.CharacterController)).grounded).toBeTrue();
  const b = must(w.get(ball, og.Transform)).position;
  const under = ground(b.x, b.z);
  expect(b.y).toBeGreaterThan(under + 0.3);
  expect(b.y).toBeLessThan(under + 0.75);
});

test("an edit raises the ground for sample, physics and the renderer", () => {
  const { engine, info } = start({ x: 0, y: 0, z: 0 }, [160, 90]);
  const w = engine.world;
  // The camera looking down at a flat-ish spot east of the hill.
  view(w, { x: 20, y: 30, z: 32 }, { x: 20, y: 0, z: 20 });
  expect(engine.advance(1).ok).toBeTrue();
  const before = decodeCapture(engine.probe.capture(160, 90));
  const spot = { data: info.data as never, origin: { x: 0, y: 0, z: 0 }, x: 20, z: 20 };
  const h0 = must(og.terrain.sample(w, spot)).height;
  const edit = must(og.terrain.edit(w, info.data as never, { mode: "raise" as never, center: { x: 20, y: 0, z: 20 }, radius: 6, strength: 4, falloff: 0.5 }));
  expect(edit.samples).toBeGreaterThan(100);
  expect(edit.maxHeight).toBeGreaterThan(info.maxHeight - 0.1);
  expect(must(og.terrain.sample(w, spot)).height).toBeCloseTo(h0 + 4, 1);
  // Physics: a ray down meets the raised ground.
  const hit = must(og.physics.raycast(w, { origin: { x: 20, y: 50, z: 20 }, direction: { x: 0, y: -1, z: 0 } }));
  expect(hit?.point.y).toBeCloseTo(h0 + 4, 1);
  // The renderer: the frame changed around the mound and not at the far corners.
  expect(engine.advance(1).ok).toBeTrue();
  const after = decodeCapture(engine.probe.capture(160, 90));
  const diff = (x: number, y: number) => {
    const [a, b] = [pixel(before, x, y), pixel(after, x, y)];
    return Math.abs(a[0]! - b[0]!) + Math.abs(a[1]! - b[1]!) + Math.abs(a[2]! - b[2]!);
  };
  let changed = 0;
  for (let y = 30; y < 60; y++) for (let x = 60; x < 100; x++) if (diff(x, y) > 6) changed++;
  expect(changed).toBeGreaterThan(50);
  // Flattening it back to the old height.
  must(og.terrain.edit(w, info.data as never, { mode: "flatten" as never, center: { x: 20, y: h0, z: 20 }, radius: 8, strength: 1, falloff: 0 }));
  expect(must(og.terrain.sample(w, spot)).height).toBeCloseTo(h0, 1);
});

test("terrain edits are world state: a snapshot keeps them and a restore brings the ground back", () => {
  const { engine, info } = start();
  const w = engine.world;
  const data = info.data as never;
  const spot = { data, origin: { x: 0, y: 0, z: 0 }, x: 20, z: 20 };
  const ray = () => must(og.physics.raycast(w, { origin: { x: 20, y: 50, z: 20 }, direction: { x: 0, y: -1, z: 0 } }))?.point.y ?? NaN;
  const h0 = must(og.terrain.sample(w, spot)).height;
  const untouched = engine.probe.snapshot() as { id: string; hash: string };
  must(og.terrain.edit(w, data, { mode: "raise" as never, center: { x: 20, y: 0, z: 20 }, radius: 6, strength: 3 }));
  expect(engine.hash()).not.toBe(untouched.hash); // the edit is in the world's state
  const raised = engine.probe.snapshot() as { id: string; hash: string };
  expect(must(og.terrain.sample(w, spot)).height).toBeCloseTo(h0 + 3, 1);
  // More edits, then back to the raised ground.
  must(og.terrain.edit(w, data, { mode: "lower" as never, center: { x: 20, y: 0, z: 20 }, radius: 6, strength: 5 }));
  must(og.terrain.edit(w, data, { mode: "paint" as never, layer: 0, center: { x: 0, y: 0, z: 0 }, radius: 4 }));
  expect(engine.advance(2).ok).toBeTrue();
  expect(must(og.terrain.sample(w, spot)).height).toBeCloseTo(h0 - 2, 1);
  expect(engine.probe.restore(raised.id)).toBeTrue();
  expect(engine.hash()).toBe(raised.hash);
  expect(must(og.terrain.sample(w, spot)).height).toBeCloseTo(h0 + 3, 1);
  expect(engine.advance(1).ok).toBeTrue();
  expect(ray()).toBeCloseTo(h0 + 3, 1); // physics stands on the restored ground too
  // Before any edit: the ground as cooked.
  expect(engine.probe.restore(untouched.id)).toBeTrue();
  expect(engine.advance(1).ok).toBeTrue();
  expect(must(og.terrain.sample(w, spot)).height).toBeCloseTo(h0, 3);
  expect(ray()).toBeCloseTo(h0, 1);
  // The same edits on two runs give the same world.
  const replay = start();
  must(og.terrain.edit(replay.engine.world, replay.info.data as never, { mode: "raise" as never, center: { x: 20, y: 0, z: 20 }, radius: 6, strength: 3 }));
  expect(replay.engine.hash()).toBe(raised.hash);
});

test("og.Terrain draws the terrain, and a layer reads back and retunes", () => {
  const { engine, info } = start({ x: 0, y: 0, z: 0 }, [160, 90]);
  const w = engine.world;
  view(w, { x: 0, y: 20, z: 40 }, { x: 0, y: 2, z: 0 });
  expect(engine.advance(1).ok).toBeTrue();
  const drawn = decodeCapture(engine.probe.capture(160, 90));
  // The grass layer: green below the horizon.
  const g = pixel(drawn, 80, 70);
  expect(g[1]!).toBeGreaterThan(g[0]! + 10);
  expect(g[1]!).toBeGreaterThan(g[2]! + 10);
  const layer = must(og.terrain.layer(w, info.data as never, 0));
  expect(layer.tile).toBe(4);
  expect(layer.roughness).toBeCloseTo(0.9, 5);
  must(og.terrain.setLayer(w, info.data as never, 0, { ...layer, color: { r: 2, g: 0.1, b: 0.1 } }));
  expect(must(og.terrain.layer(w, info.data as never, 0)).color?.r).toBe(2);
  expect(engine.advance(1).ok).toBeTrue();
  const red = pixel(decodeCapture(engine.probe.capture(160, 90)), 80, 70);
  expect(red[0]!).toBeGreaterThan(red[1]! + 10);
  // Without og.Terrain nothing is drawn there.
  w.remove(w.find("hill")!, og.Terrain);
  expect(engine.advance(1).ok).toBeTrue();
  const gone = pixel(decodeCapture(engine.probe.capture(160, 90)), 80, 70);
  expect(Math.abs(gone[0]! - red[0]!) + Math.abs(gone[1]! - red[1]!)).toBeGreaterThan(20);
});

test("og.TerrainScatter grows copies of a mesh over the terrain near the camera", () => {
  const { engine, info, terrain } = start({ x: 0, y: 0, z: 0 }, [160, 90]);
  const w = engine.world;
  view(w, { x: 0, y: 30, z: 36 }, { x: 0, y: 0, z: 0 });
  const red = must(og.render.standardMaterial(w, { color: { r: 1, g: 0, b: 0 }, roughness: 1 }));
  const box = must(og.render.boxGeometry(w, { width: 2, height: 2, depth: 2 }));
  const reds = () => {
    expect(engine.advance(1).ok).toBeTrue();
    const img = decodeCapture(engine.probe.capture(160, 90));
    let n = 0;
    for (let y = 0; y < 90; y++) {
      for (let x = 0; x < 160; x++) {
        const [r, g, b] = pixel(img, x, y);
        if (r! > 110 && g! < 60 && b! < 60) n++;
      }
    }
    return n;
  };
  expect(reds()).toBe(0);
  must(w.insert(terrain, og.TerrainScatter, { data: info.data as never, geometry: box, material: red, density: 0.05, distance: 80, fade: 0 }));
  const many = reds();
  expect(many).toBeGreaterThan(300);
  // Within 6 m of the camera's xz (36 m south of the hill, past the
  // terrain's edge): nothing in view.
  w.set(terrain, og.TerrainScatter, { distance: 6 });
  expect(reds()).toBe(0);
  // Half the density, about half the copies.
  w.set(terrain, og.TerrainScatter, { distance: 80, density: 0.025 });
  const half = reds();
  expect(half).toBeGreaterThan(many * 0.3);
  expect(half).toBeLessThan(many * 0.7);
  w.set(terrain, og.TerrainScatter, { density: 0 });
  expect(reds()).toBe(0);
});

test("og.terrain refusals name the fix", () => {
  const { engine, info } = start();
  const w = engine.world;
  const notCooked = og.terrain.load(w, "terrain/none.ogt");
  expect(notCooked.ok).toBeFalse();
  if (!notCooked.ok) expect(notCooked.reason).toContain("og cook");
  const bad = og.terrain.sample(w, { data: 12345 as never, x: 0, z: 0 });
  expect(bad.ok).toBeFalse();
  if (!bad.ok) expect(bad.reason).toContain("og.terrain.load");
  const miss = og.terrain.edit(w, info.data as never, { center: { x: 500, y: 0, z: 0 }, radius: 4 });
  expect(miss.ok).toBeFalse();
  if (!miss.ok) expect(miss.reason).toContain("misses the terrain");
  const layer = og.terrain.layer(w, info.data as never, 3);
  expect(layer.ok).toBeFalse();
  if (!layer.ok) expect(layer.reason).toContain("1 layer(s)");
  const paint = og.terrain.edit(w, info.data as never, { mode: "paint" as never, layer: 2, center: { x: 0, y: 0, z: 0 } });
  expect(paint.ok).toBeFalse();
});
