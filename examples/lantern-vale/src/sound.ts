// The sounds: one voice per effect (og sfx presets cooked from
// assets/audio/*.ogt), played once when a system sets its bit in Game.sfx;
// the 4-minute music (assets/music/dusk.wav, cooked in 4-second chunks and
// streamed) looping on the music bus, ducked under the effects; a little
// of the valley's echo on the effects and a limiter on the master bus.

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Game, Sfx, Sound } from "./components.ts";
import { must } from "./world.ts";

export function spawnVoices(world: World): void {
  const g = must(world.resource(Game));
  const sound = (path: string) => must(og.load.sound(world, path));
  const voices: [string, number, number, number][] = [
    ["audio/footstep.ogt", Sound.Step, 0.32, 120],
    ["audio/powerup.ogt", Sound.Light, 0.7, 200],
    ["audio/hurt.ogt", Sound.Hurt, 0.75, 210],
    ["audio/hit.ogt", Sound.Hit, 0.55, 205],
    ["audio/levelup.ogt", Sound.Win, 0.8, 230],
    ["audio/death.ogt", Sound.Lose, 0.7, 230],
    ["audio/whoosh.ogt", Sound.Kindle, 0.4, 150],
    ["audio/pickup.ogt", Sound.Alert, 0.45, 160],
  ];
  for (const [path, bit, volume, priority] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: "sfx", volume, priority }));
    must(world.insert(v, Sfx, { bit }));
  }
  const track = sound("music/dusk.wav");
  g.music = track as unknown as number;
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: track, bus: "music", loop: true, volume: 0.6, priority: 240 }));
  must(world.insert(must(world.spawn({ name: "music-bus" })), og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 8, duckRelease: 0.5 }));
  must(world.insert(must(world.spawn({ name: "sfx-bus" })), og.AudioBusFx, { bus: "sfx", reverb: 0.15 }));
  must(world.insert(must(world.spawn({ name: "master-bus" })), og.AudioBusFx, { bus: "master", threshold: -1, ratio: 20, attack: 0.001 }));
}

/** Plays each requested sound once; steps vary their pitch a little. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Slimes", "View"] }, (q, _time, world) => {
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
        // Still sounding: stopped this tick, started again on the next.
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        sfx.plays[i] = sfx.plays[i]! + 1;
        if (b === Sound.Step) src.pitch[i] = rng.range(0.9, 1.1);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
