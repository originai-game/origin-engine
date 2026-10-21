// The sounds: one voice entity per sound (og-audio §1), played by the Voices
// system when a system set its bit in Sfx; the music streams on its bus,
// ducked under the effects (og-audio §4).

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Sfx, Sound, SOUND_FILES, Voice } from "./components.ts";
import { must } from "./level.ts";

export function spawnVoices(world: World): void {
  for (const [bit, file] of SOUND_FILES) {
    const e = must(world.spawn({ name: `voice-${file}` }));
    must(world.insert(e, og.AudioSource, { sound: must(og.load.sound(world, `audio/${file}.ogt`)), bus: "sfx", playing: false, priority: bit === Sound.Jump || bit === Sound.Hurt ? 220 : 160 }));
    must(world.insert(e, Voice, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: must(og.load.sound(world, "music/vault.wav")), bus: "music", loop: true, playing: true, volume: 0.55, priority: 250 }));
  // The music bus ducks under the effects (og-audio §4).
  const duck = must(world.spawn({ name: "bus-music" }));
  must(world.insert(duck, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 6, duckRelease: 0.4 }));
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
        if (b === Sound.Step || b === Sound.Gear || b === Sound.Shove) src.pitch[i] = rng.range(0.92, 1.08);
        src.playing[i] = 1;
        count[i] = count[i]! + 1;
        sfx.plays += 1;
        left &= ~b;
      }
    }
  }
  sfx.bits = left;
});
