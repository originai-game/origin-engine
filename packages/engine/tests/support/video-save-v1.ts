// Writes tests/fixtures/video-v1/save.json: a save of each game of
// video-save-game.ts written by a build from before og.VideoPlayer's f64
// time (version 1, registered in every world), with each world's hash at
// the save. It is not run by the tests. The fixture was written at
// origin-engine 7998bdb1 (g3-2-video, G3-2 batch 2): build that commit's
// native host into packages/engine/native, copy this file and
// video-save-game.ts into tests/support and run
//   bun tests/support/video-save-v1.ts 7998bdb1 > save.json
import { loadNativeHost } from "@origingame/engine/node";
import { createEngine, type GameToken } from "@origingame/engine/runtime";
import { SAVE_AT, plain, player } from "./video-save-game.ts";

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
console.log(JSON.stringify({ build, tick: SAVE_AT, player: write(player), plain: write(plain) }, null, 2));
