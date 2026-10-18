// The E4 game-feel modules through TypeScript on the native host: og.Sprite
// drawn from components (y sorting, flips through the region, blob
// shadows), tweens of any numeric field with completion events, squash and
// flash presets, slow motion and hit stop reaching physics, tweens and the
// systems' time, camera trauma, punch and kick on the frame only, and a
// snapshot mid-tween and mid-hit-stop resuming into the same world.
//
// Ways this could fail: og.Sprite rows are not drawn (or drawn at the
// origin instead of the entity), sortY is ignored, a shadow is drawn over
// its sprite; a tween writes the wrong field or never reports; squash
// leaves the scale off rest; a hit stop is ignored by physics or tweens, or
// freezes the systems' tick; camera feel moves the world's og.Camera (it
// must stay a frame effect) or never decays; a restored world animates
// differently.
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, defineSystem, f32, u32, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { decodeCapture, pixel, type Image } from "./support/png.ts";

beforeAll(() => {
  loadNativeHost();
});

const W = 160;
const H = 90;

function start(game: unknown, render = false): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  if (render) {
    const attached = JSON.parse(native().og_node_attach_offscreen(r.value.id, W, H)) as { ok: boolean; reason?: string };
    if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  }
  return r.value;
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function capture(engine: Engine): Image {
  return decodeCapture(engine.probe.capture(W, H));
}

/** An orthographic camera 9 units tall at the origin, colours drawn exactly. */
function flat(w: World): void {
  const cam = must(w.resource(og.Camera));
  cam.orthographic = true;
  cam.height = 9;
  cam.position = { x: 0, y: 0, z: 10 };
  cam.background = { r: 0, g: 0, b: 0 };
  must(w.resource(og.RenderSettings)).toneMapped = false;
}

const red = { r: 1, g: 0, b: 0 };
const blue = { r: 0, g: 0, b: 1 };

test("og.Sprite draws at its entity, y-sorted, with a blob shadow under it", () => {
  const scene = (sortY: boolean) => (w: World) => {
    flat(w);
    // Red is lower on the screen (y -0.5) but submitted first.
    const a = must(w.spawn({ name: "a", position: { x: 0, y: -0.5, z: 0 } }));
    w.insert(a, og.Sprite, { width: 3, height: 3, color: red, sortY });
    const b = must(w.spawn({ name: "b", position: { x: 0, y: 0.5, z: 0 } }));
    w.insert(b, og.Sprite, { width: 3, height: 3, color: blue, sortY });
    const c = must(w.spawn({ name: "c", position: { x: 5, y: 0, z: 0 } }));
    w.insert(c, og.Sprite, { width: 1, height: 1, color: { r: 1, g: 1, b: 1 }, anchor: { x: 0.5, y: 0 }, shadow: 1, shadowSize: { x: 3, y: 1 } });
  };
  const sorted = start(og.game({ start: scene(true) }), true);
  sorted.advance(1);
  const img = capture(sorted);
  // The overlap (0, 0) is world y 0: red, lower, draws in front when sorted.
  expect(pixel(img, W / 2, H / 2).slice(0, 3)).toEqual([255, 0, 0]);
  const plain = start(og.game({ start: scene(false) }), true);
  plain.advance(1);
  expect(pixel(capture(plain), W / 2, H / 2).slice(0, 3)).toEqual([0, 0, 255]);
  // The shadow: dark beside the white sprite's feet, the sprite above it untouched.
  const px = (x: number, y: number) => pixel(img, Math.round(W / 2 + (x * H) / 9), Math.round(H / 2 - (y * H) / 9));
  expect(px(5, 0.5).slice(0, 3)).toEqual([255, 255, 255]);
  expect(px(6.2, 0)[0]).toBe(0);
});

const Log = defineResource("FtLog", { done: u32(), tag: u32(), squashX: f32(), y: f32(), ticks: u32(), scaledDt: f32(), hitStop: u32() });
const Watch = defineSystem("FtWatch", { reads: [og.TweenCompleted, og.Transform], writes: [Log] }, (q, time, world) => {
  const log = must(world.resource(Log));
  for (const c of world.events(og.TweenCompleted)) {
    log.done += 1;
    log.tag = c.tag;
  }
  log.ticks += 1;
  log.scaledDt = time.scaledDt;
  log.hitStop = time.hitStop;
  for (const a of q.archetypes()) {
    for (let i = 0; i < a.count; i++) {
      if (a.entity(i) === world.find("mover")) log.y = a.column(og.Transform).position[i * 3 + 1]!;
    }
  }
});

test("tweens animate a field, report once, and squash recovers to rest", () => {
  const engine = start(og.game({
    resources: [Log],
    systems: [Watch],
    start: (w: World) => {
      const e = must(w.spawn({ name: "mover", position: { x: 0, y: 1, z: 0 } }));
      must(og.tween.to(w, { entity: e, field: "position.y", to: [4], duration: 0.5, ease: "linear", tag: 9 }));
    },
  }));
  engine.advance(15);
  const mover = engine.world.find("mover")!;
  expect(must(engine.world.get(mover, og.Transform)).position.y).toBeCloseTo(2.5, 4);
  engine.advance(20);
  expect(must(engine.world.get(mover, og.Transform)).position.y).toBe(4);
  const log = must(engine.world.resource(Log));
  expect([log.done, log.tag]).toEqual([1, 9]);
  // Squash: the scale jumps to the impact and eases back to rest.
  const h = must(og.tween.squash(engine.world, { entity: mover, impact: { x: 1.3, y: 0.7 }, duration: 0.2 }));
  engine.advance(1);
  // The impact shows whole on the first tick, then recovers.
  const s1 = must(engine.world.get(mover, og.Transform)).scale;
  expect(s1.x).toBeCloseTo(1.3, 5);
  expect(s1.y).toBeCloseTo(0.7, 5);
  engine.advance(20);
  expect(must(engine.world.get(mover, og.Transform)).scale).toEqual({ x: 1, y: 1, z: 1 });
  expect(og.tween.active(engine.world, h)).toBeFalse();
  // Refusals name the field.
  const bad = og.tween.to(engine.world, { entity: mover, field: "nope", to: [1] });
  expect(bad.ok).toBeFalse();
  if (!bad.ok) expect(bad.reason).toContain("no field 'nope'");
});

test("a hit stop holds physics and scaled tweens while systems keep ticking, and a restore resumes it", () => {
  const game = og.game({
    resources: [Log],
    systems: [Watch],
    start: (w: World) => {
      const box = must(w.spawn({ name: "box", position: { x: 0, y: 5, z: 0 } }));
      w.insert(box, og.RigidBody);
      w.insert(box, og.Collider, { shape: SHAPE_BOX });
      const mover = must(w.spawn({ name: "mover", position: { x: 0, y: 0, z: 0 } }));
      must(og.tween.to(w, { entity: mover, field: "position.y", to: [10], duration: 2, ease: "linear" }));
    },
  });
  const engine = start(game);
  engine.advance(10);
  const box = engine.world.find("box")!;
  const mover = engine.world.find("mover")!;
  const before = must(engine.world.get(box, og.Transform)).position.y;
  const y = must(engine.world.get(mover, og.Transform)).position.y;
  must(og.time.hitStop(engine.world, 6));
  const snap = engine.probe.snapshot() as { id: string; hash: string };
  engine.advance(6);
  expect(must(engine.world.get(box, og.Transform)).position.y).toBe(before);
  expect(must(engine.world.get(mover, og.Transform)).position.y).toBe(y);
  const log = must(engine.world.resource(Log));
  expect(log.ticks).toBe(16);
  expect(log.scaledDt).toBe(0);
  expect(must(engine.world.resource(og.Time)).hitStop).toBe(0);
  engine.advance(1);
  expect(must(engine.world.get(box, og.Transform)).position.y).toBeLessThan(before);
  // Slow motion halves the scaled step the systems see.
  must(og.time.setScale(engine.world, 0.5));
  engine.advance(1);
  expect(must(engine.world.resource(Log)).scaledDt).toBeCloseTo(0.5 / 60, 6);
  expect(og.time.setScale(engine.world, -1).ok).toBeFalse();
  const after = engine.hash();
  // The snapshot taken at the start of the hit stop resumes into the same world.
  expect(engine.probe.restore(snap.id)).toBeTrue();
  engine.advance(7);
  must(og.time.setScale(engine.world, 0.5));
  engine.advance(1);
  expect(engine.hash()).toBe(after);
});

test("camera trauma, punch and kick change the frame, decay, and leave the world's camera alone", () => {
  const Row = defineSystem("FtRow", {}, (_q, _t, world: World) => {
    for (let i = -6; i <= 6; i += 2) og.render.sprite(world, { position: { x: i, y: 0, z: 0 }, width: 1, height: 6, color: { r: 1, g: 1, b: 1 } });
  });
  // Antialiasing off: TAA would blend the next capture with the shaken one.
  const render = { antialias: "off" as const };
  const still = start(og.game({ systems: [Row], start: flat, render }), true);
  still.advance(2);
  const base = capture(still);
  const kicked = start(og.game({ systems: [Row], start: flat, render }), true);
  kicked.advance(1);
  must(og.camera.kick(kicked.world, { x: 1, y: 0, z: 0 }, 1));
  must(og.camera.punch(kicked.world, 0.3));
  must(og.camera.shake(kicked.world, 0.8));
  kicked.advance(1);
  const cam = must(kicked.world.resource(og.Camera));
  expect(cam.position).toEqual({ x: 0, y: 0, z: 10 });
  expect(cam.trauma).toBeCloseTo(0.8, 6);
  expect(cam.kick.x).toBeCloseTo(1, 6);
  let differ = 0;
  const a = capture(kicked);
  for (let i = 0; i < a.rgba.length; i += 4) if (a.rgba[i] !== base.rgba[i]) differ++;
  expect(differ).toBeGreaterThan(500);
  // Trauma drains at traumaDecay per second; kick and punch spring back.
  kicked.advance(60);
  const later = must(kicked.world.resource(og.Camera));
  expect(later.trauma).toBeCloseTo(Math.max(0, 0.8 - 1.5), 6);
  expect(later.kick.x).toBe(0);
  expect(later.punch).toBe(0);
  const settled = capture(kicked);
  let left = 0;
  for (let i = 0; i < settled.rgba.length; i += 4) if (settled.rgba[i] !== base.rgba[i]) left++;
  expect(left).toBe(0);
  expect(og.camera.shake(kicked.world, 2).ok).toBeFalse();
});

test("og.tween.flash whitens an og.Sprite and fades back", () => {
  const engine = start(og.game({
    start: (w: World) => {
      flat(w);
      const e = must(w.spawn({ name: "hit" }));
      w.insert(e, og.Sprite, { width: 4, height: 4, color: red });
    },
  }), true);
  engine.advance(1);
  const e = engine.world.find("hit")!;
  must(og.tween.flash(engine.world, { entity: e, duration: 0.2 }));
  engine.advance(1);
  expect(pixel(capture(engine), W / 2, H / 2).slice(0, 3)).toEqual([255, 255, 255]);
  engine.advance(15);
  expect(pixel(capture(engine), W / 2, H / 2).slice(0, 3)).toEqual([255, 0, 0]);
  expect(og.tween.flash(engine.world, { entity: engine.world.spawn({}).ok ? 999999 : 0 }).ok).toBeFalse();
});
