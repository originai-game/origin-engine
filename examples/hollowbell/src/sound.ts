// The sounds: one voice per effect (og sfx presets cooked from
// assets/audio/*.ogt, and the synthesised bell), played once when a system
// sets its bit in Game.sfx; the music (assets/music/hollow.wav, cooked in
// 4-second chunks and streamed) looping on the music bus from the title,
// ducked under the effects; a little of the cave's echo on the effects and
// a limiter on the master bus (og-audio §4).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, Sfx, Sound } from "./components.ts";
import { must } from "./world.ts";

export function spawnVoices(world: World, g: Value<typeof Game>): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  const voices: [string, number, number, number][] = [
    ["audio/jump.ogt", Sound.Jump, 0.45, 200],
    ["audio/land.ogt", Sound.Land, 0.5, 150],
    ["audio/step.ogt", Sound.Step, 0.3, 120],
    ["audio/swing.ogt", Sound.Swing, 0.5, 190],
    ["audio/hit.ogt", Sound.Hit, 0.65, 210],
    ["audio/hurt.ogt", Sound.Hurt, 0.7, 215],
    ["audio/glimmer.ogt", Sound.Glimmer, 0.45, 160],
    ["audio/chime.ogt", Sound.Chime, 0.7, 220],
    ["audio/buy.ogt", Sound.Buy, 0.6, 200],
    ["audio/checkpoint.ogt", Sound.Checkpoint, 0.6, 200],
    ["audio/boss.ogt", Sound.Boss, 0.8, 230],
    ["audio/win.ogt", Sound.Win, 0.75, 230],
    ["audio/bell.wav", Sound.Bell, 0.9, 235],
    ["audio/ui-error.ogt", Sound.Refuse, 0.5, 180],
    ["audio/step.ogt", Sound.Drop, 0.35, 120],
  ];
  for (const [path, bit, volume, priority] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: "sfx", volume, priority }));
    must(world.insert(v, Sfx, { bit }));
  }
  const track = sound("music/hollow.wav");
  g.music = track as unknown as number;
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: track, bus: "music", loop: true, volume: 0.55, priority: 240 }));
  must(world.insert(must(world.spawn({ name: "music-bus" })), og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 8, duckRelease: 0.5 }));
  must(world.insert(must(world.spawn({ name: "sfx-bus" })), og.AudioBusFx, { bus: "sfx", reverb: 0.2 }));
  must(world.insert(must(world.spawn({ name: "master-bus" })), og.AudioBusFx, { bus: "master", threshold: -1, ratio: 20, attack: 0.001 }));
}

/** Plays each requested sound once; steps, hits and glimmers vary their pitch a little (og-audio §1). */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Effects"] }, (q, _time, world) => {
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
        if (b === Sound.Step || b === Sound.Hit || b === Sound.Glimmer) src.pitch[i] = rng.range(0.92, 1.08);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
