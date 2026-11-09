// og.Lighting2D, og.Light2D and og.Occluder2D in a save: every field comes
// back from a save this build wrote, and a world without 2D lights saves
// and restores as one that never had the types (they are lazy). Saves from
// before engine save format 3 are refused without conversion
// (save-versions.test.ts); there is no migration to test.
//
// Ways it could fail, and the check that catches each:
// - a field read at the wrong offset, or a default written over a saved
//   value → a restored value differs
// - the lighting resource dropped as unused although it was enabled
// - a restore registers the types in a world that had none → its hash
//   differs from the same game run to the same tick
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { SAVE_AT, Ticks, lit, plain } from "./support/light2d-save-game.ts";

beforeAll(() => {
  loadNativeHost();
});

function engineOf(game: unknown): Engine {
  const created = createEngine(game as GameToken);
  if (!created.ok) throw new Error(created.reason);
  return created.value;
}

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

const near = (a: number, b: number) => Math.abs(a - b) < 1e-6;

function saved(game: unknown): { save: string; hash: string } {
  const e = engineOf(game);
  e.advance(SAVE_AT);
  const save = must(e.world.snapshot());
  const hash = e.hash();
  e.destroy();
  return { save, hash };
}

test("saved 2D lights restore with every field", () => {
  const engine = engineOf(plain);
  engine.advance(1);
  expect<unknown>(engine.world.restore(saved(lit).save)).toEqual({ ok: true, value: null });
  expect(engine.world.tick).toBe(SAVE_AT);
  const l = must(engine.world.resource(og.Lighting2D));
  expect([l.enabled, l.maxLayer, l.shadowAngles]).toEqual([true, 40, 1024]);
  expect(near(l.ambient.r, 0.2) && near(l.ambient.g, 0.25) && near(l.ambient.b, 0.3)).toBe(true);
  const lamp = must(engine.world.get(engine.world.find("lamp")!, og.Light2D));
  expect([lamp.radius, lamp.intensity, lamp.falloff, lamp.sourceRadius, lamp.castShadows]).toEqual([6, 2, 1.5, 0.5, false]);
  expect([lamp.kind, lamp.layers, lamp.height]).toEqual(["point", 4294967295, 1]);
  expect(near(lamp.innerAngle, 0.35) && near(lamp.outerAngle, 0.6)).toBe(true);
  const wall = must(engine.world.get(engine.world.find("wall")!, og.Occluder2D));
  expect([wall.width, wall.height, wall.layers, wall.points.length]).toEqual([2, 0.5, 4294967295, 0]);
  engine.advance(5);
  const t = engine.world.resource(Ticks);
  expect(t.ok && t.value.n).toBe(SAVE_AT + 5);
  engine.destroy();
});

test("a world without 2D lights restores as one that never had the types", () => {
  const { save, hash } = saved(plain);
  const restored = engineOf(plain);
  restored.advance(1);
  expect<unknown>(restored.world.restore(save)).toEqual({ ok: true, value: null });
  expect(restored.hash()).toBe(hash);
  restored.destroy();
});
