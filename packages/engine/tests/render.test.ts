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

test("a sprite submitted on some ticks is drawn after those ticks only", () => {
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

const diff = (a: number[], b: number[]) => Math.max(...a.map((v, i) => Math.abs(v - b[i]!)));
const lum = (p: number[]) => 0.2126 * p[0]! + 0.7152 * p[1]! + 0.0722 * p[2]!;
/** Mean luma of a rectangle. */
function meanLuma(img: Image, x0: number, y0: number, x1: number, y1: number): number {
  let s = 0;
  let n = 0;
  for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++, n++) s += lum(pixel(img, x, y));
  return s / n;
}
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

test("screen-space sprites are drawn over the finished frame in CSS pixels, untonemapped and clipped", () => {
  const Hud = defineSystem("RtHud", {}, (_q, _t, world: World) => {
    og.render.sprite(world, { screen: true, position: { x: 10, y: 10, z: 0 }, width: 40, height: 30, anchor: { x: 0, y: 0 }, color: { r: 1, g: 0.5, b: 0 } });
    og.render.sprite(world, { screen: true, position: { x: 80, y: 10, z: 0 }, width: 40, height: 40, anchor: { x: 0, y: 0 }, color: { r: 0, g: 0, b: 1 }, clip: [80, 10, 20, 40] });
  });
  const img = frameOf(og.game({ systems: [Hud], start: (w: World) => void wall(w) }));
  // The authored colour (sRGB of linear 0.5 is 188) though the scene is ACES
  // tone mapped: within one step of the encode's rounding (lavapipe gives 187).
  expect(diff(pixel(img, 25, 20), [255, 188, 0])).toBeLessThan(2);
  expect(isWhite(pixel(img, 25, 20))).toBeFalse();
  // Its top-left corner is (10, 10): the pixel above and left is the scene.
  expect(diff(pixel(img, 9, 20), [255, 188, 0])).toBeGreaterThan(40);
  // The clipped sprite shows its left 20 px only.
  expect(pixel(img, 90, 30)).toEqual([0, 0, 255]);
  expect(diff(pixel(img, 110, 30), [0, 0, 255])).toBeGreaterThan(40);
});

test("PostEffects reach the frame: a screen flash and a vignette", () => {
  const plain = frameOf(og.game({ start: (w: World) => void wall(w) }));
  const flashed = frameOf(
    og.game({
      start: (w: World) => {
        wall(w);
        const p = must(w.resource(og.PostEffects));
        p.flash = 1;
        p.flashColor = { r: 0, g: 1, b: 0 };
      },
    }),
  );
  expect(pixel(flashed, W / 2, H / 2)).toEqual([0, 255, 0]);
  const vignetted = frameOf(
    og.game({
      start: (w: World) => {
        wall(w);
        const p = must(w.resource(og.PostEffects));
        p.vignette = true;
        p.vignetteIntensity = 1;
      },
    }),
  );
  const corner = (img: Image) => meanLuma(img, 0, 0, 12, 12);
  const centre = (img: Image) => meanLuma(img, W / 2 - 6, H / 2 - 6, W / 2 + 6, H / 2 + 6);
  expect(corner(vignetted)).toBeLessThan(corner(plain) * 0.6);
  expect(Math.abs(centre(vignetted) - centre(plain))).toBeLessThan(6);
});

test("a Decal component projects onto the wall", () => {
  const base = (w: World) => wall(w);
  const plain = frameOf(og.game({ start: base }));
  const decal = frameOf(
    og.game({
      start: (w: World) => {
        base(w);
        const e = must(w.spawn({ name: "decal", position: { x: 0, y: 0, z: -6 } }));
        w.set(e, og.Transform, { scale: { x: 3, y: 3, z: 2 } });
        w.insert(e, og.Decal, { color: { r: 1, g: 0, b: 0 } });
      },
    }),
  );
  const [r, g] = pixel(decal, W / 2, H / 2);
  const [r0, g0] = pixel(plain, W / 2, H / 2);
  expect(r).toBeGreaterThan(g + 30);
  expect(Math.abs(r0 - g0)).toBeLessThan(8);
  // Outside the decal's box the wall is unchanged.
  expect(diff(pixel(decal, 4, 4), pixel(plain, 4, 4))).toBeLessThan(6);
});

test("2D lighting: Lighting2D darkens lit layers and a Light2D lights around it", () => {
  const board = (w: World) => {
    const cam = must(w.resource(og.Camera));
    cam.orthographic = true;
    cam.height = 10;
    cam.position = { x: 0, y: 0, z: 10 };
    const s = must(w.resource(og.RenderSettings));
    s.toneMapped = false;
  };
  const Floor = defineSystem("RtFloor", {}, (_q, _t, world: World) => {
    og.render.sprite(world, { position: { x: 0, y: 0, z: 0 }, width: 30, height: 12, color: { r: 0.8, g: 0.8, b: 0.8 } });
  });
  const unlit = frameOf(og.game({ systems: [Floor], start: board }));
  const lit = frameOf(
    og.game({
      systems: [Floor],
      start: (w: World) => {
        board(w);
        const l = must(w.resource(og.Lighting2D));
        l.enabled = true;
        l.ambient = { r: 0, g: 0, b: 0 };
        const e = must(w.spawn({ name: "lamp", position: { x: -6, y: 0, z: 0 } }));
        w.insert(e, og.Light2D, { radius: 5, intensity: 1.5 });
      },
    }),
  );
  const left = meanLuma(lit, 20, H / 2 - 4, 40, H / 2 + 4);
  const right = meanLuma(lit, W - 30, H / 2 - 4, W - 10, H / 2 + 4);
  expect(right).toBeLessThan(10);
  expect(left).toBeGreaterThan(60);
  expect(meanLuma(unlit, W - 30, H / 2 - 4, W - 10, H / 2 + 4)).toBeGreaterThan(150);
});

test("trails, beams and ribbons draw strips", () => {
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

test("material and sprite effects: a hit flash, toon bands, dissolve", () => {
  const flashed = frameOf(og.game({ start: (w: World) => void wall(w, { flash: 1, flashColor: { r: 0, g: 0, b: 1 } }) }));
  const [r, g, b] = pixel(flashed, W / 2, H / 2);
  expect(b).toBeGreaterThan(r + 100);
  expect(b).toBeGreaterThan(g + 100);
  const gone = frameOf(og.game({ start: (w: World) => void wall(w, { dissolve: 1 }) }));
  const plain = frameOf(og.game({ start: (w: World) => void wall(w) }));
  expect(changed(gone, plain)).toBeGreaterThan((W * H) / 2);
  const toon = frameOf(og.game({ start: (w: World) => void wall(w, { toon: true }) }));
  expect(changed(toon, plain, 8)).toBeGreaterThan(100);
  // A white sprite flashed red.
  const Hit = defineSystem("RtHit", {}, (_q, _t, world: World) => {
    og.render.sprite(world, { position: { x: 0, y: 0, z: -4 }, width: 2, height: 2, flash: 1, flashColor: { r: 1, g: 0, b: 0 } });
  });
  const hit = frameOf(og.game({ systems: [Hit], start: (w: World) => void must(w.resource(og.RenderSettings)) }));
  const p = pixel(hit, W / 2, H / 2);
  expect(p[0]!).toBeGreaterThan(p[1]! + 100);
});

test("camera shake moves the view deterministically by the tick", () => {
  const shaken = (trauma: number) =>
    frameOf(
      og.game({
        start: (w: World) => {
          wall(w);
          const e = must(w.spawn({ name: "post", position: { x: 0, y: 0, z: -5 } }));
          w.insert(e, og.Mesh, { geometry: must(og.render.boxGeometry(w, { width: 0.3, height: 3, depth: 0.3 })), material: must(og.render.standardMaterial(w, { color: { r: 1, g: 0.1, b: 0.1 } })) });
          const p = must(w.resource(og.PostEffects));
          p.shakeTrauma = trauma;
          p.shakeMaxOffset = 0.5;
        },
      }),
      10,
    );
  const still = shaken(0);
  const a = shaken(1);
  const b = shaken(1);
  expect(changed(a, still)).toBeGreaterThan(50);
  expect(changed(a, b, 0)).toBe(0);
});
