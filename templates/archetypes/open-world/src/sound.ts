// The sounds: one voice per sound effect, played once when a system sets
// its bit in Game.sfx (a voice still playing is stopped this tick and
// started again on the next), the island's music ducked under the
// effects, and the surf's loop under everything.

import { og, defineSystem } from "@origingame/engine";
import type { AudioBus, World } from "@origingame/engine";
import { Game, Sfx, Sound } from "./components.ts";
import { must } from "./world.ts";

export function spawnVoices(world: World): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  const voices: [string, number, number, AudioBus][] = [
    ["audio/step.ogt", Sound.Step, 0.3, "sfx"],
    ["audio/jump.wav", Sound.Jump, 0.4, "sfx"],
    ["audio/splash.wav", Sound.Splash, 0.5, "sfx"],
    ["audio/chop.wav", Sound.Chop, 0.6, "sfx"],
    ["audio/stone.wav", Sound.Stone, 0.55, "sfx"],
    ["audio/pickup.wav", Sound.Pickup, 0.55, "sfx"],
    ["audio/claim.wav", Sound.Claim, 0.75, "sfx"],
    ["audio/discover.wav", Sound.Discover, 0.55, "sfx"],
    ["audio/build.wav", Sound.Build, 0.7, "sfx"],
    ["audio/dawn.wav", Sound.Dawn, 0.75, "sfx"],
    ["audio/night.wav", Sound.Night, 0.75, "sfx"],
    ["audio/collapse.wav", Sound.Lost, 0.6, "sfx"],
    // The touch HUD's buttons: the press sound on the ui bus, like every og.ui button.
    ["audio/ui-press.ogt", Sound.Tap, 0.5, "ui"],
  ];
  for (const [path, bit, volume, bus] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus, volume, priority: 160 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/isle.wav"), bus: "music", loop: true, volume: 0.5, priority: 220 }));
  const surf = must(world.spawn({ name: "surf" }));
  must(world.insert(surf, og.AudioSource, { sound: sound("audio/surf.wav"), bus: "sfx", loop: true, volume: 0.22, priority: 230 }));
  const fx = must(world.spawn({ name: "music-bus" }));
  must(world.insert(fx, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 4, duckRelease: 0.6 }));
  const echo = must(world.spawn({ name: "sfx-bus" }));
  must(world.insert(echo, og.AudioBusFx, { bus: "sfx", reverb: 0.12 }));
}

/** Plays each requested sound once; steps, strokes and splashes vary their pitch a little. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Work", "Eye"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  // The last system: every system has seen this tick's start.
  g.start = false;
  if (g.sfx === 0) return;
  const rng = must(world.resource(og.Rng));
  let left = g.sfx;
  const varied = Sound.Step | Sound.Splash | Sound.Chop | Sound.Stone;
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
        if ((b & varied) !== 0) src.pitch[i] = rng.range(0.88, 1.12);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});
