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
