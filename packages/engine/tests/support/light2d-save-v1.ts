// Writes tests/fixtures/light2d-v1/save.json: a save of each game of
// light2d-save-game.ts written by a build from before R11 (og.Lighting2D,
// og.Light2D and og.Occluder2D at version 1, registered in every world),
// with each world's hash at the save. It is not run by the tests. The
// fixture was written at origin-engine c6986333 (main before R11): build
// that commit's native host into packages/engine/native, copy this file
// and light2d-save-game.ts into tests/support and run
//   bun tests/support/light2d-save-v1.ts c6986333 > save.json
import { loadNativeHost } from "@origingame/engine/node";
import { createEngine, type GameToken } from "@origingame/engine/runtime";
import { SAVE_AT, lit, plain } from "./light2d-save-game.ts";

loadNativeHost();
const write = (game: unknown) => {
  const created = createEngine(game as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  engine.advance(SAVE_AT);
  const save = engine.world.snapshot();
  if (!save.ok) throw new Error(save.reason);
  const out = { save: save.value, hash: engine.hash() };
  engine.destroy();
  return out;
};
const build = process.argv[2] ?? "unknown";
console.log(JSON.stringify({ build, tick: SAVE_AT, lit: write(lit), plain: write(plain) }, null, 2));
