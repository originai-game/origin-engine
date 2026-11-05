// Saves written before R11 (og.Lighting2D, og.Light2D and og.Occluder2D
// at version 1, registered in every world) restore on this build without
// `migrate`: the engine's steps (save_migrations, each 1 -> 2) keep every
// saved field by name and give the new ones their defaults (a point light
// on every layer, an occluder on every layer, 1024 shadow angles), and
// drop the types from a save that holds no 2D light, so that world
// restores as one that never had them (they are lazy now). The fixture is
// tests/fixtures/light2d-v1/save.json, written by
// tests/support/light2d-save-v1.ts at c6986333.
//
// Ways it could fail, and the check that catches each:
// - the old save refused for the changed layouts → `restore` not ok
// - a field shifted by the new ones (read at the wrong offset) → a saved
//   value differs, or a new field is not its default
// - the lighting resource dropped as unused although it was enabled
// - a save without 2D lights registers the types on restore (its hash then
//   differs from the same game run here to the same tick)
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { SAVE_AT, Ticks, plain } from "./support/light2d-save-game.ts";
import { asSaved } from "./support/saved-defaults.ts";

const fixture = JSON.parse(readFileSync(join(dirname(fileURLToPath(import.meta.url)), "fixtures", "light2d-v1", "save.json"), "utf8")) as {
  build: string;
  tick: number;
  lit: { save: string; hash: string };
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

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

const near = (a: number, b: number) => Math.abs(a - b) < 1e-6;

test("a save with version-1 2D lights restores with every field and the new defaults", () => {
  const engine = engineOf(plain);
  engine.advance(1);
  expect(engine.world.restore(fixture.lit.save)).toEqual({ ok: true, value: null });
  expect(engine.world.tick).toBe(fixture.tick);
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

test("a save without 2D lights restores as a world that never had the types", () => {
  const here = engineOf(plain);
  here.advance(SAVE_AT);
  const restored = engineOf(plain);
  restored.advance(1);
  expect(restored.world.restore(fixture.plain.save)).toEqual({ ok: true, value: null });
  expect(restored.world.tick).toBe(fixture.tick);
  // The SSR defaults moved since the fixture was written; the save keeps its own.
  asSaved(here);
  expect(restored.hash()).toBe(here.hash());
  // The old build registered the three types in every world, so its hash
  // of the same world was another (the one move of a world without lights).
  expect(fixture.plain.hash).not.toBe(here.hash());
  here.destroy();
  restored.destroy();
});
