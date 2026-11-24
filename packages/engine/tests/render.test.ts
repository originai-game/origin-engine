import { gpuTest } from "./support/gpu.ts";
// What game code draws, read from frames the native host renders offscreen
// and the probe captures: immediate-mode sprites last exactly the ticks that
// submit them, screen-space sprites and text land in the overlay over the
// finished frame, and the effects games declare from TypeScript reach the
// renderer (each compared with the same scene without the effect).
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { decodeCapture, pixel, type Image } from "./support/png.ts";

beforeAll(() => {
  loadNativeHost();
});

const W = 160;
const H = 90;

function start(game: unknown): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  const attached = JSON.parse(native().og_node_attach_offscreen(r.value.id, W, H)) as { ok: boolean; reason?: string };
  if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  return r.value;
}

function capture(engine: Engine): Image {
  return decodeCapture(engine.probe.capture(W, H));
}

const isWhite = (p: number[]) => p.every((c) => c > 200);

gpuTest("a sprite submitted on some ticks is drawn after those ticks only", () => {
  // Sprites on ticks 0 and 1, none after: the queue must not outlive them.
  const Draw = defineSystem("RtDraw", {}, (_q, _t, world: World) => {
    if (world.tick < 2) og.render.sprite(world, { position: { x: 0, y: 0, z: -5 }, width: 3, height: 3 });
  });
  const engine = start(og.game({ systems: [Draw] }));
  expect(engine.advance(1).ok).toBeTrue();
  expect(isWhite(pixel(capture(engine), W / 2, H / 2))).toBeTrue();
  expect(engine.advance(1).ok).toBeTrue();
  expect(isWhite(pixel(capture(engine), W / 2, H / 2))).toBeTrue();
  expect(engine.advance(3).ok).toBeTrue();
  expect(isWhite(pixel(capture(engine), W / 2, H / 2))).toBeFalse();
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Renders a game for `ticks` and captures it. */
function frameOf(game: unknown, ticks = 1): Image {
  const engine = start(game);
  const stepped = engine.advance(ticks);
  if (!stepped.ok) throw new Error(stepped.reason);
  return capture(engine);
}

const lum = (p: number[]) => 0.2126 * p[0]! + 0.7152 * p[1]! + 0.0722 * p[2]!;
/** How many pixels differ by more than `by` in any channel. */
function changed(a: Image, b: Image, by = 24): number {
  let n = 0;
  for (let i = 0; i < a.rgba.length; i += 4) {
    if (Math.abs(a.rgba[i]! - b.rgba[i]!) > by || Math.abs(a.rgba[i + 1]! - b.rgba[i + 1]!) > by || Math.abs(a.rgba[i + 2]! - b.rgba[i + 2]!) > by) n++;
  }
  return n;
}

/** A grey wall facing the camera 6 m ahead, lit by the default sun. */
function wall(world: World, material?: Parameters<typeof og.render.standardMaterial>[1]): number {
  const geometry = must(og.render.planeGeometry(world, { width: 20, height: 12 }));
  const m = must(og.render.standardMaterial(world, { color: { r: 0.6, g: 0.6, b: 0.6 }, roughness: 0.9, ...material }));
  const e = must(world.spawn({ name: "wall", position: { x: 0, y: 0, z: -6 } }));
  world.insert(e, og.Mesh, { geometry, material: m });
  return m;
}

gpuTest("trails, beams and ribbons draw strips", () => {
  const dark = (w: World) => {
    const s = must(w.resource(og.RenderSettings));
    s.environmentIntensity = 0;
  };
  const plain = frameOf(og.game({ start: dark }), 30);
  // A beam across the view.
  const beam = frameOf(
    og.game({
      start: (w: World) => {
        dark(w);
        const e = must(w.spawn({ name: "emitter", position: { x: -3, y: 0, z: -5 } }));
        w.insert(e, og.Beam, { end: { x: 3, y: 0, z: -5 }, width: 0.3, color: { r: 2, g: 2, b: 2 } });
      },
    }),
  );
  expect(lum(pixel(beam, W / 2, H / 2))).toBeGreaterThan(lum(pixel(plain, W / 2, H / 2)) + 80);
  // A trail behind an entity moved left to right between 30 ticks.
  const comet = start(
    og.game({
      start: (w: World) => {
        dark(w);
        const e = must(w.spawn({ name: "comet", position: { x: -1.5, y: 0, z: -5 } }));
        w.insert(e, og.Trail, { lifetime: 2, widthStart: 0.4, widthEnd: 0.4, opacityEnd: 1, color: { r: 2, g: 2, b: 2 }, colorEnd: { r: 2, g: 2, b: 2 } });
      },
    }),
  );
  const head = comet.world.find("comet")!;
  for (let i = 1; i <= 30; i++) {
    expect(comet.advance(1).ok).toBeTrue();
    must(comet.world.set(head, og.Transform, { position: { x: -1.5 + i * 0.1, y: 0, z: -5 } }));
    capture(comet);
  }
  const trail = capture(comet);
  expect(changed(trail, plain)).toBeGreaterThan(200);
  // A ribbon through three points, submitted every tick.
  const Ribbon = defineSystem("RtRibbon", {}, (_q, _t, world: World) => {
    must(
      og.render.ribbon(world, {
        points: [
          { position: { x: -2, y: -1, z: -5 }, width: 0.3, color: { r: 2, g: 0, b: 0 } },
          { position: { x: 0, y: 1, z: -5 }, width: 0.3, color: { r: 2, g: 0, b: 0 } },
          { position: { x: 2, y: -1, z: -5 }, width: 0.3, color: { r: 2, g: 0, b: 0 } },
        ],
      }),
    );
  });
  const ribbon = frameOf(og.game({ systems: [Ribbon], start: dark }));
  expect(changed(ribbon, plain)).toBeGreaterThan(200);
  expect(og.render.ribbon(start(og.game({})).world, { points: [] }).ok).toBeFalse();
});

