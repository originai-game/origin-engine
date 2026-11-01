// Writes tests/fixtures/gamepad-v1/save.json: a save of pad-save-game.ts
// written by a build from before og.Gamepad's eight pads (og.Gamepad
// version 1, four pads, no og.input.actions block), and the game's state at
// the save and after AFTER more ticks of the same pad script. It is not run
// by the tests. The fixture was written at origin-engine 3aca1209: check
// that commit out, build its native host into packages/engine/native, copy
// this file and pad-save-game.ts into tests/support and run
//   bun tests/support/pad-save-v1.ts 3aca1209 > save.json
// (a release's saves, 0.7.7's, are older still: their og.ui block is
// version 9, which this build refuses for its own reason.)
import { loadNativeHost } from "@origingame/engine/node";
import { createEngine, type GameToken } from "@origingame/engine/runtime";
import { AFTER, PadRun, SAVE_AT, game, padAt } from "./pad-save-game.ts";

loadNativeHost();
const created = createEngine(game as unknown as GameToken);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const run = () => {
  const r = engine.world.resource(PadRun);
  if (!r.ok) throw new Error(r.reason);
  return { ...(r.value as object) };
};
for (let t = 0; t < SAVE_AT; t++) {
  engine.probe.input(padAt(t) as never);
  engine.advance(1);
}
const save = engine.world.snapshot();
if (!save.ok) throw new Error(save.reason);
const atSave = run();
for (let t = SAVE_AT; t < SAVE_AT + AFTER; t++) {
  engine.probe.input(padAt(t) as never);
  engine.advance(1);
}
const build = process.argv[2] ?? "unknown";
console.log(JSON.stringify({ build, tick: SAVE_AT, save: save.value, atSave, after: run() }, null, 2));
