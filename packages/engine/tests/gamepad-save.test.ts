// A recorded save of a game that follows pad 1 restores and the world goes
// on as it did on the build that saved. The fixture is
// tests/fixtures/gamepad-save/save.json, written by
// tests/support/pad-save-write.ts at the current save format.
//
// Ways it could fail, and the check that catches each:
// - the save refused because a session type (og.Gamepad, which a restore
//   replaces with the host's) is saved where it should not be → `restored.ok` false
// - the game's own state not brought back → `atSave` differs
// - the restored world going on otherwise than the saving build did (pads
//   mapped to the wrong slots, the action map's state lost) → `after`
//   differs
// - pads past the fourth not reaching the restored world → pad 6's `a`
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { AFTER, PadRun, SAVE_AT, game, padAt } from "./support/pad-save-game.ts";

const fixture = JSON.parse(readFileSync(join(dirname(fileURLToPath(import.meta.url)), "fixtures", "gamepad-save", "save.json"), "utf8")) as {
  build: string;
  tick: number;
  save: string;
  atSave: { presses: number; x: number; connected: number };
  after: { presses: number; x: number; connected: number };
};

function run(engine: Engine): { presses: number; x: number; connected: number } {
  const r = engine.world.resource(PadRun);
  if (!r.ok) throw new Error(r.reason);
  return { ...(r.value as { presses: number; x: number; connected: number }) };
}

beforeAll(() => {
  loadNativeHost();
});

test("a pad save restores and replays as the build that saved it did", () => {
  const created = createEngine(game as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  // The host's pad 1 as it was when the save was taken: a restore keeps the session.
  engine.probe.input(padAt(SAVE_AT - 1) as never);
  engine.advance(1);

  const restored = engine.world.restore(fixture.save);
  expect<unknown>(restored).toEqual({ ok: true, value: null });
  expect(engine.world.tick).toBe(fixture.tick);
  expect(run(engine)).toEqual(fixture.atSave);

  for (let t = SAVE_AT; t < SAVE_AT + AFTER; t++) {
    engine.probe.input(padAt(t) as never);
    engine.advance(1);
  }
  expect(run(engine)).toEqual(fixture.after);
  // Pads past the fourth work in the restored world.
  engine.probe.input({ kind: "gamepad", pad: 6, buttons: ["a"] } as never);
  engine.advance(1);
  const pads = engine.world.resource(og.Gamepad);
  expect(pads.ok && pads.value.down("a", 6)).toBe(true);
  engine.destroy();
});
