// og.VideoPlayer in a save: its f64 `time` and its other fields come back
// from a save this build wrote, and a world without a player saves and
// restores as one that never had the type (og.VideoPlayer is lazy). Saves
// from before engine save format 3 are refused without conversion
// (save-versions.test.ts); there is no migration to test.
//
// Ways it could fail, and the check that catches each:
// - the time narrowed to f32 (or reset to 0) → `time` is not 2.5 + 2^-30
// - the other fields shifted by the wider time → speed, loop or playing wrong
// - a restore registers og.VideoPlayer in a world that had none → its hash
//   differs from the same game run to the same tick
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { SAVE_AT, TIME, Ticks, plain, player } from "./support/video-save-game.ts";

beforeAll(() => {
  loadNativeHost();
});

function engineOf(game: unknown): Engine {
  const created = createEngine(game as GameToken);
  if (!created.ok) throw new Error(created.reason);
  return created.value;
}

function saved(game: unknown): { save: string; hash: string } {
  const e = engineOf(game);
  e.advance(SAVE_AT);
  const save = e.world.snapshot();
  if (!save.ok) throw new Error(save.reason);
  const hash = e.hash();
  e.destroy();
  return { save: save.value, hash };
}

test("a saved VideoPlayer restores with its f64 time and every field", () => {
  const { save } = saved(player);
  const engine = engineOf(plain);
  engine.advance(1);
  expect<unknown>(engine.world.restore(save)).toEqual({ ok: true, value: null });
  expect(engine.world.tick).toBe(SAVE_AT);
  const p = engine.world.get(engine.world.find("screen")!, og.VideoPlayer);
  if (!p.ok) throw new Error(p.reason);
  expect([p.value.time, p.value.speed, p.value.loop, p.value.playing]).toEqual([TIME, 0.5, true, false]);
  engine.advance(5);
  const t = engine.world.resource(Ticks);
  expect(t.ok && t.value.n).toBe(SAVE_AT + 5);
  engine.destroy();
});

test("a world without a player restores as one that never had the type", () => {
  const { save, hash } = saved(plain);
  const restored = engineOf(plain);
  restored.advance(1);
  expect<unknown>(restored.world.restore(save)).toEqual({ ok: true, value: null });
  expect(restored.hash()).toBe(hash);
  restored.destroy();
});
