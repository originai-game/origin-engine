// A save written before og.Gamepad's eight pads (origin-engine 3aca1209:
// og.Gamepad version 1, four pads, and no og.input.actions state block)
// restores on this build without `migrate` and the world goes on as it did
// on the build that saved. The fixture is tests/fixtures/gamepad-v1/save.json,
// written by tests/support/pad-save-v1.ts on that build.
//
// Ways it could fail, and the check that catches each:
// - the save refused because a session type (og.Gamepad, which a restore
//   replaces with the host's) changed layout → `restored.ok` false
// - the game's own state not brought back → `atSave` differs
// - the restored world going on otherwise than the old build did (pads
//   mapped to the wrong slots, the action map's missing block) → `after`
//   differs
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { AFTER, PadRun, SAVE_AT, game, padAt } from "./support/pad-save-game.ts";

const fixture = JSON.parse(readFileSync(join(dirname(fileURLToPath(import.meta.url)), "fixtures", "gamepad-v1", "save.json"), "utf8")) as {
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

test("a four-pad save restores without migrate and replays as that build did", () => {
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
