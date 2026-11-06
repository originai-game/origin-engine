// The sounds: one voice per sound effect, played once when a system sets
// its bit in Game.sfx (a voice still playing is stopped this tick and
// started again on the next), and the arena's music, ducked under the
// effects, with a little of the canyon's echo on them.

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Game, Sfx, Sound } from "./components.ts";
import { must } from "./world.ts";

export function spawnVoices(world: World): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  const voices: [string, number, number, string][] = [
    ["audio/shot.wav", Sound.Shot, 0.55, "sfx"],
    ["audio/reload.wav", Sound.Reload, 0.6, "sfx"],
    ["audio/empty.wav", Sound.Empty, 0.6, "sfx"],
    ["audio/hit.ogt", Sound.Hit, 0.4, "sfx"],
    ["audio/pop.ogt", Sound.Pop, 0.6, "sfx"],
    ["audio/hurt.ogt", Sound.Hurt, 0.55, "sfx"],
    ["audio/down.ogt", Sound.Down, 0.7, "sfx"],
    ["audio/swipe.wav", Sound.Swipe, 0.55, "sfx"],
    ["audio/charge.wav", Sound.Charge, 0.4, "sfx"],
    ["audio/bolt.wav", Sound.Bolt, 0.55, "sfx"],
    ["audio/horn.wav", Sound.Horn, 0.6, "sfx"],
    ["audio/clear.ogt", Sound.Clear, 0.6, "sfx"],
    ["audio/win.wav", Sound.Win, 0.75, "sfx"],
    ["audio/lose.wav", Sound.Lose, 0.75, "sfx"],
    ["audio/step.ogt", Sound.Step, 0.25, "sfx"],
    ["audio/jump.ogt", Sound.Jump, 0.35, "sfx"],
    ["audio/land.ogt", Sound.Land, 0.4, "sfx"],
    // The touch HUD's buttons: the style's press sound on the ui bus, like every og.ui button.
    ["audio/ui-press.ogt", Sound.Tap, 0.5, "ui"],
  ];
  for (const [path, bit, volume, bus] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: bus as "sfx", volume, priority: bit === Sound.Shot ? 200 : 160 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/arena.wav"), bus: "music", loop: true, volume: 0.5, priority: 220 }));
  const fx = must(world.spawn({ name: "music-bus" }));
  must(world.insert(fx, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 4, duckRelease: 0.5 }));
  const echo = must(world.spawn({ name: "sfx-bus" }));
  must(world.insert(echo, og.AudioBusFx, { bus: "sfx", reverb: 0.14 }));
}

/** Plays each requested sound once; shots, hits and steps vary their pitch a little. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Waves"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  // The last system: every system has seen this tick's start.
  g.start = false;
  if (g.sfx === 0) return;
  const rng = must(world.resource(og.Rng));
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
        if (b === Sound.Shot || b === Sound.Hit || b === Sound.Step) src.pitch[i] = rng.range(0.92, 1.08);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
