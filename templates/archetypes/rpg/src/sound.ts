// The sounds: one voice per effect, played once when a system sets its bit
// in Game.sfx (a voice still playing is stopped this tick and started
// again on the next), and the yard's music, ducked under the effects, with
// a little of the courtyard's stone echo on them.

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Game, Sfx, Sound } from "./components.ts";
import { must } from "./world.ts";

export function spawnVoices(world: World): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  const voices: [string, number, number][] = [
    ["audio/step.ogg", Sound.Step, 0.32],
    ["audio/swing.ogg", Sound.Swing, 0.55],
    ["audio/hit.ogg", Sound.Hit, 0.8],
    ["audio/clang.ogg", Sound.Clang, 0.6],
    ["audio/bones.ogg", Sound.Bones, 0.75],
    ["audio/hurt.ogt", Sound.Hurt, 0.6],
    ["audio/potion.ogt", Sound.Potion, 0.55],
    ["audio/coin.ogt", Sound.Coin, 0.5],
    ["audio/levelup.ogt", Sound.LevelUp, 0.65],
    ["audio/alarm.ogt", Sound.Alarm, 0.35],
    ["audio/chest.ogg", Sound.Chest, 0.8],
    ["audio/crush.ogg", Sound.Crush, 0.75],
    ["audio/quest.wav", Sound.Quest, 0.6],
    ["audio/won.wav", Sound.Win, 0.75],
    ["audio/fallen.wav", Sound.Lose, 0.75],
  ];
  for (const [path, bit, volume] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: "sfx", volume, priority: 160 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/keep.wav"), bus: "music", loop: true, volume: 0.5, priority: 220 }));
  const fx = must(world.spawn({ name: "music-bus" }));
  must(world.insert(fx, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 4, duckRelease: 0.6 }));
  const echo = must(world.spawn({ name: "sfx-bus" }));
  must(world.insert(echo, og.AudioBusFx, { bus: "sfx", reverb: 0.22 }));
  // A reply picked: the style's press sound on the ui bus, like every og.ui button.
  const tap = must(world.spawn({ name: `sfx-${Sound.Tap}` }));
  must(world.insert(tap, og.AudioSource, { sound: sound("audio/ui-press.ogt"), playing: false, bus: "ui", volume: 0.5, priority: 150 }));
  must(world.insert(tap, Sfx, { bit: Sound.Tap }));
}

/** Plays each requested sound once; steps, swings and blows vary their pitch a little. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Foes", "Story"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  // The last system: every system has seen this tick's start.
  g.start = false;
  if (g.sfx === 0) return;
  const rng = must(world.resource(og.Rng));
  const varied = Sound.Step | Sound.Swing | Sound.Hit | Sound.Bones | Sound.Clang;
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
        if ((b & varied) !== 0) src.pitch[i] = rng.range(0.9, 1.1);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
