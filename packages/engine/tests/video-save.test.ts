// Saves written before og.VideoPlayer's f64 time (G3-2: version 1, an f32
// `time`, registered in every world) restore on this build without
// `migrate`: the engine's step (save_migrations, og.VideoPlayer 1 -> 2)
// widens a saved player's time and keeps its other fields, and drops the
// type from a save that holds no player, so that world restores as one
// that never had it (og.VideoPlayer is lazy now). The fixture is
// tests/fixtures/video-v1/save.json, written by
// tests/support/video-save-v1.ts at 7998bdb1.
//
// Ways it could fail, and the check that catches each:
// - the old save refused for the retyped field → `restored.ok` false
// - the time read as the f32's bytes, or reset to 0 → `time` is not 2.5
// - the other fields shifted by the wider time → speed, loop or playing wrong
// - a save without a player registers the type on restore (its hash then
//   differs from the same game run here to the same tick)
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { SAVE_AT, Ticks, plain, player } from "./support/video-save-game.ts";
import { asSaved } from "./support/saved-defaults.ts";

const fixture = JSON.parse(readFileSync(join(dirname(fileURLToPath(import.meta.url)), "fixtures", "video-v1", "save.json"), "utf8")) as {
  build: string;
  tick: number;
  player: { save: string; hash: string };
  plain: { save: string; hash: string };
};

beforeAll(() => {
  loadNativeHost();
});

function engineOf(game: unknown): Engine {
  const created = createEngine(game as GameToken);
  if (!created.ok) throw new Error(created.reason);
  return created.value;
}

test("a save with an f32 VideoPlayer.time restores with its player intact", () => {
  const engine = engineOf(plain);
  engine.advance(1);
  expect(engine.world.restore(fixture.player.save)).toEqual({ ok: true, value: null });
  expect(engine.world.tick).toBe(fixture.tick);
  const screen = engine.world.find("screen");
  expect(screen).not.toBeNull();
  const p = engine.world.get(screen!, og.VideoPlayer);
  if (!p.ok) throw new Error(p.reason);
  expect([p.value.time, p.value.speed, p.value.loop, p.value.playing]).toEqual([2.5, 0.5, true, false]);
  engine.advance(5);
  const t = engine.world.resource(Ticks);
  expect(t.ok && t.value.n).toBe(SAVE_AT + 5);
  engine.destroy();
});

test("a save without a player restores as a world that never had the type", () => {
  const here = engineOf(plain);
  here.advance(SAVE_AT);
  const restored = engineOf(plain);
  restored.advance(1);
  expect(restored.world.restore(fixture.plain.save)).toEqual({ ok: true, value: null });
  expect(restored.world.tick).toBe(fixture.tick);
  // The SSR defaults moved since the fixture was written; the save keeps its own.
  asSaved(here);
  expect(restored.hash()).toBe(here.hash());
  // The old build registered og.VideoPlayer in every world, so its hash of
  // the same world was another (the one move of a world without video).
  expect(fixture.plain.hash).not.toBe(here.hash());
  here.destroy();
  restored.destroy();
});
