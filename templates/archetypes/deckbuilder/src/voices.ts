// Plays each requested sound once (Game.sfx's bits, src/components.ts's
// Sound): a voice already playing is stopped this tick and restarted the
// next. The hits and the cards vary their pitch a little (og.Rng).

import { og, defineSystem } from "@origingame/engine";
import { Game, Sfx, Sound } from "./components.ts";

export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Stage"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (g.sfx === 0) return;
  const rng = world.resource(og.Rng);
  let left = g.sfx;
  for (const a of q.archetypes()) {
    const sfx = a.column(Sfx);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = sfx.bit[i]!;
      if ((g.sfx & b) === 0) continue;
      if (src.playing[i]) {
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        sfx.plays[i] = sfx.plays[i]! + 1;
        if ((b === Sound.Card || b === Sound.Hit || b === Sound.Draw) && rng.ok) src.pitch[i] = rng.value.range(0.92, 1.08);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
