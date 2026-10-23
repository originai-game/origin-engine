// The sounds: one voice per sound effect, played once when a system sets
// its bit in Game.sfx (a voice still playing is stopped this tick and
// started again on the next), and the vale's music, ducked under the
// effects, with a little of the valley's echo on them.

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Game, Sfx, Sound } from "./components.ts";
import { must } from "./world.ts";

export function spawnVoices(world: World): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  const voices: [string, number, number][] = [
    ["audio/step.ogt", Sound.Step, 0.3],
    ["audio/jump.wav", Sound.Jump, 0.45],
    ["audio/splash.wav", Sound.Splash, 0.5],
    ["audio/ignite.wav", Sound.Ignite, 0.75],
    ["audio/alarm.wav", Sound.Alarm, 0.6],
    ["audio/grab.wav", Sound.Grab, 0.7],
    ["audio/dawn.wav", Sound.Dawn, 0.75],
    ["audio/night.wav", Sound.Night, 0.75],
  ];
  for (const [path, bit, volume] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: "sfx", volume, priority: 160 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/vale.wav"), bus: "music", loop: true, volume: 0.55, priority: 220 }));
  const fx = must(world.spawn({ name: "music-bus" }));
  must(world.insert(fx, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 5, duckRelease: 0.6 }));
  const echo = must(world.spawn({ name: "sfx-bus" }));
  must(world.insert(echo, og.AudioBusFx, { bus: "sfx", reverb: 0.18 }));
  // The touch HUD's Jump and Kindle: the style's press sound on the ui bus,
  // like every og.ui button, beside the jump's or the fire's own effect.
  const tap = must(world.spawn({ name: `sfx-${Sound.Tap}` }));
  must(world.insert(tap, og.AudioSource, { sound: sound("audio/ui-press.ogt"), playing: false, bus: "ui", volume: 0.5, priority: 150 }));
  must(world.insert(tap, Sfx, { bit: Sound.Tap }));
}

/** Plays each requested sound once; steps and splashes vary their pitch a little. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Beacons", "Sentinels", "View"] }, (q, _time, world) => {
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
        if (b === Sound.Step || b === Sound.Splash) src.pitch[i] = rng.range(0.9, 1.1);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
