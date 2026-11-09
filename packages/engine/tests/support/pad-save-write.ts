// Writes tests/fixtures/gamepad-save/save.json: a save of pad-save-game.ts
// taken at SAVE_AT, and the game's state at the save and after AFTER more
// ticks of the same pad script. It is not run by the tests; run it again
// whenever the engine's save format changes (saves are not migrated):
//   bun tests/support/pad-save-write.ts $(git rev-parse --short HEAD) > tests/fixtures/gamepad-save/save.json
// from packages/engine, with this checkout's native host built.
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
