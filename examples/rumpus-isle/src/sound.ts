// The sounds: one voice entity per sound (og-audio §1), played by the Voices
// system when a system set its bit in Sfx; the music streams on its bus,
// ducked under the effects (og-audio §4), and a limiter keeps the master
// from clipping when four critters bump at once.

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Sfx, Sound, SOUND_FILES, Voice } from "./components.ts";
import { must } from "./util.ts";

export function spawnVoices(world: World): void {
  for (const [bit, file] of SOUND_FILES) {
    const e = must(world.spawn({ name: `voice-${file.replace(/^audio\/|\.(ogt|wav)$/g, "")}` }));
    must(world.insert(e, og.AudioSource, { sound: must(og.load.sound(world, file)), bus: "sfx", playing: false, priority: bit === Sound.Go || bit === Sound.Win ? 230 : 170 }));
    must(world.insert(e, Voice, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: must(og.load.sound(world, "music/romp.wav")), bus: "music", loop: true, playing: true, volume: 0.5, priority: 250 }));
  const duck = must(world.spawn({ name: "bus-music" }));
  must(world.insert(duck, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 6, duckRelease: 0.45 }));
  const master = must(world.spawn({ name: "bus-master" }));
  must(world.insert(master, og.AudioBusFx, { bus: "master", threshold: -1, ratio: 20, attack: 0.001, release: 0.1 }));
}

/** Plays each requested sound once: a voice still playing is stopped this tick and restarted the next (the archetypes' Voices). */
export const Voices = defineSystem("Voices", { writes: [Voice, Sfx, og.AudioSource, og.Rng] }, (q, _time, world) => {
  const sfx = must(world.resource(Sfx));
  if (sfx.bits === 0) return;
  const rng = must(world.resource(og.Rng));
  let left = sfx.bits;
  for (const a of q.archetypes()) {
    const voice = a.column(Voice).bit;
    const count = a.column(Voice).plays;
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = voice[i]!;
      if ((sfx.bits & b) === 0) continue;
      if (src.playing[i]) {
        src.playing[i] = 0;
      } else {
        // Repeated sounds vary a little (og-audio §1).
        if (b === Sound.Bump || b === Sound.Berry || b === Sound.Dash || b === Sound.Crack) src.pitch[i] = rng.range(0.9, 1.1);
        src.playing[i] = 1;
        count[i] = count[i]! + 1;
        sfx.plays += 1;
        left &= ~b;
      }
    }
  }
  sfx.bits = left;
});
